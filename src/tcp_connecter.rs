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
// #include <string>

// #include "macros.hpp"
// #include "tcp_connecter.hpp"
// #include "io_thread.hpp"
// #include "err.hpp"
// #include "ip.hpp"
// #include "tcp.hpp"
// #include "address.hpp"
// #include "tcp_address.hpp"
// #include "session_base.hpp"

// #if !defined ZMQ_HAVE_WINDOWS
// #include <unistd.h>
// #include <sys/types.h>
// #include <sys/socket.h>
// #include <arpa/inet.h>
// #include <netinet/tcp.h>
// #include <netinet/in.h>
// #include <netdb.h>
// #include <fcntl.h>
// #ifdef ZMQ_HAVE_VXWORKS
// #include <sockLib.h>
// #endif
// #ifdef ZMQ_HAVE_OPENVMS
// #include <ioctl.h>
// #endif
// #endif

use crate::address::SocketEnd::SocketEndLocal;
use crate::address::{get_socket_name, ZmqAddress};
use crate::defines::ZMQ_RECONNECT_STOP_CONN_REFUSED;
use crate::err::wsa_error_to_errno;
use crate::defines::ZmqFileDesc;
use crate::ip::{tune_socket, unblock_socket};
use crate::ops::zmq_errno;

use crate::session_base::ZmqSessionBase;
use crate::stream_connecter_base::StreamConnecterBase;
use crate::tcp::{tcp_open_socket, tune_tcp_keepalives, tune_tcp_maxrt, tune_tcp_socket};
use crate::tcp_address::TcpAddress;
use crate::thread_context::ZmqThreadContext;
use bincode::options;
use libc::{
    bind, close, connect, getsockopt, open, setsockopt, ECONNREFUSED, EINPROGRESS, EINTR, ENOBUFS,
    ENOTSOCK,
};
use std::mem;
use std::ptr::null_mut;
use windows::Win32::Networking::WinSock::{
    socklen_t, WSAGetLastError, SOCKET_ERROR, SOL_SOCKET, SO_ERROR, SO_REUSEADDR, WSAEBADF,
    WSAEINPROGRESS, WSAENOBUFS, WSAENOPROTOOPT, WSAENOTSOCK, WSAEWOULDBLOCK, WSA_ERROR,
};
use crate::context::ZmqContext;

// enum
// {
//     connect_timer_id = 2
// };
pub const connect_timer_id: i32 = 2;

// #ifdef __APPLE__
// #include <TargetConditionals.h>
// #endif
#[derive(Default, Debug, Clone)]
pub struct ZmqTcpConnector<'a> {
    //: public StreamConnecterBase
    pub stream_connecter_base: StreamConnecterBase<'a>,
    //  If 'delayed_start' is true connecter first waits for a while,
    //  then starts connection process.
    // ZmqTcpConnector (ZmqIoThread *io_thread_,
    //                  ZmqSessionBase *session_,
    //                  options: &ZmqOptions,
    //                  Address *addr_,
    //                  delayed_start_: bool);
    // ~ZmqTcpConnector ();
    //  ID of the timer used to check the connect timeout, must be different from stream_connecter_base_t::reconnect_timer_id.
    //  Handlers for incoming commands.
    // void process_term (linger: i32);
    //  Handlers for I/O evens.
    // void out_event ();
    // void timer_event (id_: i32);
    //  Internal function to start the actual connection establishment.
    // void start_connecting ();
    //  Internal function to add a connect timer
    // void add_connect_timer ();
    //  Open TCP connecting socket. Returns -1 in case of error,
    //  0 if connect was successful immediately. Returns -1 with
    //  EAGAIN errno if async connect was launched.
    // int open ();
    //  Get the file descriptor of newly created connection. Returns
    //  retired_fd if the connection was unsuccessful.
    // ZmqFileDesc connect ();
    //  Tunes a connected socket.
    // bool tune_socket (ZmqFileDesc fd);
    //  True iff a timer has been started.
    pub _connect_timer_started: bool,
    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqTcpConnector)
}

impl ZmqTcpConnector {
    pub fn new(
        io_thread_: &mut ZmqThreadContext,
        session_: &mut ZmqSessionBase,
        ctx: &ZmqContext,
        addr_: &mut ZmqAddress<TcpAddress>,
        delayed_start_: bool,
    ) -> Self {
        // StreamConnecterBase (
        //           io_thread_, session_, options_, addr_, delayed_start_),
        //         _connect_timer_started (false)
        // zmq_assert (_addr.protocol == protocol_name::tcp);
        Self {
            stream_connecter_base: StreamConnecterBase(
                io_thread_,
                session_,
                ctx,
                addr_,
                delayed_start_,
            ),
            _connect_timer_started: false,
        }
    }

    pub fn process_term(&mut self, linger: i32) {
        if self._connect_timer_started {
            cancel_timer(connect_timer_id);
            self._connect_timer_started = false;
        }

        self.stream_connecter_base.process_term(linger);
    }

    pub fn out_event(&mut self) {
        if self._connect_timer_started {
            cancel_timer(connect_timer_id);
            self._connect_timer_started = false;
        }

        //  TODO this is still very similar to (t)ipc_connecter_t, maybe the
        //  differences can be factored out

        rm_handle();

        let fd = self.stream_connecter_base.connect();

        if fd == retired_fd
            && ((options.reconnect_stop & ZMQ_RECONNECT_STOP_CONN_REFUSED) && errno == ECONNREFUSED)
        {
            send_conn_failed(_session);
            self.stream_connecter_base.close();
            terminate();
            return;
        }

        //  Handle the error condition by attempt to reconnect.
        if fd == retired_fd || tune_socket(fd).is_ok() {
            self.stream_connecter_base.close();
            add_reconnect_timer();
            return;
        }

        create_engine(fd, get_socket_name(fd, SocketEndLocal));
    }

    pub fn timer_event(&mut self, id_: i32) {
        if id_ == connect_timer_id {
            self._connect_timer_started = false;
            rm_handle();
            self.stream_connecter_base.close();
            add_reconnect_timer();
        } else {
            self.stream_connecter_base.timer_event(id_);
        }
    }

    pub fn start_connecting(&mut self) {
        //  Open the connecting socket.
        let rc: i32 = self.stream_connecter_base.open();

        //  Connect may succeed in synchronous manner.
        if (rc == 0) {
            _handle = add_fd(_s);
            out_event();
        }
        //  Connection establishment may be delayed. Poll for its completion.
        else if (rc == -1 && errno == EINPROGRESS) {
            _handle = add_fd(_s);
            set_pollout(_handle);
            self._socket.event_connect_delayed(
                make_unconnected_connect_endpoint_pair(_endpoint),
                zmq_errno(),
            );

            //  add userspace connect timeout
            add_connect_timer();
        }
        //  Handle any other error condition by eventual reconnect.
        else {
            if (_s != retired_fd) {
                self.stream_connecter_base.close();
            }
            add_reconnect_timer();
        }
    }

    pub fn add_connect_timer(&mut self) {
        if (options.connect_timeout > 0) {
            add_timer(options.connect_timeout, connect_timer_id);
            _connect_timer_started = true;
        }
    }

    pub unsafe fn open(&mut self) -> i32 {
        // zmq_assert (_s == retired_fd);

        //  Resolve the address
        if (_addr.resolved.tcp_addr != null_mut()) {
            LIBZMQ_DELETE(_addr.resolved.tcp_addr);
        }

        _addr.resolved.tcp_addr = TcpAddress();
        // alloc_assert (_addr.resolved.tcp_addr);
        _s = tcp_open_socket(
            _addr.address,
            self.options,
            false,
            true,
            _addr.resolved.tcp_addr,
        );
        if (_s == retired_fd) {
            //  TODO we should emit some event in this case!

            LIBZMQ_DELETE(_addr.resolved.tcp_addr);
            return -1;
        }
        // zmq_assert (_addr.resolved.tcp_addr != null_mut());

        // Set the socket to non-blocking mode so that we get async connect().
        unblock_socket(_s);

        let tcp_addr = _addr.resolved.tcp_addr;

        rc: i32;

        // Set a source address for conversations
        if (tcp_addr.has_src_addr()) {
            //  Allow reusing of the address, to connect to different servers
            //  using the same source port on the client.
            let mut flag = 1;
            // #ifdef ZMQ_HAVE_WINDOWS
            unsafe {
                rc = setsockopt(_s, SOL_SOCKET, SO_REUSEADDR, (&flag), 4);
            }
            wsa_assert(rc != SOCKET_ERROR);
            // #elif defined ZMQ_HAVE_VXWORKS
            //         unsafe {
            //             rc = setsockopt(_s, SOL_SOCKET, SO_REUSEADDR, &flag,
            //                             4);
            //         }
            //         // errno_assert (rc == 0);
            // #else
            //         unsafe { rc = setsockopt(_s, SOL_SOCKET, SO_REUSEADDR, &flag, 4); }
            //         // errno_assert (rc == 0);
            // #endif

            // #if defined ZMQ_HAVE_VXWORKS
            //         unsafe {
            //             rc = Bind(_s, tcp_addr.src_addr(),
            //                       tcp_addr.src_addrlen());
            //         }
            // #else
            unsafe {
                rc = bind(_s, tcp_addr.src_addr(), tcp_addr.src_addrlen());
            }
            // #endif
            if (rc == -1) {
                return -1;
            }
        }

        //  Connect to the remote peer.
        // #if defined ZMQ_HAVE_VXWORKS
        //     rc = ::connect (_s, (sockaddr *) tcp_addr.addr (), tcp_addr.addrlen ());
        // #else
        unsafe {
            rc = connect(_s, tcp_addr.addr(), tcp_addr.addrlen());
        }
        // #endif
        //  Connect was successful immediately.
        if rc == 0 {
            return 0;
        }

        //  Translate error codes indicating asynchronous connect has been
        //  launched to a uniform EINPROGRESS.
        // #ifdef ZMQ_HAVE_WINDOWS
        let last_error: WSA_ERROR = WSAGetLastError();
        if (last_error == WSAEINPROGRESS || last_error == WSAEWOULDBLOCK) {
            errno = EINPROGRESS;
        } else {
            errno = wsa_error_to_errno(last_error);
        }
        // #else
        if (errno == EINTR) {
            errno = EINPROGRESS;
        }
        // #endif
        return -1;
    }

    pub fn connect() -> ZmqFileDesc {
        //  Async connect has finished. Check whether an error occurred
        let mut err = 0;
        // #if defined ZMQ_HAVE_HPUX || defined ZMQ_HAVE_VXWORKS
        //     int len = sizeof err;
        // #else
        let mut len = 4usize;
        // #endif

        // let rc: i32 = unsafe {
        //     getsockopt(_s, SOL_SOCKET, SO_ERROR,
        //                (&err), &len)
        // };

        //  Assert if the error was caused by 0MQ bug.
        //  Networking problems are OK. No need to assert.
        // #ifdef ZMQ_HAVE_WINDOWS
        // zmq_assert (rc == 0);
        if (err != 0) {
            if err == WSAEBADF || err == WSAENOPROTOOPT || err == WSAENOTSOCK || err == WSAENOBUFS {
                wsa_assert_no(err);
            }
            errno = wsa_error_to_errno(err as WSA_ERROR);
            return retired_fd;
        }
        // #else
        //  Following code should handle both Berkeley-derived socket
        //  implementations and Solaris.
        if (rc == -1) {
            err = errno;
        }
        if (err != 0) {
            errno = err;
            // #if !defined(TARGET_OS_IPHONE) || !TARGET_OS_IPHONE
            // errno_assert (errno != EBADF && errno != ENOPROTOOPT
            //               && errno != ENOTSOCK && errno != ENOBUFS);
            // #else
            // errno_assert (errno != ENOPROTOOPT && errno != ENOTSOCK
            //               && errno != ENOBUFS);
            // #endif
            return retired_fd;
        }
        // #endif

        //  Return the newly connected socket.
        let result = _s;
        _s = retired_fd;
        return result;
    }

    pub fn tune_socket(&mut self, mut fd: ZmqFileDesc) -> bool {
        let rc: i32 = tune_tcp_socket(fd)
            | tune_tcp_keepalives(
                fd,
                options.tcp_keepalive,
                options.tcp_keepalive_cnt,
                options.tcp_keepalive_idle,
                options.tcp_keepalive_intvl,
            )
            | tune_tcp_maxrt(&mut fd, options.tcp_maxrt);
        return rc == 0;
    }
}
