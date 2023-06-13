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
// #include "stream_listener_base.hpp"
// #include "session_base.hpp"
// #include "socket_base.hpp"
// #include "zmtp_engine.hpp"
// #include "raw_engine.hpp"

use crate::address::get_socket_name;
use crate::address::SocketEnd::{SocketEndLocal, SocketEndRemote};
use crate::context::{choose_io_thread, ZmqContext};
use crate::defines::{retired_fd, ZmqHandle};
use crate::endpoint::EndpointType::endpoint_type_bind;
use crate::endpoint::{make_unconnected_bind_endpoint_pair, EndpointUriPair};
use crate::fd::ZmqFileDesc;
use crate::io_object::ZmqIoObject;
use crate::object::ZmqObject;

use crate::own::ZmqOwn;
use crate::raw_engine::RawEngine;
use crate::session_base::ZmqSessionBase;
use crate::socket::ZmqSocket;
use crate::thread_context::ZmqThreadContext;
use crate::zmtp_engine::ZmqZmtpEngine;
use bincode::options;
use libc::close;
use std::ptr::null_mut;
use windows::Win32::Networking::WinSock::{closesocket, SOCKET_ERROR};

// #ifndef ZMQ_HAVE_WINDOWS
// #include <unistd.h>
// #else
// #include <winsock2.h>
// #endif
pub struct ZmqStreamListenerBase {
    // : public ZmqOwn, public ZmqIoObject
    pub own: ZmqOwn,
    pub io_object: ZmqIoObject,
    // ZmqStreamListenerBase (ZmqIoThread *io_thread_,
    //                         socket: *mut ZmqSocketBase,
    //                         options: &ZmqOptions);
    // ~ZmqStreamListenerBase () ;
    // Get the bound address for use with wildcards
    // int get_local_address (std::string &addr_) const;
    // virtual std::string get_socket_name (fd: ZmqFileDesc,
    //                                      SocketEnd socket_end_) const = 0;
    //  Handlers for incoming commands.
    // void process_plug () ;
    // void process_term (linger: i32) ;
    //  Close the listening socket.
    // virtual int close ();
    // virtual void create_engine (ZmqFileDesc fd);
    //  Underlying socket.
    // ZmqFileDesc _s;
    pub _s: ZmqFileDesc,
    //  Handle corresponding to the listening socket.
    // let mut _handle: ZmqHandle;
    pub _handle: Option<ZmqHandle>,
    //  Socket the listener belongs to.
    // ZmqSocketBase *_socket;
    pub _socket: ZmqSocket,
    // String representation of endpoint to Bind to
    pub _endpoint: String,
    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqStreamListenerBase)
}

impl ZmqStreamListenerBase {
    pub fn new(
        io_thread: &mut ZmqThreadContext,
        socket: &mut ZmqSocket,
        options: &mut ZmqContext,
    ) -> Self {
        //   ZmqOwn (io_thread_, options_),
        //         ZmqIoObject (io_thread_),
        //         _s (retired_fd),
        //         _handle ( (null_mut())),
        //         self._socket (socket)
        Self {
            own: ZmqOwn::new(options, io_thread.get_ctx_mut(), io_thread.tid),
            io_object: ZmqIoObject::new(io_thread_),
            _s: retired_fd as ZmqFileDesc,
            _handle: None,
            _socket: socket.clone(),
            _endpoint: String::new(),
        }
    }

    // ZmqStreamListenerBase::~ZmqStreamListenerBase ()
    // {
    //     // zmq_assert (_s == retired_fd);
    //     // zmq_assert (!_handle);
    // }

    pub fn get_local_address(&mut self, addr_: &mut str) -> i32 {
        // *addr_ = get_socket_name (_s, SocketEndLocal).unwrap().as_str();
        return if addr_.is_empty() { -1 } else { 0 };
    }

    pub fn process_plug(&mut self) {
        //  Start polling for incoming connections.
        _handle = add_fd(_s);
        set_pollin(_handle);
    }

    pub fn process_term(&mut self, options: &mut ZmqContext, linger: i32) {
        rm_fd(_handle);
        _handle = (null_mut());
        self.close(options);
        self.own.process_term(linger);
    }

    pub fn close(&mut self, options: &mut ZmqContext) -> i32 {
        // TODO this is identical to stream_connector_base_t::close

        // zmq_assert (_s != retired_fd);
        // #ifdef ZMQ_HAVE_WINDOWS
        //     let rc: i32 = closesocket (_s);
        //     wsa_assert (rc != SOCKET_ERROR);
        // #else
        let rc: i32 = ::close(_s);
        // errno_assert (rc == 0);
        // #endif
        self._socket
            .event_closed(options, &make_unconnected_bind_endpoint_pair(_endpoint), _s);
        _s = retired_fd;

        return 0;
    }

    pub fn create_engine(&mut self, ctx: &mut ZmqContext, fd: ZmqFileDesc) {
        let endpoint_pair = EndpointUriPair::new(
            &get_socket_name(fd, SocketEndLocal).unwrap(),
            &get_socket_name(fd, SocketEndRemote).unwrap(),
            endpoint_type_bind,
        );

        let mut engine = ZmqEngineInterface::default();
        if (ctx.raw_socket) {
            engine = RawEngine(fd, ctx, endpoint_pair);
        } else {
            engine = ZmqZmtpEngine(fd, ctx, endpoint_pair);
        }
        // alloc_assert (engine);

        //  Choose I/O thread to run connecter in. Given that we are already
        //  running in an I/O thread, there must be at least one available.
        let io_thread = self.choose_io_thread(ctx.affinity);
        // zmq_assert (io_thread);

        //  Create and launch a session object.
        let session = ZmqSessionBase::create(
            &mut self.own.ctx,
            io_thread,
            false,
            &mut self._socket,
            ctx,
            None,
        );
        // errno_assert (session);
        session.inc_seqnum();
        launch_child(session);
        send_attach(&session, engine, false);

        self._socket.event_accepted(ctx, &endpoint_pair, fd);
    }
}
