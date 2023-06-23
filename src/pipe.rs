/*
    Copyright (c) 2007-2016 Contributors as noted in the AUTHORS file

    This file is part of libzmq, the ZeroMQ core engine in C+= 1.

    libzmq is free software; you can redistribute it and/or modify it under
    the terms of the GNU Lesser General Public License (LGPL) as published
    by the Free Software Foundation; either version 3 of the License, or
    (at your option) any later version.

    As a special exception, the Contributors give you permission to link
    this library with independent modules to produce an executable,
    regardless of the license terms of these independent modules, and to
    copy and distribute the resulting executable under terms of your choice,
    provided that you also meet, for each linked independent module, the
    terms and conditions of the license of that module. An independent
    module is a module which is not derived from or based on this library.
    If you modify this library, you must extend this exception to your
    version of the library.

    libzmq is distributed in the hope that it will be useful, but WITHOUT
    ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
    FITNESS FOR A PARTICULAR PURPOSE. See the GNU Lesser General Public
    License for more details.

    You should have received a copy of the GNU Lesser General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

// #include "precompiled.hpp"
// #include <new>
// #include <stddef.h>

// #include "macros.hpp"
// #include "pipe.hpp"
// #include "err.hpp"

// #include "ypipe.hpp"
// #include "ypipe_conflate.hpp"


//  Create a pipepair for bi-directional transfer of messages.
//  First HWM is for messages passed from first pipe to the second pipe.
//  Second HWM is for messages passed from second pipe to the first pipe.
//  Delay specifies how the pipe behaves when the peer terminates. If true
//  pipe receives all the pending messages before terminating, otherwise it
//  terminates straight away.
//  If conflate is true, only the most recently arrived message could be
//  read (older messages are discarded)

use std::collections::VecDeque;
use std::io::Write;
use std::ptr::null_mut;
use libc::memcpy;
use serde::{Deserialize, Serialize};
use crate::context::ZmqContext;
use crate::endpoint_uri::EndpointUriPair;
use crate::ypipe_base::YpipeBase;
use crate::message::{ZMQ_MSG_MORE, ZMQ_MSG_ROUTING_ID, ZmqMessage};

use crate::own::ZmqOwn;
use crate::pipe::PipeState::{active, delimiter_received, term_ack_sent, term_req_sent1, term_req_sent2, waiting_for_delimiter};
use crate::session_base::ZmqSessionBase;
use crate::socket::ZmqSocket;
use crate::ypipe::Ypipe;

// int pipepair (ZmqObject *parents_[2],
// ZmqPipe *pipes_[2],
// const int hwms_[2],
//               const bool conflate_[2]);

pub trait i_pipe_events {
    // virtual ~i_pipe_events () ZMQ_DEFAULT; fn read_activated(&mut self, pipe: &mut ZmqPipe);
    fn write_activated(&mut self, pipe: &mut ZmqPipe);
    fn hiccuped(&mut self, pipe: &mut ZmqPipe);
    fn pipe_terminated(&mut self, pipe: &mut ZmqPipe);
}

//  States of the pipe endpoint:
//  active: common state before any termination begins,
//  delimiter_received: delimiter was read from pipe before
//      Term command was received,
//  waiting_for_delimiter: Term command was already received
//      from the peer but there are still pending messages to read,
//  term_ack_sent: all pending messages were already read and
//      all we are waiting for is ack from the peer,
//  term_req_sent1: 'terminate' was explicitly called by the user,
//  term_req_sent2: user called 'terminate' and then we've got
//      Term command from the peer as well.
pub enum PipeState {
    active,
    delimiter_received,
    waiting_for_delimiter,
    term_ack_sent,
    term_req_sent1,
    term_req_sent2,
}

//  Note that pipe can be stored in three different arrays.
//  The array of inbound pipes (1), the array of outbound pipes (2) and
//  the generic array of pipes to be deallocated (3).
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
// pub struct ZmqPipe  : public ZmqObject,
//                          public array_ZmqItem<1>,
//                          public array_ZmqItem<2>,
//                          public array_ZmqItem<3>
pub struct ZmqPipe {
    // typedef YpipeBase<ZmqMessage> upipe_t;
    //  Underlying pipes for both directions.
    // upipe_t *_in_pipe;
    pub in_pipe: VecDeque<ZmqMessage>,
    // upipe_t *_out_pipe;
    pub out_pipe: VecDeque<ZmqMessage>,
    //  Can the pipe be read from / written to?
    pub in_active: bool,
    pub out_active: bool,
    //  High watermark for the outbound pipe.
    pub hwm: u32,
    //  Low watermark for the inbound pipe.
    pub lwm: u32,
    // boosts for high and low watermarks, used with inproc sockets so hwm are sum of send and recv hmws on each side of pipe
    pub in_hwm_boost: u32,
    pub out_hwm_boost: u32,
    //  Number of messages read and written so far.
    pub msgs_read: u64,
    pub msgs_written: u64,
    //  Last received peer's msgs_read. The actual number in the peer
    //  can be higher at the moment.
    // u64 _peers_msgs_read;
    pub peers_msgs_read: u64,
    //  The pipe object on the other side of the pipepair.
    // TODO:
    // ZmqPipe *_peer;
    //  Sink to send events to.
    // i_pipe_events *_sink;
    //  If true, we receive all the pending inbound messages before
    //  terminating. If false, we terminate immediately when the peer
    //  asks us to.
    pub delay: bool,
    //  Routing id of the writer. Used uniquely by the reader side.
    pub router_socket_routing_id: Vec<u8>,
    //  Routing id of the writer. Used uniquely by the reader side.
    pub server_socket_routing_id: i32,
    pub conflate: bool,
    // The endpoints of this pipe.
    pub endpoint_pair: Vec<EndpointUriPair>,
    // Disconnect msg
    pub disconnect_msg: ZmqMessage,

    // // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqPipe)
}

impl ZmqPipe {
    //  This allows pipepair to create pipe objects.
    // friend int pipepair (ZmqObject *parents_[2],
    // ZmqPipe *pipes_[2],
    // const int hwms_[2],
    // const bool conflate_[2]);
    pub fn pipepair(&mut self, parents: (&mut ZmqSessionBase, &mut ZmqSocket), pipes: &mut [Self], hwms: &[i32; 2], conflate: &[bool; 2]) -> anyhow::Result<()> {
        //   Creates two pipe objects. These objects are connected by two ypipes,
        //   each to pass messages in one direction.

        // typedef Ypipe<ZmqMessage, message_pipe_granularity> upipe_normal_t;
        // typedef YpipeConflate<ZmqMessage> upipe_conflate_t;

        // ZmqPipe::upipe_t *upipe1;
        // typedef YpipeBase<ZmqMessage> upipe_t;
        let mut upipe1: VecDeque<ZmqMessage> = VecDeque::new();
        // if (conflate_[0]) {
        //     upipe1 = upipe_conflate_t();
        // }
        // else {
        //     upipe1 = upipe_normal_t();
        // }
        // alloc_assert (upipe1);

        // ZmqPipe::upipe_t *upipe2;
        let mut upipe2: VecDeque<ZmqMessage> = VecDeque::new();
        // if (conflate_[1]) {
        //     upipe2 = upipe_conflate_t();
        // }
        // else {
        //     upipe2 = upipe_normal_t();
        // }
        // alloc_assert (upipe2);

        pipes[0] = ZmqPipe::new(parents[0], &mut upipe1, &mut upipe2, hwms[1], hwms[0], conflate[0]);
        // alloc_assert (pipes_[0]);
        pipes[1] = ZmqPipe::new(parents[1], &mut upipe2, &mut upipe1, hwms[0], hwms[1], conflate[1]);
        // alloc_assert (pipes_[1]);

        pipes[0].set_peer(&mut pipes[1]);
        pipes[1].set_peer(&mut pipes[0]);

        Ok(())
    }

    //
    //  Specifies the object to send events to.
    pub fn set_event_sink(&mut self, sink: &mut i_pipe_events) {
        // Sink can be set once only.
        // zmq_assert (!_sink);
        self._sink = sink;
    }

    //  Pipe endpoint can store an routing ID to be used by its clients.
    pub fn set_server_socket_routing_id(&mut self, server_socket_routing_id_: u32) {
        self.server_socket_routing_id = server_socket_routing_id_ as i32;
    }

    // u32 get_server_socket_routing_id () const;
    pub fn get_server_socket_routing_id(&mut self) -> u32 {
        self.server_socket_routing_id as u32
    }

    //  Pipe endpoint can store an opaque ID to be used by its clients.
    pub fn set_router_socket_routing_id(&mut self, router_socket_routing_id_: &mut Vec<u8>) {
        self.router_socket_routing_id = router_socket_routing_id_.clone();
    }

    // const Blob &get_routing_id () const;
    pub fn get_routing_id(&self) -> &Vec<u8> {
        &self.router_socket_routing_id
    }

    //  Returns true if there is at least one message to read in the pipe.
    // bool check_read ();
    pub fn check_read(&mut self) -> bool {
        if ((!self.in_active)) {
            return false;
        }
        if ((self._state != active && self._state != waiting_for_delimiter)) {
            return false;
        }

        //  Check if there's an item in the pipe.
        if (!self.in_pipe.check_read()) {
            self.in_active = false;
            return false;
        }

        //  If the next item in the pipe is message delimiter,
        //  initiate termination process.
        if (self.in_pipe.probe(is_delimiter)) {
            // ZmqMessage msg;
            let mut msg = ZmqMessage::default();
            let ok = self.in_pipe.read(msg);
            // zmq_assert (ok);
            self.process_delimiter();
            return false;
        }

        return true;
    }

    //  Reads a message to the underlying pipe.
    // bool read (msg: &mut ZmqMessage);
    pub fn read(&mut self, msg: &mut ZmqMessage) -> bool {
        if ((!self.in_active)) {
            return false;
        }
        if ((self._state != active && self._state != waiting_for_delimiter)) {
            return false;
        }

        loop {
            if (!self.in_pipe.read(msg)) {
                self.in_active = false;
                return false;
            }

            //  If this is a credential, ignore it and receive next message.
            if ((msg.is_credential())) {
                msg.close();
                // zmq_assert (rc == 0);
            } else {
                break;
            }
        }

        //  If delimiter was read, start termination process of the pipe.
        if (msg.is_delimiter()) {
            self.process_delimiter();
            return false;
        }

        if ((msg.flags() & ZMQ_MSG_MORE) == 0 && !msg.is_routing_id()) {
            self.msgs_read += 1;
        }

        if (_lwm > 0 && _msgs_read % _lwm == 0) {
            send_activate_write(_peer, _msgs_read);
        }

        return true;
    }

    //  Checks whether messages can be written to the pipe. If the pipe is
    //  closed or if writing the message would cause high watermark the
    //  function returns false.
    // bool check_write ();
    pub fn check_write(&mut self) -> bool {
        if ((!self.out_active || self._state != PipeState::active)) {
            return false;
        }

        let full = !self.check_hwm();

        if ((full)) {
            self.out_active = false;
            return false;
        }

        return true;
    }

    //  Writes a message to the underlying pipe. Returns false if the
    //  message does not pass check_write. If false, the message object
    //  retains ownership of its message buffer.
    // bool write (const msg: &mut ZmqMessage);
    pub fn write(&mut self, msg: &mut ZmqMessage) -> bool {
        if ((!check_write())) {
            return false;
        }

        let more = (msg.flags() & ZMQ_MSG_MORE) != 0;
        let is_routing_id = msg.is_routing_id();
        self.out_pipe.write(msg, more);
        if (!more && !is_routing_id) {
            self.msgs_written += 1;
        }

        return true;
    }

    //  Remove unfinished parts of the outbound message from the pipe.
    // void rollback () const;
    pub fn rollback(&mut self) -> anyhow::Result<()> {
        //  Remove incomplete message from the outbound pipe.
        let mut msg = ZmqMessage::default();
        if (_out_pipe) {
            while (_out_pipe.unwrite(&msg)) {
                // zmq_assert (msg.flags () & ZMQ_MSG_MORE);
                msg.close()?;
                // errno_assert (rc == 0);
            }
        }
        Ok(())
    }

    //  Flush the messages downstream.
    // void flush ();
    pub fn flush(&mut self) {
        //  The peer does not exist anymore at this point.
        if (self._state == PipeState::term_ack_sent) {
            return;
        }

        if (self.out_pipe.len() > 0 && !self.out_pipe.flush()) {
            self.send_activate_read(self._peer);
        }
    }

    //  Temporarily disconnects the inbound message stream and drops
    //  all the messages on the fly. Causes 'hiccuped' event to be generated
    //  in the peer.
    pub fn hiccup(&mut self) {
        //  If termination is already under way do nothing.
        if (_state != PipeState::active) {
            return;
        }

        //  We'll drop the pointer to the inpipe. From now on, the peer is
        //  responsible for deallocating it.

        //  Create new inpipe.
        // self._in_pipe =
        //     if self._conflate
        //         ?  ( YpipeConflate<ZmqMessage> ())
        // :  Ypipe<ZmqMessage, message_pipe_granularity> ();
        self.in_pipe.clear();

        // alloc_assert (_in_pipe);
        _in_active = true;

        //  Notify the peer about the Hiccup.
        send_hiccup(_peer, _in_pipe);
    }

    //  Ensure the pipe won't block on receiving PipeTerm.
    pub fn set_nodelay(&mut self) {
        self.delay = false;
    }

    //  Ask pipe to terminate. The termination will happen asynchronously
    //  and user will be notified about actual deallocation by 'terminated'
    //  event. If delay is true, the pending messages will be processed
    //  before actual shutdown.
    pub fn terminate(&mut self, delay_: bool) {
        //  Overload the value specified at pipe creation.
        self.delay = delay_;

        //  If terminate was already called, we can ignore the duplicate invocation.
        if (self._state == PeerState::term_req_sent1 || self._state == PeerState::term_req_sent2) {
            return;
        }
        //  If the pipe is in the final phase of async termination, it's going to
        //  closed anyway. No need to do anything special here.
        if (self._state == PeerState::term_ack_sent) {
            return;
        }
        //  The simple sync termination case. Ask the peer to terminate and wait
        //  for the ack.
        if (self._state == PeerState::active) {
            self.send_pipe_term(self._peer);
            self._state = PeerState::term_req_sent1;
        }
        //  There are still pending messages available, but the user calls
        //  'terminate'. We can act as if all the pending messages were read.
        else if (self._state == PeerState::waiting_for_delimiter && !self.delay) {
            //  Drop any unfinished outbound messages.
            self.rollback();
            self.out_pipe.clear();
            self.send_pipe_term_ack(self._peer);
            self._state = PeerState::term_ack_sent;
        }
        //  If there are pending messages still available, do nothing.
        else if (self._state == PeerState::waiting_for_delimiter) {}
        //  We've already got delimiter, but not Term command yet. We can ignore
        //  the delimiter and ack synchronously terminate as if we were in
        //  active state.
        else if (self._state == PeerState::delimiter_received) {
            self.send_pipe_term(self._peer);
            self._state = PeerState::term_req_sent1;
        }
        //  There are no other states.
        else {
            // zmq_assert (false);
        }

        //  Stop outbound flow of messages.
        self.out_active = false;

        if (self.out_pipe) {
            //  Drop any unfinished outbound messages.
            self.rollback();

            //  Write the delimiter into the pipe. Note that watermarks are not
            //  checked; thus the delimiter can be written even when the pipe is full.
            let mut msg = ZmqMessage::default();
            msg.init_delimiter();
            self.out_pipe.write(&msg, false);
            self.flush();
        }
    }

    //  Set the high water marks.
    pub fn set_hwms(&mut self, inhwm: u32, outhwm: u32) {
        let mut in_ = inhwm + u32::max(self.in_hwm_boost, 0);
        let mut out = outhwm + u32::max(self.out_hwm_boost, 0);

        // if either send or recv side has hwm <= 0 it means infinite so we should set hwms infinite
        if (inhwm <= 0 || self.in_hwm_boost == 0) {
            in_ = 0;
        }

        if (outhwm <= 0 || self.out_hwm_boost == 0) {
            out = 0;
        }

        self.lwm = self.compute_lwm(in_ as i32) as u32;
        self.hwm = out;
    }

    //  Set the boost to high water marks, used by inproc sockets so total hwm are sum of connect and Bind sockets watermarks
    pub fn set_hwms_boost(&mut self, inhwmboost_: i32, outhwmboost_: i32) {
        self.in_hwm_boost = inhwmboost_;
        self.out_hwm_boost = outhwmboost_;
    }

    // send command to peer for notify the change of hwm
    pub fn send_hwms_to_peer(&mut self, inhwm: i32, outhwm: i32) {
        self.send_pipe_hwm(self._peer, inhwm, outhwm);
    }

    //  Returns true if HWM is not reached
    pub fn check_hwm(&mut self) -> bool {
        let full = self.hwm > 0 && self.msgs_written - self.peers_msgs_read >= u64(self.hwm);
        return !full;
    }

    pub fn set_endpoint_pair(&mut self, endpoint_pair: EndpointUriPair) {
        self.endpoint_pair = endpoint_pair;
    }


    // const EndpointUriPair &get_endpoint_pair () const;
    pub fn get_endpoint_pair(&mut self) -> &EndpointUriPair {
        &self.endpoint_pair
    }

    // void send_stats_to_peer (ZmqOwn *socket_base);
    pub fn send_stats_to_peer(&mut self, socket_base: &mut ZmqSocket) {
        // EndpointUriPair *ep = new (std::nothrow) EndpointUriPair (_endpoint_pair);
        let mut ep = EndpointUriPair::default();
        ep = self.endpoint_pair.clone();
        self.send_pipe_peer_stats(self._peer, self.msgs_written - self.peers_msgs_read, socket_base, &mut ep);
    }

    // void send_disconnect_msg ();
    pub fn send_disconnect_msg(&mut self) {
        if (self.disconnect_msg.size() > 0 && self.out_pipe.is_empty() == false) {
            // Rollback any incomplete message in the pipe, and push the disconnect message.
            self.rollback();

            self.out_pipe.write(&self.disconnect_msg, false);
            self.flush();
            self.disconnect_msg.init2();
        }
    }

    //
    //  Type of the underlying lock-free pipe.


    //  Command handlers.
    pub fn process_activate_read(&mut self) {
        if (!self.in_active && (self._state == PipeState::active || self._state == PipeState::waiting_for_delimiter)) {
            self.in_active = true;
            self._sink.read_activated(this);
        }
    }

    // void process_activate_write (u64 msgs_read) ;
    pub fn process_activate_write(&mut self, msgs_read: u64) {
        //  Remember the peer's message sequence number.
        self.peers_msgs_read = msgs_read;

        if (!_out_active && self._state == PipeState::active) {
            self.out_active = true;
            self._sink.write_activated(this);
        }
    }

    // void process_hiccup (pipe: *mut c_void) ;
    pub fn process_hiccup(&mut self, pipe: &mut VecDec<ZmqMessage>) {
        //  Destroy old outpipe. Note that the read end of the pipe was already
        //  migrated to this thread.
        // zmq_assert (_out_pipe);
        self.out_pipe.flush();
        let mut msg = ZmqMessage::default();
        while (self.out_pipe.read(&mut msg)) {
            if (!(self.msg.flags() & ZMQ_MSG_MORE)) {
                self.msgs_written -= 1;
            }
            let rc: i32 = msg.clsose();
            // errno_assert (rc == 0);
        }
        // LIBZMQ_DELETE (_out_pipe);

        //  Plug in the new outpipe.
        // zmq_assert (pipe);
        self.out_pipe = (pipe.clone());
        self.out_active = true;

        //  If appropriate, notify the user about the Hiccup.
        if (self._state == PipeState::active) {
            self._sink.hiccuped(this);
        }
    }


    // void process_pipe_term () ;
    pub fn process_pipe_term(&mut self) {
        // zmq_assert (_state == active || _state == delimiter_received
        //     || _state == term_req_sent1);

        //  This is the simple case of peer-induced termination. If there are no
        //  more pending messages to read, or if the pipe was configured to drop
        //  pending messages, we can move directly to the term_ack_sent state.
        //  Otherwise we'll hang up in waiting_for_delimiter state till all
        //  pending messages are read.
        if (self._state == PeerState::active) {
            if (self.delay) {
                self._state = PeerState::waiting_for_delimiter;
            } else {
                self._state = PeerState::term_ack_sent;
                self.out_pipe.clear();
                self.send_pipe_term_ack(_peer);
            }
        }

        //  Delimiter happened to arrive before the Term command. Now we have the
        //  Term command as well, so we can move straight to term_ack_sent state.
        else if (self._state == PeerState::delimiter_received) {
            self._state = PeerState::term_ack_sent;
            self.out_pipe.clear();
            self.send_pipe_term_ack(self._peer);
        }

        //  This is the case where both ends of the pipe are closed in parallel.
        //  We simply reply to the request by ack and continue waiting for our
        //  Own ack.
        else if (self._state == PeerState::term_req_sent1) {
            self._state = PeerState::term_req_sent2;
            self.out_pipe.clear();
            self.send_pipe_term_ack(self._peer);
        }
    }

    // void process_pipe_term_ack () ;
    pub fn process_pipe_term_ack(&mut self) -> anyhow::Result<()> {
        //  Notify the user that all the references to the pipe should be dropped.
        // zmq_assert (_sink);
        self._sink.pipe_terminated(this);

        //  In term_ack_sent and term_req_sent2 states there's nothing to do.
        //  Simply deallocate the pipe. In term_req_sent1 state we have to ack
        //  the peer before deallocating this side of the pipe.
        //  All the other states are invalid.
        if (self._state == PeerState::term_req_sent1) {
            self.out_pipe.clear();
            self.send_pipe_term_ack(self._peer);
        } else {}
        // zmq_assert (_state == term_ack_sent || _state == term_req_sent2);

        //  We'll deallocate the inbound pipe, the peer will deallocate the outbound
        //  pipe (which is an inbound pipe from its point of view).
        //  First, delete all the unread messages in the pipe. We have to do it by
        //  hand because ZmqMessage doesn't have automatic destructor. Then deallocate
        //  the ypipe itself.

        if (!self.conflate) {
            let mut msg = ZmqMessage::default();
            while (self.in_pipe.read(&msg)) {
                msg.close()?;
                // errno_assert (rc == 0);
            }
        }

        // LIBZMQ_DELETE (_in_pipe);

        //  Deallocate the pipe object
        // delete this;
        Ok(())
    }

    // void process_pipe_hwm (inhwm: i32, outhwm: i32) ;
    pub fn process_pipe_hwm(&mut self, inhwm: i32, outhwm: i32) {
        self.set_hwms(inhwm as u32, outhwm as u32);
    }

    //  Handler for delimiter read from the pipe.
    // void process_delimiter ();
    pub fn process_delimiter(&mut self) {
        // zmq_assert (_state == active || _state == waiting_for_delimiter);

        if (self._state == PeerState::active) {
            self._state = PeerState::delimiter_received;
        } else {
            self.rollback();
            self.out_pipe.clone();
            self.send_pipe_term_ack(self._peer);
            self._state = PeerState::term_ack_sent;
        }
    }

    //  Constructor is private. Pipe can only be created using
    //  pipepair function.
    // ZmqPipe (ZmqObject *parent_, upipe_t *inpipe_, upipe_t *outpipe_, inhwm: i32, outhwm: i32, conflate_: bool);
    // ZmqPipe::ZmqPipe (ZmqObject *parent_,
    //                  upipe_t *inpipe_,
    //                  upipe_t *outpipe_,
    //                  inhwm: i32,
    //                  outhwm: i32,
    //                  conflate_: bool) :
    // ZmqObject (parent_),
    // _in_pipe (inpipe_),
    // _out_pipe (outpipe_),
    // _in_active (true),
    // _out_active (true),
    // _hwm (outhwm),
    // _lwm (compute_lwm (inhwm)),
    // _in_hwm_boost (-1),
    // _out_hwm_boost (-1),
    // _msgs_read (0),
    // _msgs_written (0),
    // _peers_msgs_read (0),
    // _peer (null_mut()),
    // _sink (null_mut()),
    // _state (active),
    // _delay (true),
    // _server_socket_routing_id (0),
    // _conflate (conflate_)
    pub fn new(parent: &mut ZmqObject, inpipe: &mut VecDeque<ZmqMessage>, outpipe: &mut VecDeque<ZmqMessage>, inhwm: i32, outhwm: i32, conflate: bool) -> Self {
        let mut out = Self {
            in_pipe: inpipe.clone(),
            out_pipe: outpipe.clone(),
            in_active: true,
            out_active: true,
            hwm: outhwm as u32,
            lwm: compute_lwm(inhwm),
            in_hwm_boost: -1,
            out_hwm_boost: -1,
            msgs_read: 0,
            msgs_written: 0,
            peers_msgs_read: 0,
            delay: false,
            router_socket_routing_id: vec![],
            server_socket_routing_id: 0,
            conflate: false,
            endpoint_pair: Default::default(),
            disconnect_msg: Default::default(),
        };
        _disconnect_msg.init();
        out
    }


    //  Pipepair uses this function to let us know about
    //  the peer pipe object.
    // void set_peer (ZmqPipe *peer_);
    pub fn set_peer(&mut self, peer: &mut Self) {
        //  Peer can be set once only.
        // zmq_assert (!_peer);
        self._peer = peer_;
    }

    //  Destructor is private. Pipe objects destroy themselves.
    // ~ZmqPipe () ;

    //  Returns true if the message is delimiter; false otherwise.
    // static bool is_delimiter (const ZmqMessage &msg);
    pub fn is_delimiter(&mut self, msg: &ZmqMessage) -> bool {
        return msg.is_delimiter();
    }

    //  Computes appropriate low watermark from the given high watermark.
    // static int compute_lwm (hwm_: i32);
    pub fn compute_lwm(&mut self, hwm_: i32) -> i32 {
        //  Compute the low water mark. Following point should be taken
        //  into consideration:
        //
        //  1. LWM has to be less than HWM.
        //  2. LWM cannot be set to very low value (such as zero) as after filling
        //     the queue it would start to refill only after all the messages are
        //     read from it and thus unnecessarily hold the progress back.
        //  3. LWM cannot be set to very high value (such as HWM-1) as it would
        //     result in lock-step filling of the queue - if a single message is
        //     read from a full queue, writer thread is resumed to write exactly one
        //     message to the queue and go back to sleep immediately. This would
        //     result in low performance.
        //
        //  Given the 3. it would be good to keep HWM and LWM as far apart as
        //  possible to reduce the thread switching overhead to almost zero.
        //  Let's make LWM 1/2 of HWM.
        let result: i32 = (hwm_ + 1) / 2;

        return result;
    }

    pub fn process_pipe_peer_stats(&mut self, queue_count: u64,
                                   socket_base: &mut ZmqOwn,
                                   endpoint_pair: &mut EndpointUriPair) {
        send_pipe_stats_publish(socket_base, queue_count,
                                _msgs_written - _peers_msgs_read, endpoint_pair);
    }

    pub fn set_disconnect_msg(&mut self, disconnect_: &mut Vec<u8>) {
        self.disconnect_msg.close();
        let rc: i32 = self.disconnect_msg.init_buffer(&mut disconnect_[0..], disconnect_.size());
        // errno_assert (rc == 0);
    }

    pub fn send_hiccup_msg(&mut self, hiccup_: &mut Vec<u8>) {
        if (!hiccup_.is_empty() && _out_pipe) {
            let mut msg = ZmqMessage::default();
            let rc: i32 = msg.init_buffer(hiccup_, hiccup_.size());
            // errno_assert (rc == 0);

            _out_pipe.write(msg, false);
            flush();
        }
    }


    pub fn pipe_index(&mut self, pipe: &ZmqPipe, pipes: &[ZmqPipe]) -> i32 {
        let mut index = -1;
        for p in pipes.iter() {
            index += 1;
            if p == pipe {
                return index;
            }
        }
        return -1;
    }

    pub fn pipe_erase(&mut self, pipe: &ZmqPipe, pipes: &[ZmqPipe]) -> bool {
        let pipe_idx = pipe_index(pipe, pipes);
        if pipe_idx == -1 {
            return false;
        }

        pipes[pipe_idx] = ZmqPipe::default();
    }

    pub fn pipe_swap(&mut self, idx1: usize, idx2: usize, pipes: &mut [ZmqPipe]) -> bool {
        if idx1 >= pipes.len() || idx2 >= pipes.len() {
            return false;
        }
        let tmp = pipes[idx1].clone();
        pipes[idx1] = pipes[idx2].clone();
        pipes[idx2] = tmp;
        return true;
    }
}

impl i_pipe_events for ZmqPipe {
    fn read_activated(&mut self, pipe: &mut ZmqPipe) {
        todo!()
    }

    fn write_activated(&mut self, pipe: &mut ZmqPipe) {
        todo!()
    }

    fn hiccuped(&mut self, pipe: &mut ZmqPipe) {
        todo!()
    }

    fn pipe_terminated(&mut self, pipe: &mut ZmqPipe) {
        todo!()
    }
}// end of impl pipe_t

// void send_routing_id (pipe: &mut ZmqPipe, options: &ZmqOptions);

// void send_hello_msg (pipe: &mut ZmqPipe, options: &ZmqOptions);

// int pipepair (ZmqObject *parents_[2],
//                    ZmqPipe *pipes_[2],
//                    const int hwms_[2],
//                    const bool conflate_[2])
// {
//     //   Creates two pipe objects. These objects are connected by two ypipes,
//     //   each to pass messages in one direction.
//
//     typedef Ypipe<ZmqMessage, message_pipe_granularity> upipe_normal_t;
//     typedef YpipeConflate<ZmqMessage> upipe_conflate_t;
//
//     ZmqPipe::upipe_t *upipe1;
//     if (conflate_[0])
//         upipe1 = new (std::nothrow) upipe_conflate_t ();
//     else
//         upipe1 = new (std::nothrow) upipe_normal_t ();
//     alloc_assert (upipe1);
//
//     ZmqPipe::upipe_t *upipe2;
//     if (conflate_[1])
//         upipe2 = new (std::nothrow) upipe_conflate_t ();
//     else
//         upipe2 = new (std::nothrow) upipe_normal_t ();
//     alloc_assert (upipe2);
//
//     pipes_[0] = new (std::nothrow)
//       ZmqPipe (parents_[0], upipe1, upipe2, hwms_[1], hwms_[0], conflate_[0]);
//     alloc_assert (pipes_[0]);
//     pipes_[1] = new (std::nothrow)
//       ZmqPipe (parents_[1], upipe2, upipe1, hwms_[0], hwms_[1], conflate_[1]);
//     alloc_assert (pipes_[1]);
//
//     pipes_[0]->set_peer (pipes_[1]);
//     pipes_[1]->set_peer (pipes_[0]);
//
//     return 0;
// }


pub fn send_hello_msg(pipe: &mut ZmqPipe, options: &ZmqContext) {
    // ZmqMessage hello;
    let mut hello = ZmqMessage::default();
    let rc: i32 = hello.init_buffer(&mut options_.hello_msg[0], options_.hello_msg.size());
    // errno_assert (rc == 0);
    let written = pipe.write(&mut hello);
    // zmq_assert (written);
    pipe.flush();
}


// ZmqPipe::~ZmqPipe ()
// {
//     _disconnect_msg.close ();
// }

// void ZmqPipe::set_peer (ZmqPipe *peer_)
// {
// //  Peer can be set once only.
// zmq_assert (!_peer);
// _peer = peer_;
// }

// void ZmqPipe::set_event_sink (i_pipe_events *sink_)
// {
//     // Sink can be set once only.
//     zmq_assert (!_sink);
//     _sink = sink_;
// }

// void ZmqPipe::set_server_socket_routing_id (
//   u32 server_socket_routing_id_)
// {
//     _server_socket_routing_id = server_socket_routing_id_;
// }

// u32 ZmqPipe::get_server_socket_routing_id () const
// {
//     return _server_socket_routing_id;
// }

// void ZmqPipe::set_router_socket_routing_id (
//   const Blob &router_socket_routing_id_)
// {
//     _router_socket_routing_id.set_deep_copy (router_socket_routing_id_);
// }

// const Blob &ZmqPipe::get_routing_id () const
// {
//     return _router_socket_routing_id;
// }

// bool ZmqPipe::check_read ()
// {
//     if (unlikely (!_in_active))
//         return false;
//     if (unlikely (_state != active && _state != waiting_for_delimiter))
//         return false;
//
//     //  Check if there's an item in the pipe.
//     if (!_in_pipe.check_read ()) {
//         _in_active = false;
//         return false;
//     }
//
//     //  If the next item in the pipe is message delimiter,
//     //  initiate termination process.
//     if (_in_pipe.probe (is_delimiter)) {
//         ZmqMessage msg;
//         const bool ok = _in_pipe.read (&msg);
//         zmq_assert (ok);
//         process_delimiter ();
//         return false;
//     }
//
//     return true;
// }
//
// bool ZmqPipe::read (msg: &mut ZmqMessage)
// {
//     if (unlikely (!_in_active))
//         return false;
//     if (unlikely (_state != active && _state != waiting_for_delimiter))
//         return false;
//
//     while (true) {
//         if (!_in_pipe.read (msg)) {
//             _in_active = false;
//             return false;
//         }
//
//         //  If this is a credential, ignore it and receive next message.
//         if (unlikely (msg.is_credential ())) {
//             let rc: i32 = msg.close ();
//             zmq_assert (rc == 0);
//         } else {
//             break;
//         }
//     }
//
//     //  If delimiter was read, start termination process of the pipe.
//     if (msg.is_delimiter ()) {
//         process_delimiter ();
//         return false;
//     }
//
//     if (!(msg.flags () & ZMQ_MSG_MORE) && !msg.is_routing_id ())
//         _msgs_read+= 1;
//
//     if (_lwm > 0 && _msgs_read % _lwm == 0)
//         send_activate_write (_peer, _msgs_read);
//
//     return true;
// }

// bool ZmqPipe::check_write ()
// {
//     if (unlikely (!_out_active || _state != active))
//         return false;
//
//     const bool full = !check_hwm ();
//
//     if (unlikely (full)) {
//         _out_active = false;
//         return false;
//     }
//
//     return true;
// }

// bool ZmqPipe::write (const msg: &mut ZmqMessage)
// {
//     if (unlikely (!check_write ()))
//         return false;
//
//     const bool more = (msg.flags () & ZMQ_MSG_MORE) != 0;
//     const bool is_routing_id = msg.is_routing_id ();
//     _out_pipe.write (*msg, more);
//     if (!more && !is_routing_id)
//         _msgs_written+= 1;
//
//     return true;
// }

// void ZmqPipe::rollback () const
// {
//     //  Remove incomplete message from the outbound pipe.
//     ZmqMessage msg;
//     if (_out_pipe) {
//         while (_out_pipe.unwrite (&msg)) {
//             zmq_assert (msg.flags () & ZMQ_MSG_MORE);
//             let rc: i32 = msg.close ();
//             errno_assert (rc == 0);
//         }
//     }
// }

// void ZmqPipe::flush ()
// {
//     //  The peer does not exist anymore at this point.
//     if (_state == term_ack_sent){
// return;}
//
//     if (_out_pipe && !_out_pipe.flush ()){
// send_activate_read (_peer);}
// }

// void ZmqPipe::process_activate_read ()
// {
//     if (!_in_active && (_state == active || _state == waiting_for_delimiter)) {
//         _in_active = true;
//         _sink.read_activated (this);
//     }
// }

// void ZmqPipe::process_activate_write (u64 msgs_read)
// {
//     //  Remember the peer's message sequence number.
//     _peers_msgs_read = msgs_read;
//
//     if (!_out_active && _state == active) {
//         _out_active = true;
//         _sink.write_activated (this);
//     }
// }

// void ZmqPipe::process_hiccup (pipe: *mut c_void)
// {
//     //  Destroy old outpipe. Note that the read end of the pipe was already
//     //  migrated to this thread.
//     zmq_assert (_out_pipe);
//     _out_pipe.flush ();
// let mut msg = ZmqMessage::default();
//     while (_out_pipe.read (&msg)) {
//         if (!(msg.flags () & ZMQ_MSG_MORE))
//             _msgs_written -= 1;
//         let rc: i32 = msg.close ();
//         errno_assert (rc == 0);
//     }
//     LIBZMQ_DELETE (_out_pipe);
//
//     //  Plug in the new outpipe.
//     zmq_assert (pipe);
//     _out_pipe = static_cast<upipe_t *> (pipe);
//     _out_active = true;
//
//     //  If appropriate, notify the user about the Hiccup.
//     if (_state == active)
//         _sink.hiccuped (this);
// }

// void ZmqPipe::process_pipe_term ()
// {
//     zmq_assert (_state == active || _state == delimiter_received
//                 || _state == term_req_sent1);
//
//     //  This is the simple case of peer-induced termination. If there are no
//     //  more pending messages to read, or if the pipe was configured to drop
//     //  pending messages, we can move directly to the term_ack_sent state.
//     //  Otherwise we'll hang up in waiting_for_delimiter state till all
//     //  pending messages are read.
//     if (_state == active) {
//         if (_delay)
//             _state = waiting_for_delimiter;
//         else {
//             _state = term_ack_sent;
//             _out_pipe = null_mut();
//             send_pipe_term_ack (_peer);
//         }
//     }
//
//     //  Delimiter happened to arrive before the Term command. Now we have the
//     //  Term command as well, so we can move straight to term_ack_sent state.
//     else if (_state == delimiter_received) {
//         _state = term_ack_sent;
//         _out_pipe = null_mut();
//         send_pipe_term_ack (_peer);
//     }
//
//     //  This is the case where both ends of the pipe are closed in parallel.
//     //  We simply reply to the request by ack and continue waiting for our
//     //  Own ack.
//     else if (_state == term_req_sent1) {
//         _state = term_req_sent2;
//         _out_pipe = null_mut();
//         send_pipe_term_ack (_peer);
//     }
// }

// void ZmqPipe::process_pipe_term_ack ()
// {
//     //  Notify the user that all the references to the pipe should be dropped.
//     zmq_assert (_sink);
//     _sink.pipe_terminated (this);
//
//     //  In term_ack_sent and term_req_sent2 states there's nothing to do.
//     //  Simply deallocate the pipe. In term_req_sent1 state we have to ack
//     //  the peer before deallocating this side of the pipe.
//     //  All the other states are invalid.
//     if (_state == term_req_sent1) {
//         _out_pipe = null_mut();
//         send_pipe_term_ack (_peer);
//     } else
//         zmq_assert (_state == term_ack_sent || _state == term_req_sent2);
//
//     //  We'll deallocate the inbound pipe, the peer will deallocate the outbound
//     //  pipe (which is an inbound pipe from its point of view).
//     //  First, delete all the unread messages in the pipe. We have to do it by
//     //  hand because ZmqMessage doesn't have automatic destructor. Then deallocate
//     //  the ypipe itself.
//
//     if (!_conflate) {
// let mut msg = ZmqMessage::default();
//         while (_in_pipe.read (&msg)) {
//             let rc: i32 = msg.close ();
//             errno_assert (rc == 0);
//         }
//     }
//
//     LIBZMQ_DELETE (_in_pipe);
//
//     //  Deallocate the pipe object
//     delete this;
// }

// void ZmqPipe::process_pipe_hwm (inhwm: i32, outhwm: i32)
// {
//     set_hwms (inhwm, outhwm);
// }

// void ZmqPipe::set_nodelay ()
// {
//     this._delay = false;
// }

// void ZmqPipe::terminate (delay_: bool)
// {
//     //  Overload the value specified at pipe creation.
//     _delay = delay_;
//
//     //  If terminate was already called, we can ignore the duplicate invocation.
//     if (_state == term_req_sent1 || _state == term_req_sent2) {
//         return;
//     }
//     //  If the pipe is in the final phase of async termination, it's going to
//     //  closed anyway. No need to do anything special here.
//     if (_state == term_ack_sent) {
//         return;
//     }
//     //  The simple sync termination case. Ask the peer to terminate and wait
//     //  for the ack.
//     if (_state == active) {
//         send_pipe_term (_peer);
//         _state = term_req_sent1;
//     }
//     //  There are still pending messages available, but the user calls
//     //  'terminate'. We can act as if all the pending messages were read.
//     else if (_state == waiting_for_delimiter && !_delay) {
//         //  Drop any unfinished outbound messages.
//         rollback ();
//         _out_pipe = null_mut();
//         send_pipe_term_ack (_peer);
//         _state = term_ack_sent;
//     }
//     //  If there are pending messages still available, do nothing.
//     else if (_state == waiting_for_delimiter) {
//     }
//     //  We've already got delimiter, but not Term command yet. We can ignore
//     //  the delimiter and ack synchronously terminate as if we were in
//     //  active state.
//     else if (_state == delimiter_received) {
//         send_pipe_term (_peer);
//         _state = term_req_sent1;
//     }
//     //  There are no other states.
//     else {
//         zmq_assert (false);
//     }
//
//     //  Stop outbound flow of messages.
//     _out_active = false;
//
//     if (_out_pipe) {
//         //  Drop any unfinished outbound messages.
//         rollback ();
//
//         //  Write the delimiter into the pipe. Note that watermarks are not
//         //  checked; thus the delimiter can be written even when the pipe is full.
// let mut msg = ZmqMessage::default();
//         msg.init_delimiter ();
//         _out_pipe.write (msg, false);
//         flush ();
//     }
// }

// bool ZmqPipe::is_delimiter (const ZmqMessage &msg)
// {
//     return msg.is_delimiter ();
// }

// int ZmqPipe::compute_lwm (hwm_: i32)
// {
//     //  Compute the low water mark. Following point should be taken
//     //  into consideration:
//     //
//     //  1. LWM has to be less than HWM.
//     //  2. LWM cannot be set to very low value (such as zero) as after filling
//     //     the queue it would start to refill only after all the messages are
//     //     read from it and thus unnecessarily hold the progress back.
//     //  3. LWM cannot be set to very high value (such as HWM-1) as it would
//     //     result in lock-step filling of the queue - if a single message is
//     //     read from a full queue, writer thread is resumed to write exactly one
//     //     message to the queue and go back to sleep immediately. This would
//     //     result in low performance.
//     //
//     //  Given the 3. it would be good to keep HWM and LWM as far apart as
//     //  possible to reduce the thread switching overhead to almost zero.
//     //  Let's make LWM 1/2 of HWM.
//     let result: i32 = (hwm_ + 1) / 2;
//
//     return result;
// }

// void ZmqPipe::process_delimiter ()
// {
//     zmq_assert (_state == active || _state == waiting_for_delimiter);
//
//     if (_state == active)
//         _state = delimiter_received;
//     else {
//         rollback ();
//         _out_pipe = null_mut();
//         send_pipe_term_ack (_peer);
//         _state = term_ack_sent;
//     }
// }

// void ZmqPipe::Hiccup ()
// {
//     //  If termination is already under way do nothing.
//     if (_state != active)
//         return;
//
//     //  We'll drop the pointer to the inpipe. From now on, the peer is
//     //  responsible for deallocating it.
//
//     //  Create new inpipe.
//     _in_pipe =
//       _conflate
//         ? static_cast<upipe_t *> (new (std::nothrow) YpipeConflate<ZmqMessage> ())
//         : new (std::nothrow) Ypipe<ZmqMessage, message_pipe_granularity> ();
//
//     alloc_assert (_in_pipe);
//     _in_active = true;
//
//     //  Notify the peer about the Hiccup.
//     send_hiccup (_peer, _in_pipe);
// }

// void ZmqPipe::set_hwms (inhwm: i32, outhwm: i32)
// {
//     int in = inhwm + std::max (_in_hwm_boost, 0);
//     int out = outhwm + std::max (_out_hwm_boost, 0);
//
//     // if either send or recv side has hwm <= 0 it means infinite so we should set hwms infinite
//     if (inhwm <= 0 || _in_hwm_boost == 0)
//         in = 0;
//
//     if (outhwm <= 0 || _out_hwm_boost == 0)
//         out = 0;
//
//     _lwm = compute_lwm (in);
//     _hwm = out;
// }

// void ZmqPipe::set_hwms_boost (inhwmboost_: i32, outhwmboost_: i32)
// {
//     _in_hwm_boost = inhwmboost_;
//     _out_hwm_boost = outhwmboost_;
// }

// bool ZmqPipe::check_hwm () const
// {
//     const bool full =
//       _hwm > 0 && _msgs_written - _peers_msgs_read >= u64 (_hwm);
//     return !full;
// }

// void ZmqPipe::send_hwms_to_peer (inhwm: i32, outhwm: i32)
// {
//     send_pipe_hwm (_peer, inhwm, outhwm);
// }

// void ZmqPipe::set_endpoint_pair (EndpointUriPair endpoint_pair)
// {
//     _endpoint_pair = ZMQ_MOVE (endpoint_pair);
// }

// const EndpointUriPair &ZmqPipe::get_endpoint_pair () const
// {
//     return _endpoint_pair;
// }

// void ZmqPipe::send_stats_to_peer (ZmqOwn *socket_base)
// {
//     EndpointUriPair *ep =
//       new (std::nothrow) EndpointUriPair (_endpoint_pair);
//     send_pipe_peer_stats (_peer, _msgs_written - _peers_msgs_read, socket_base,
//                           ep);
// }


// void ZmqPipe::send_disconnect_msg ()
// {
//     if (_disconnect_msg.size () > 0 && _out_pipe) {
//         // Rollback any incomplete message in the pipe, and push the disconnect message.
//         rollback ();
//
//         _out_pipe.write (_disconnect_msg, false);
//         flush ();
//         _disconnect_msg.init ();
//     }
// }



