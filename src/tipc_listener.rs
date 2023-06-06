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

// #include "tipc_listener.hpp"

// #if defined ZMQ_HAVE_TIPC

// #include <new>

// #include <string.h>

// #include "tipc_address.hpp"
// #include "io_thread.hpp"
// #include "config.hpp"
// #include "err.hpp"
// #include "ip.hpp"
// #include "socket_base.hpp"
// #include "address.hpp"

use crate::address::SocketEnd::SocketEndLocal;
use crate::address::{get_socket_address, get_socket_name, SocketEnd};
use crate::address_family::AF_TIPC;
use crate::endpoint::make_unconnected_bind_endpoint_pair;
use crate::fd::ZmqFileDesc;
use crate::ip::open_socket;
use crate::mechanism::ZmqMechanismStatus::error;
use crate::ops::zmq_errno;

use crate::socket_base::ZmqSocketBase;
use crate::stream_listener_base::ZmqStreamListenerBase;
use crate::thread_context::ZmqThreadContext;
use crate::tipc_address::ZmqTipcAddress;
use bincode::options;
use libc::{
    accept, bind, close, listen, ECONNABORTED, EINTR, EINVAL, EMFILE, ENFILE, ENOBUFS, EPROTO,
};
use std::mem;
use windows::Win32::Networking::WinSock::{socklen_t, SOCK_STREAM};
use crate::context::ZmqContext;

// #include <unistd.h>
// #include <sys/socket.h>
// #include <fcntl.h>
// #if defined ZMQ_HAVE_VXWORKS
// #include <sockLib.h>
// #include <tipc/tipc.h>
// #else
// #include <linux/tipc.h>
// #endif
pub struct ZmqTipcListener {
    // : public ZmqStreamListenerBase
    pub stream_listener_base: ZmqStreamListenerBase,
    // ZmqTipcListener (ZmqIoThread *io_thread_,
    //                  socket: *mut ZmqSocketBase,
    //                  options: &ZmqOptions);
    //  Set address to listen on.
    // int set_local_address (addr_: &str);
    // std::string get_socket_name (fd: ZmqFileDesc,
    //                              SocketEnd socket_end_) const ;
    //
    //  Handlers for I/O events.
    // void in_event () ;

    //  Accept the new connection. Returns the file descriptor of the
    //  newly created connection. The function may return retired_fd
    //  if the connection was dropped while waiting in the listen backlog.
    // ZmqFileDesc accept ();

    // Address to listen on
    // ZmqTipcAddress address;
    pub address: ZmqTipcAddress,
    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqTipcListener)
}

impl ZmqTipcListener {
    // ZmqTipcListener::ZmqTipcListener (ZmqIoThread *io_thread_,
    //                                        ZmqSocketBase *socket,
    //                                        options: &ZmqOptions) :
    //     ZmqStreamListenerBase (io_thread_, socket, options_)
    // {
    // }
    pub fn new(
        io_thread_: &mut ZmqThreadContext,
        socket: &mut ZmqSocketBase,
        ctx: &mut ZmqContext,
    ) -> Self {
        Self {
            stream_listener_base: ZmqStreamListenerBase {
                own: Default::default(),
                io_object: Default::default(),
                _s: 0,
                _handle: None,
                _socket: socket.clone(),
                _endpoint: "".to_string(),
            },
            address: ZmqTipcAddress {
                _random: false,
                address: (),
            },
        }
    }

    pub fn in_event(&mut self) {
        let mut fd: ZmqFileDesc = self.accept();

        //  If connection was reset by the peer in the meantime, just ignore it.
        //  TODO: Handle specific errors like ENFILE/EMFILE etc.
        if (fd == retired_fd) {
            self._socket
                .event_accept_failed(make_unconnected_bind_endpoint_pair(_endpoint), zmq_errno());
            return;
        }

        //  Create the engine object for this connection.
        create_engine(fd);
    }

    pub fn get_socket_name(&mut self, fd: ZmqFileDesc, socket_end_: SocketEnd) -> String {
        return get_socket_name(fd, socket_end_).unwrap();
    }

    pub fn set_local_address(&mut self, addr_: &str) -> i32 {
        // Convert str to address struct
        let rc = address.resolve(addr_);
        if (rc != 0) {
            return -1;
        }

        // Cannot bind non-random Port Identity
        let a = (address.addr());
        if (!address.is_random() && a.addrtype == TIPC_ADDR_ID) {
            errno = EINVAL;
            return -1;
        }

        //  Create a listening socket.
        _s = open_socket(AF_TIPC as i32, SOCK_STREAM as i32, 0);
        if (_s == retired_fd) {
            return -1;
        }

        // If random Port Identity, update address object to reflect the assigned address
        if (address.is_random()) {
            let ss = sockaddr_storage {};
            let sl = get_socket_address(_s, SocketEndLocal, &mut ss).unwrap();
            if (sl == 0) {
                // goto
                // error;
            }

            self.address = ZmqTipcAddress::new2((&ss), sl as socklen_t);
        }

        self.address.to_string(_endpoint);

        //  Bind the socket to tipc name
        if (address.is_service()) {
            // #ifdef ZMQ_HAVE_VXWORKS
            //         rc = bind (_s,  address.addr (), address.addrlen ());
            // #else
            unsafe {
                rc = bind(_s, address.addr(), address.addrlen());
            }
            // #endif
            if (rc != 0) {
                // goto
                // error;
            }
        }

        //  Listen for incoming connections.
        unsafe {
            rc = listen(_s, options.backlog);
        }
        if (rc != 0) {
            // goto
            // error;
        }

        self._socket
            .event_listening(make_unconnected_bind_endpoint_pair(_endpoint), _s);
        return 0;

        // error:
        //     int err = errno;
        //     close ();
        //     errno = err;
        //     return -1;
    }

    pub fn accept(&mut self) -> ZmqFileDesc {
        //  Accept one connection and deal with different failure modes.
        //  The situation where connection cannot be accepted due to insufficient
        //  resources is considered valid and treated by ignoring the connection.
        let mut ss = sockaddr_storage {};
        // socklen_t ss_len = mem::size_of::<ss>();
        let mut ss_len = mem::size_of_val(&ss);

        // zmq_assert (_s != retired_fd);
        // #ifdef ZMQ_HAVE_VXWORKS
        //      let mut sock: ZmqFileDesc = accept (_s, &ss, &ss_len);
        // #else
        let mut sock: ZmqFileDesc = ::accept(_s, (&ss), &ss_len);
        // #endif
        if (sock == -1) {
            // errno_assert (errno == EAGAIN || errno == EWOULDBLOCK
            //               || errno == ENOBUFS || errno == EINTR
            //               || errno == ECONNABORTED || errno == EPROTO
            //               || errno == EMFILE || errno == ENFILE);
            return retired_fd;
        }
        /*FIXME Accept filters?*/
        return sock;
    }
}

// #endif
