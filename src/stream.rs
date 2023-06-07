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
// #include "macros.hpp"
// #include "stream.hpp"
// #include "pipe.hpp"
// #include "wire.hpp"
// #include "random.hpp"
// #include "likely.hpp"
// #include "err.hpp"



use std::ptr::null_mut;
use bincode::options;
use libc::{EAGAIN, EHOSTUNREACH, memcpy, pipe};
use crate::context::ZmqContext;
use crate::decoder_allocators::buffer;
use crate::options::set_opt_bool;
use crate::socket::routing_socket_base_t;
use crate::defines::{ZMQ_STREAM, ZMQ_STREAM_NOTIFY};
use crate::fq::ZmqFq;
use crate::message::{ZMQ_MSG_MORE, ZmqMessage};
use crate::metadata::ZmqMetadata;
use crate::out_pipe::ZmqOutPipe;
use crate::pipe::ZmqPipe;
use crate::utils::{copy_bytes, put_u32};

pub struct ZmqStream {
    //  : public routing_socket_base_t
    pub routing_socket_base: routing_socket_base_t,
    // ZmqStream (ZmqContext *parent_, tid: u32, sid_: i32);
    // ~ZmqStream ();
    //  Overrides of functions from ZmqSocketBase.
    // void xattach_pipe (pipe: &mut ZmqPipe,
    //                    subscribe_to_all_: bool,
    //                    locally_initiated_: bool);
    // int xsend (msg: &mut ZmqMessage);
    // int xrecv (msg: &mut ZmqMessage);
    // bool xhas_in ();
    // bool xhas_out ();
    // void xread_activated (pipe: &mut ZmqPipe);
    // void xpipe_terminated (pipe: &mut ZmqPipe);
    // int xsetsockopt (option_: i32, const optval_: &mut [u8], optvallen_: usize);
    //  Generate peer's id and update lookup map
    // void identify_peer (pipe: &mut ZmqPipe, locally_initiated_: bool);
    //  Fair queueing object for inbound pipes.
    // ZmqFq fair_queue;
    pub fair_queue: ZmqFq,
    //  True iff there is a message held in the pre-fetch buffer.
    pub _prefetched: bool,
    //  If true, the receiver got the message part with
    //  the peer's identity.
    pub _routing_id_sent: bool,
    //  Holds the prefetched identity.
    // ZmqMessage _prefetched_routing_id;
    pub _prefetched_routing_id: ZmqMessage,
    //  Holds the prefetched message.
    // ZmqMessage _prefetched_msg;
    pub _prefetched_msg: ZmqMessage,
    //  The pipe we are currently writing to.
    // ZmqPipe *_current_out;
    pub _current_out: ZmqPipe,
    //  If true, more outgoing message parts are expected.
    pub _more_out: bool,
    //  Routing IDs are generated. It's a simple increment and wrap-over
    //  algorithm. This value is the next ID to use (if not used already).
    // u32 _next_integral_routing_id;
    pub _next_integral_routing_id: u32,
    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqStream)
}

impl ZmqStream {
    pub fn new(parent: &mut ZmqContext, tid: u32, sid_: i32) -> Self

    {
        // routing_socket_base_t (parent_, tid, sid_),
        //         _prefetched (false),
        //         _routing_id_sent (false),
        //         _current_out (null_mut()),
        //         _more_out (false),
        //         _next_integral_routing_id (generate_random ())
        options.type_ = ZMQ_STREAM;
        options.raw_socket = true;

        _prefetched_routing_id.init();
        _prefetched_msg.init();
    }

    pub fn xattach_pipe(&mut self, pipe: &mut ZmqPipe,
                        subscribe_to_all_: bool,
                        locally_initiated_: bool) {
        // LIBZMQ_UNUSED (subscribe_to_all_);

        // zmq_assert (pipe);

        identify_peer(pipe, locally_initiated_);
        fair_queue.attach(pipe);
    }

    pub fn xpipe_terminated(&mut self, pipe: &mut ZmqPipe) {
        erase_out_pipe(pipe);
        fair_queue.pipe_terminated(pipe);
        // TODO router_t calls pipe_->rollback() here; should this be Done here as
        // well? then xpipe_terminated could be pulled up to routing_socket_base_t
        if (pipe == _current_out) {
            _current_out = null_mut();
        }
    }

    pub fn xread_activated(&mut self, pipe: &mut ZmqPipe) {
        fair_queue.activated(pipe);
    }

    pub fn xsend(&mut self, msg: &mut ZmqMessage) -> i32 {
        //  If this is the first part of the message it's the ID of the
        //  peer to send the message to.
        if (!_more_out) {
            // zmq_assert (!_current_out);

            //  If we have malformed message (prefix with no subsequent message)
            //  then just silently ignore it.
            //  TODO: The connections should be killed instead.
            if (msg.flags() & ZMQ_MSG_MORE) {
                //  Find the pipe associated with the routing id stored in the prefix.
                //  If there's no such pipe return an error

                ZmqOutPipe * out_pipe = lookup_out_pipe(
                    Blob((msg.data()),
                         msg.size(), ReferenceTag()));

                if (out_pipe) {
                    _current_out = out_pipe.pipe;
                    if (!_current_out.check_write()) {
                        out_pipe.active = false;
                        _current_out = null_mut();
                        errno = EAGAIN;
                        return -1;
                    }
                } else {
                    errno = EHOSTUNREACH;
                    return -1;
                }
            }

            //  Expect one more message frame.
            _more_out = true;

            let rc = msg.close();
            // errno_assert (rc == 0);
            msg.init2();
            // errno_assert (rc == 0);
            return 0;
        }

        //  Ignore the MORE flag
        msg.reset_flags(ZMQ_MSG_MORE);

        //  This is the last part of the message.
        _more_out = false;

        //  Push the message into the pipe. If there's no out pipe, just drop it.
        if (_current_out) {
            // Close the remote connection if user has asked to do so
            // by sending zero length message.
            // Pending messages in the pipe will be dropped (on receiving Term- ack)
            if (msg.size() == 0) {
                _current_out.terminate(false);
                msg.close();
                // errno_assert (rc == 0);
                msg.init2();
                // errno_assert (rc == 0);
                _current_out = null_mut();
                return 0;
            }
            let ok = _current_out.write(msg);
            if ((ok)) {
                _current_out.flush();
            }
            _current_out = null_mut();
        } else {
            msg.close();
            // errno_assert (rc == 0);
        }

        //  Detach the message from the data buffer.
        msg.init2();
        // errno_assert (rc == 0);

        return 0;
    }


    pub fn xsetsockopt(&mut self, opt_kind: i32,
                       optval_: &[u8]) -> anyhow::Result<()> {
        match (opt_kind) {
            ZMQ_STREAM_NOTIFY => {
                // return do_setsockopt_int_as_bool_strict (optval_, optvallen_,
                //                                          &options.raw_notify);
                return set_opt_bool(optval_, &mut options.raw_notify);
            }

            // _ =>
            _ => {
                return self.xsetsockopt(opt_kind, optval_);
            }
        }
    }

    pub fn xrecv(msg: &mut ZmqMessage) -> i32 {
        if (_prefetched) {
            if (!_routing_id_sent) {
                // let rc: i32 = msg.move (_prefetched_routing_id);
                // errno_assert (rc == 0);
                _routing_id_sent = true;
            } else {
                // let rc: i32 = msg.move (_prefetched_msg);
                // errno_assert (rc == 0);
                _prefetched = false;
            }
            return 0;
        }

        ZmqPipe * pipe = null_mut();
        let rc = fair_queue.recvpipe(&_prefetched_msg, &pipe);
        if (rc != 0) {
            return -1;
        }

        // zmq_assert (pipe != null_mut());
        // zmq_assert ((_prefetched_msg.flags () & ZMQ_MSG_MORE) == 0);

        //  We have received a frame with TCP data.
        //  Rather than sending this frame, we keep it in prefetched
        //  buffer and send a frame with peer's ID.
        let routing_id = pipe.get_routing_id();
        rc = msg.close();
        // errno_assert (rc == 0);
        rc = msg.init_size(routing_id.size());
        // errno_assert (rc == 0);

        // forward metadata (if any)
        let metadata = _prefetched_msg.metadata();
        if (metadata) {
            msg.set_metadata(metadata);
        }

        copy_bytes(msg.data_mut(), 0, routing_id.data(), 0, routing_id.size());
        msg.set_flags(ZMQ_MSG_MORE);

        _prefetched = true;
        _routing_id_sent = true;

        return 0;
    }

    pub fn xhas_in(&mut self) -> bool {
        //  We may already have a message pre-fetched.
        if (_prefetched) {
            return true;
        }

        //  Try to read the next message.
        //  The message, if read, is kept in the pre-fetch buffer.
        ZmqPipe * pipe = null_mut();
        let rc = fair_queue.recvpipe(&_prefetched_msg, &pipe);
        if (rc != 0) {
            return false;
        }

        // zmq_assert (pipe != null_mut());
        // zmq_assert ((_prefetched_msg.flags () & ZMQ_MSG_MORE) == 0);

        let routing_id = pipe.get_routing_id();
        rc = _prefetched_routing_id.init_size(routing_id.size());
        // errno_assert (rc == 0);

        // forward metadata (if any)
        let metadata = _prefetched_msg.metadata();
        if (metadata) {
            _prefetched_routing_id.set_metadata(metadata);
        }

        copy_bytes(_prefetched_routing_id.data_mut(), 0, routing_id.data(),
                   0, routing_id.size());
        _prefetched_routing_id.set_flags(ZMQ_MSG_MORE);

        _prefetched = true;
        _routing_id_sent = false;

        return true;
    }

    pub fn xhas_out(&mut self) -> bool {
        //  In theory, STREAM socket is always ready for writing. Whether actual
        //  attempt to write succeeds depends on which pipe the message is going
        //  to be routed to.
        return true;
    }

    pub fn identify_peer(&mut self, pipe: &mut ZmqPipe, locally_initiated_: bool) {
        //  Always assign routing id for raw-socket
        let mut buffer: [u8; 5] = [0; 5];
        buffer[0] = 0;
        let mut routing_id: Vec<u8> = vec![];
        if (locally_initiated_ && connect_routing_id_is_set()) {
            let connect_routing_id = extract_connect_routing_id();
            routing_id.set(
                (connect_routing_id),
                connect_routing_id.length());
            //  Not allowed to duplicate an existing rid
            // zmq_assert (!has_out_pipe (routing_id));
        } else {
            _next_integral_routing_id += 1;
            put_u32(&mut buffer[1..], 0, _next_integral_routing_id);
            routing_id.set(buffer, 5);
            copy_bytes(options.routing_id, 0, routing_id.data(), 0, routing_id.size());
            options.routing_id_size = (routing_id.size());
        }
        pipe.set_router_socket_routing_id(&mut routing_id);
        add_out_pipe(ZMQ_MOVE(routing_id), pipe);
    }
}
