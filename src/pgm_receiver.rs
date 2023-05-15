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

// #if defined ZMQ_HAVE_OPENPGM

// #include <new>

// #include "pgm_receiver.hpp"
// #include "session_base.hpp"
// #include "v1_decoder.hpp"
// #include "stdint.hpp"
// #include "wire.hpp"
// #include "err.hpp"

use std::collections::HashMap;
use std::ffi::c_void;
use std::intrinsics::offset;
use std::mem;
use std::ptr::null_mut;

use bincode::options;
use libc::{size_t, ssize_t, uint16_t, EAGAIN, EBUSY, ENOMEM};

use crate::defines::ZmqHandle;
use crate::endpoint::{EndpointType, EndpointUriPair};
use crate::engine_interface::ZmqEngineInterface;
use crate::fd::ZmqFileDesc;
use crate::io_object::ZmqIoObject;
use crate::io_thread::ZmqIoThread;
use crate::message::ZmqMessage;
use crate::options::ZmqOptions;
use crate::pgm_socket;
use crate::session_base::ZmqSessionBase;
use crate::utils::{decoder, get_u16};
use crate::v1_decoder::ZmqV1Decoder;

//  RX timeout timer ID.
// enum
// {
//     rx_timer_id = 0xa1
// };
pub const rx_timer_id: u8 = 0xa1;

pub const _empty_endpoint: EndpointUriPair =
    EndpointUriPair::new("", "", EndpointType::endpoint_type_none);

#[derive(Default, Debug, Clone)]
pub struct ZmqPeerInfo {
    joined: bool,
    // ZmqV1Decoder *decoder;
    decoder: Option<ZmqV1Decoder>,
}

// #[derive(Default,Debug,Clone)]
// pub struct tsi_comp
// {
//
// }
//
// impl tsi_comp {
//     bool operator() (const pgm_tsi_t &ltsi, const pgm_tsi_t &rtsi) const
//     {
//         u32 ll[2], rl[2];
//         memcpy (ll, &ltsi, mem::size_of::<ll>());
//         memcpy (rl, &rtsi, mem::size_of::<rl>());
//         return (ll[0] < rl[0]) || (ll[0] == rl[0] && ll[1] < rl[1]);
//     }
// }

#[derive(Default, Debug, Clone)]
pub struct ZmqPgmReceiver {
    // : public ZmqIoObject, public ZmqEngineInterface
    pub zmq_io_object: ZmqIoObject,
    pub zmq_options: ZmqOptions,
    //  RX timer is running.
    pub has_rx_timer: bool,
    //  If joined is true we are already getting messages from the peer.
    //  It it's false, we are getting data but still we haven't seen
    //  beginning of a message.
    // typedef std::map<pgm_tsi_t, ZmqPeerInfo, tsi_comp> peers_t;
    // peers_t peers;
    pub peers: HashMap<pgm_tsi_t, ZmqPeerInfo>,
    //  PGM socket.
    pub PgmSocket: pgm_socket,
    //  Socket options.
    pub options: ZmqOptions,
    //  Associated session.
    pub session: ZmqSessionBase,
    pub active_tsi: Option<pgm_tsi_t>,
    //  Number of bytes not consumed by the decoder due to pipe overflow.
    pub insize: usize,
    //  Pointer to data still waiting to be processed by the decoder.
    // const unsigned char *inpos;
    pub inpos: *const u8,
    //  Poll handle associated with PGM socket.
    // handle_t socket_handle;
    pub socket_handle: Option<ZmqHandle>,
    //  Poll handle associated with engine PGM waiting pipe.
    // handle_t pipe_handle;
    pub pipe_handle: Option<ZmqHandle>,
    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqPgmReceiver)
}

impl ZmqEngineInterface for ZmqPgmReceiver {
    fn has_handshake_state(&self) -> bool {
        todo!()
    }

    fn plug(&mut self, io_thread: &mut ZmqIoThread, session: &mut ZmqSessionBase) {
        todo!()
    }

    fn terminate(&mut self) {
        todo!()
    }

    fn restart_input(&mut self) -> bool {
        todo!()
    }

    fn restart_output(&mut self) {
        todo!()
    }

    fn zap_msg_available(&mut self) {
        todo!()
    }

    fn get_endpoint(&mut self) -> &EndpointUriPair {
        todo!()
    }
}

impl ZmqPgmReceiver {
    // ZmqPgmReceiver (parent_: &mut ZmqIoThread, options: &ZmqOptions);
    pub fn new(parent: &mut ZmqIoThread, options: &ZmqOptions) -> Self {
        // : ZmqIoObject (parent_),
        //     has_rx_timer (false),
        //     pgm_socket (true, options_),
        //     options (options_),
        //     session (null_mut()),
        //     active_tsi (null_mut()),
        //     insize (0)
        Self {
            zmq_io_object: ZmqIoObject::new(Some(parent.clone())),
            zmq_options: options.clone(),
            has_rx_timer: false,
            peers: HashMap::new(),
            PgmSocket: pgm_socket::new(true, options),
            options: options.clone(),
            session: ZmqSessionBase::default(),
            active_tsi: None,
            insize: 0,
            inpos: null_mut(),
            socket_handle: None,
            pipe_handle: None,
        }
    }
    // ~ZmqPgmReceiver ();
    // ZmqPgmReceiver::~ZmqPgmReceiver ()
    // {
    // //  Destructor should not be called before unplug.
    // // zmq_assert (peers.empty ());
    // }

    // int init (udp_encapsulation_: bool, network_: &str);
    pub fn init(&mut self, udp_encapsulation_: bool, network_: &str) -> i32 {
        return self.pgm_socket.init(udp_encapsulation_, network_);
    }

    //  ZmqIEngine interface implementation.
    // bool has_handshake_stage () { return false; };

    //  Decode received data (inpos, insize) and forward decoded
    //  messages to the session.
    // int process_input (ZmqV1Decoder *decoder);

    //  PGM is not able to move subscriptions upstream. Thus, drop all
    //  the pending subscriptions.
    // void drop_subscriptions ();

    // void plug (ZmqIoThread *io_thread_, ZmqSessionBase *session_);
    pub fn plug(&mut self, io_thread: &mut ZmqIoThread, session_: &mut ZmqSessionBase) {
        // LIBZMQ_UNUSED (io_thread_);
        //  Retrieve PGM fds and start polling.
        let socket_fd = retired_fd;
        let waiting_pipe_fd = retired_fd;
        pgm_socket.get_receiver_fds(&socket_fd, &waiting_pipe_fd);
        socket_handle = add_fd(socket_fd);
        pipe_handle = add_fd(waiting_pipe_fd);
        set_pollin(pipe_handle);
        set_pollin(socket_handle);
        session = session_;

        //  If there are any subscriptions already queued in the session, drop them.
        self.drop_subscriptions();
    }

    // void terminate ();
    pub fn terminate(&mut self) {
        self.unplug();
        // delete this;
    }

    // bool restart_input ();

    // void restart_output ();
    pub fn restart_output(&mut self) {
        self.drop_subscriptions();
    }

    // void zap_msg_available () {}

    // const EndpointUriPair &get_endpoint () const;

    //  i_poll_events interface implementation.

    // void in_event ();

    // void timer_event (token: i32);

    //
    //  Unplug the engine from the session.
    // void unplug ();
    pub fn unplug(&mut self) {
        //  Delete decoders.
        // for (peers_t::iterator it = peers.begin (), end = peers.end (); it != end; += 1it)
        for peer in self.peers {
            if peer.second.decoder != null_mut() {
                LIBZMQ_DELETE(it.second.decoder);
            }
        }
        self.peers.clear();
        self.active_tsi = None;

        if (has_rx_timer) {
            cancel_timer(rx_timer_id);
            has_rx_timer = false;
        }

        rm_fd(socket_handle);
        rm_fd(pipe_handle);

        session = null_mut();
    }

    pub fn restart_input(&mut self) -> bool {
        // zmq_assert (session != null_mut());
        // zmq_assert (active_tsi != null_mut());

        let it = peers.find(*active_tsi);
        // zmq_assert (it != peers.end ());
        // zmq_assert (it.second.joined);

        //  Push the pending message into the session.
        let rc = session.push_msg(it.second.decoder.msg());
        // errno_assert (rc == 0);

        if (insize > 0) {
            rc = process_input(it.second.decoder);
            if (rc == -1) {
                //  HWM reached; we will try later.
                if (errno == EAGAIN) {
                    session.flush();
                    return true;
                }
                //  Data error. Delete message decoder, mark the
                //  peer as not joined and drop remaining data.
                it.second.joined = false;
                LIBZMQ_DELETE(it.second.decoder);
                insize = 0;
            }
        }

        //  Resume polling.
        set_pollin(pipe_handle);
        set_pollin(socket_handle);

        active_tsi = null_mut();
        in_event();

        return true;
    }

    pub fn get_endpoint(&mut self) -> EndpointUriPair {
        return self._empty_endpoint;
    }

    pub fn in_event(&mut self) {
        // If active_tsi is not null, there is a pending restart_input.
        // Keep the internal state as is so that restart_input would process the right data
        if (active_tsi) {
            return;
        }

        // Read data from the underlying pgm_socket. const pgm_tsi_t * tsi = null_mut();

        if (has_rx_timer) {
            cancel_timer(rx_timer_id);
            has_rx_timer = false;
        }

        //  TODO: This loop can effectively block other engines in the same I/O
        //  thread in the case of high load.
        loop {
            //  Get new batch of data.
            //  Note the workaround made not to break strict-aliasing rules.
            let mut insize = 0;
            let mut tmp: *mut c_void = null_mut();
            let mut received = pgm_socket.receive(&tmp, &tsi);

            //  No data to process. This may happen if the packet received is
            //  neither ODATA nor ODATA.
            if received == 0 {
                if errno == ENOMEM || errno == EBUSY {
                    let timeout = pgm_socket.get_rx_timeout();
                    add_timer(timeout, rx_timer_id);
                    has_rx_timer = true;
                }
                break;
            }

            //  Find the peer based on its TSI.
            let it = peers.find(*tsi);

            //  Data loss. Delete decoder and mark the peer as disjoint.
            if received == -1 {
                if it != peers.end() {
                    it.second.joined = false;
                    if it.second.decoder != null_mut() {
                        LIBZMQ_DELETE(it.second.decoder);
                    }
                }
                break;
            }

            //  New peer. Add it to the list of know but unjoint peers.
            if (it == peers.end()) {
                // let peer_info = ZmqPeerInfo{false, null_mut()};
                let peer_info = ZmqPeerInfo {
                    joined: false,
                    decoder: None,
                };
                it = peers.ZMQ_MAP_INSERT_OR_EMPLACE(*tsi, peer_info).first;
            }

            insize = (received);
            inpos = tmp;

            //  Read the offset of the fist message in the current packet.
            // zmq_assert (insize >= mem::size_of::<uint16_t>());
            // let offset = get_u16 (self.inpos);
            inpos += mem::size_of::<u16>();
            insize -= mem::size_of::<u16>();

            //  Join the stream if needed.
            if (!it.second.joined) {
                //  There is no beginning of the message in current packet.
                //  Ignore the data.
                if (offset == 0xffff) {
                    continue;
                }

                // zmq_assert (offset <= insize);
                // zmq_assert (it.second.decoder == null_mut());

                //  We have to move data to the beginning of the first message.
                inpos += offset;
                insize -= offset;

                //  Mark the stream as joined.
                it.second.joined = true;

                //  Create and connect decoder for the peer.
                it.second.decoder = ZmqV1Decoder(0, options.maxmsgsize);
                // alloc_assert (it.second.decoder);
            }

            let rc = process_input(it.second.decoder);
            if (rc == -1) {
                if (errno == EAGAIN) {
                    active_tsi = tsi;

                    //  Stop polling.
                    reset_pollin(pipe_handle);
                    reset_pollin(socket_handle);

                    break;
                }

                it.second.joined = false;
                LIBZMQ_DELETE(it.second.decoder);
                insize = 0;
            }
        }

        //  Flush any messages decoder may have produced.
        session.flush();
    }

    pub fn process_input(&mut self, decoder: &mut ZmqV1Decoder) -> i32 {
        // zmq_assert (session != null_mut());

        while insize > 0 {
            let mut n = 0usize;
            let rc = decoder.decode(inpos, insize, n);
            if rc == -1 {
                return -1;
            }
            inpos += n;
            insize -= n;
            if (rc == 0) {
                break;
            }
            rc = self.session.push_msg(decoder.msg());
            if (rc == -1) {
                // errno_assert (errno == EAGAIN); return - 1;
            }
        }
        return 0;
    }

    pub fn timer_event(&mut self, token: i32) {
        // zmq_assert (token == rx_timer_id);

        //  Timer cancels on return by poller_base.
        self.has_rx_timer = false;
        self.in_event();
    }

    pub fn drop_subscriptions(&mut self) {
        let mut msg = ZmqMessage::default();
        msg.init2();
        while self.session.pull_msg(&mut msg) == 0 {
            msg.close();
        }
    }
}
// #endif
