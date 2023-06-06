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

// #include "tipc_connecter.hpp"

// #if defined ZMQ_HAVE_TIPC

// #include <new>
// #include <string>

// #include "io_thread.hpp"
// #include "platform.hpp"
// #include "random.hpp"
// #include "err.hpp"
// #include "ip.hpp"
// #include "address.hpp"
// #include "tipc_address.hpp"
// #include "session_base.hpp"

use std::mem;
use std::os::raw::c_void;
use libc::{c_char, c_int, close, connect, EHOSTUNREACH, EINPROGRESS, EINTR, EINVAL, ENETDOWN, ENETUNREACH, ETIMEDOUT, getsockopt, open};
use windows::s;
use windows::Win32::Networking::WinSock::{SO_ERROR, SOCK_STREAM, socklen_t, SOL_SOCKET};
use crate::address::{ZmqAddress, get_socket_name};
use crate::address::SocketEnd::SocketEndLocal;
use crate::address_family::AF_TIPC;
use crate::fd::ZmqFileDesc;
use crate::thread_context::ZmqThreadContext;
use crate::ip::{open_socket, unblock_socket};
use crate::ops::zmq_errno;
use crate::options::ZmqOptions;
use crate::session_base::ZmqSessionBase;
use crate::stream_connecter_base::StreamConnecterBase;
use crate::tipc_address::ZmqTipcAddress;

// #include <unistd.h>
// #include <sys/types.h>
// #include <sys/socket.h>
// #ifdef ZMQ_HAVE_VXWORKS
// #include <sockLib.h>
// #endif
pub struct ZmqTipcConnecter<'a> {
    // : public StreamConnecterBase
    pub stream_connecter_base: StreamConnecterBase<'a>,

    //  If 'delayed_start' is true connecter first waits for a while,
    //  then starts connection process.
    // ZmqTipcConnecter (ZmqIoThread *io_thread_,
    //                   ZmqSessionBase *session_,
    //                   options: &ZmqOptions,
    //                   Address *addr_,
    //                   delayed_start_: bool);

    //
    //  Handlers for I/O events.
    // void out_event () ;

    //  Internal function to start the actual connection establishment.
    // void start_connecting () ;

    //  Get the file descriptor of newly created connection. Returns
    //  retired_fd if the connection was unsuccessful.
    // ZmqFileDesc connect ();

    //  Open IPC connecting socket. Returns -1 in case of error,
    //  0 if connect was successful immediately. Returns -1 with
    //  EAGAIN errno if async connect was launched.
    // int open ();

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqTipcConnecter)
}

impl ZmqTipcConnecter {
    // ZmqTipcConnecter::ZmqTipcConnecter (class ZmqIoThread *io_thread_,
    // pub struct ZmqSessionBase *session_,
    //                                          options: &ZmqOptions,
    //                                          Address *addr_,
    //                                          delayed_start_: bool) :
    //     StreamConnecterBase (
    //       io_thread_, session_, options_, addr_, delayed_start_)
    // {
    //     // zmq_assert (_addr.protocol == "tipc");
    // }
    pub fn new(io_thread: &mut ZmqThreadContext, session: &mut ZmqSessionBase, options: &mut ZmqOptions, addr: &mut ZmqAddress<ZmqTipcAddress>, delayed_start: bool) -> Self {
        Self {
            stream_connecter_base: StreamConnecterBase {
                own: Default::default(),
                // io_object:io_thread,
                io_object: Default::default(),
                _addr: addr,
                _s: 0,
                _handle: None,
                _endpoint: "".to_string(),
                _socket: &Default::default(),
                start_connecting: None,
                _delayed_start: delayed_start,
                _reconnect_timer_started: false,
                _current_reconnect_ivl: 0,

                // options: options,


                _session: session,
            }
        }
    }

    pub fn out_event(&mut self) {
        // TODO
        // let mut fd: ZmqFileDesc = unsafe { connect() };
        rm_handle();

        //  Handle the error condition by attempt to reconnect.
        if (fd == retired_fd) {
            unsafe { close(fd as c_int); }
            add_reconnect_timer();
            return;
        }

        create_engine(fd, get_socket_name(fd, SocketEndLocal));
    }

    pub fn start_connecting(&mut self) {
        //  Open the connecting socket.
        // unsafe { rc = open(); }

        //  Connect may succeed in synchronous manner.
        if (rc == 0) {
            _handle = add_fd(_s);
            out_event();
        }

        //  Connection establishment may be delayed. Poll for its completion.
        else if rc == -1 && errno == EINPROGRESS {
            _handle = add_fd(_s);
            set_pollout(_handle);
            self._socket.event_connect_delayed(
                make_unconnected_connect_endpoint_pair(_endpoint), zmq_errno());
        }

        //  Handle any other error condition by eventual reconnect.
        else {
            if (_s != retired_fd) {
                unsafe { close(_s); }
            }
            add_reconnect_timer();
        }
    }

    pub fn open(&mut self) -> i32 {
        // zmq_assert (_s == retired_fd);

        // Cannot connect to random tipc addresses
        if (_addr.resolved.tipc_addr.is_random()) {
            errno = EINVAL;
            return -1;
        }
        //  Create the socket.
        _s = open_socket(AF_TIPC as i32, SOCK_STREAM as i32, 0);
        if (_s == retired_fd) {
            return -1;
        }

        //  Set the non-blocking flag.
        unblock_socket(_s);
        //  Connect to the remote peer.
        // #ifdef ZMQ_HAVE_VXWORKS
        //     let mut rc = connect (s, addr.resolved.tipc_addr.addr (),
        //                         addr.resolved.tipc_addr.addrlen ());
        // #else
        let mut rc = unsafe {
            connect(_s, _addr.resolved.tipc_addr.addr(),
                    _addr.resolved.tipc_addr.addrlen())
        };
        // #endif
        //  Connect was successful immediately.
        if (rc == 0) {
            return 0;
        }

        //  Translate other error codes indicating asynchronous connect has been
        //  launched to a uniform EINPROGRESS.
        if (rc == -1 && errno == EINTR) {
            errno = EINPROGRESS;
            return -1;
        }
        //  Forward the error.
        return -1;
    }

    pub fn connect(&mut self) -> ZmqFileDesc {
        //  Following code should handle both Berkeley-derived socket
        //  implementations and Solaris.
        let mut err = 0;
        // #ifdef ZMQ_HAVE_VXWORKS
        let mut len = 4;
        // #else
        let mut len = 4;
        // #endif
        let mut rc = unsafe {
            getsockopt(_s, SOL_SOCKET, SO_ERROR,
                       (&err as *mut c_char), &mut len)
        };
        if (rc == -1) {
            err = errno;
        }
        if (err != 0) {
            //  Assert if the error was caused by 0MQ bug.
            //  Networking problems are OK. No need to assert.
            errno = err;
            // errno_assert (errno == ECONNREFUSED || errno == ECONNRESET
            // || errno == ETIMEDOUT || errno == EHOSTUNREACH || errno == ENETUNREACH || errno == ENETDOWN);

            return retired_fd;
        }
        let mut result: ZmqFileDesc = _s;
        _s = retired_fd;
        return result;
    }
}

// #endif
