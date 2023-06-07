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
// #include <stdio.h>

// #include "tcp_listener.hpp"
// #include "io_thread.hpp"
// #include "config.hpp"
// #include "err.hpp"
// #include "ip.hpp"
// #include "tcp.hpp"
// #include "socket_base.hpp"
// #include "address.hpp"

// #ifndef ZMQ_HAVE_WINDOWS
// #include <unistd.h>
// #include <sys/socket.h>
// #include <arpa/inet.h>
// #include <netinet/tcp.h>
// #include <netinet/in.h>
// #include <netdb.h>
// #include <fcntl.h>
// #ifdef ZMQ_HAVE_VXWORKS
// #include <sockLib.h>
// #endif
// #endif

use crate::address::SocketEnd::SocketEndLocal;
use crate::address::{get_socket_name, SocketEnd};
use crate::decoder_allocators::size;
use crate::endpoint::make_unconnected_bind_endpoint_pair;
use crate::err::wsa_error_to_errno;
use crate::fd::ZmqFileDesc;
use crate::ip::{
    make_socket_noninheritable, set_ip_type_of_service, set_nosigpipe, set_socket_priority,
};
use crate::mechanism::ZmqMechanismStatus::error;
use crate::ops::zmq_errno;

use crate::socket::ZmqSocket;
use crate::stream_listener_base::ZmqStreamListenerBase;
use crate::tcp::{tcp_open_socket, tune_tcp_keepalives, tune_tcp_maxrt, tune_tcp_socket};
use crate::tcp_address::TcpAddress;
use crate::thread_context::ZmqThreadContext;
use bincode::options;
use libc::{
    accept, bind, c_int, close, listen, memset, setsockopt, ECONNABORTED, EINVAL, EMFILE, ENFILE,
    ENOBUFS, ENOMEM, EPROTO,
};
use std::mem;
use windows::Win32::Networking::WinSock::{
    closesocket, socklen_t, WSAGetLastError, SOCKET_ERROR, SOL_SOCKET, SO_REUSEADDR, WSAECONNRESET,
    WSAEMFILE, WSAENOBUFS, WSAEWOULDBLOCK,
};
use crate::context::ZmqContext;

// #ifdef ZMQ_HAVE_OPENVMS
// #include <ioctl.h>
// #endif
pub struct TcpListener {
    //  : public ZmqStreamListenerBase
    pub stream_listener_base: ZmqStreamListenerBase,
    //     TcpListener (ZmqIoThread *io_thread_,
    //                     socket: *mut ZmqSocketBase,
    //                     options: &ZmqOptions);

    //  Set address to listen on.
    // int set_local_address (addr_: &str);
    // std::string get_socket_name (fd: ZmqFileDesc, SocketEnd socket_end_) const;

    //  Handlers for I/O events.
    // void in_event ();

    //  Accept the new connection. Returns the file descriptor of the
    //  newly created connection. The function may return retired_fd
    //  if the connection was dropped while waiting in the listen backlog
    //  or was denied because of accept filters.
    // ZmqFileDesc accept ();

    // int create_socket (addr_: &str);

    //  Address to listen on.
    // TcpAddress address;
    pub address: TcpAddress,
    // ZMQ_NON_COPYABLE_NOR_MOVABLE (TcpListener)
}

impl TcpListener {
    pub fn new(
        io_thread: &mut ZmqThreadContext,
        socket: &mut ZmqSocket,
        ctx: &mut ZmqContext,
    ) -> TcpListener {
        // ZmqStreamListenerBase (io_thread_, socket, options_)
        Self {
            stream_listener_base: ZmqStreamListenerBase::new(io_thread, socket, ctx),
            address: Default::default(),
        }
    }

    pub fn in_event(&mut self) {
        // TODO
        // let fd = unsafe { accept() };

        //  If connection was reset by the peer in the meantime, just ignore it.
        //  TODO: Handle specific errors like ENFILE/EMFILE etc.
        if (fd == retired_fd) {
            self._socket
                .event_accept_failed(make_unconnected_bind_endpoint_pair(_endpoint), zmq_errno());
            return;
        }

        let mut rc = tune_tcp_socket(fd);
        rc = rc
            | tune_tcp_keepalives(
                fd,
                options.tcp_keepalive,
                options.tcp_keepalive_cnt,
                options.tcp_keepalive_idle,
                options.tcp_keepalive_intvl,
            );
        rc = rc | tune_tcp_maxrt(fd, options.tcp_maxrt);
        if (rc != 0) {
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

    pub fn create_socket(&mut self, addr_: &mut str) -> i32 {
        _s = tcp_open_socket(addr_, self.options, true, true, &mut address);
        if (_s == retired_fd) {
            return -1;
        }

        //  TODO why is this only Done for the listener?
        make_socket_noninheritable(_s);

        //  Allow reusing of the address.
        let mut flag = 1;
        rc: i32;
        // #ifdef ZMQ_HAVE_WINDOWS
        //  TODO this was changed for Windows from SO_REUSEADDRE to
        //  SE_EXCLUSIVEADDRUSE by 0ab65324195ad70205514d465b03d851a6de051c,
        //  so the comment above is no longer correct; also, now the settings are
        //  different between listener and connecter with a src address.
        //  is this intentional?
        unsafe {
            rc = setsockopt(_s, SOL_SOCKET, SO_EXCLUSIVEADDRUSE, (&flag), 4);
        }
        wsa_assert(rc != SOCKET_ERROR);
        // #elif defined ZMQ_HAVE_VXWORKS
        //     rc =
        //       setsockopt (_s, SOL_SOCKET, SO_REUSEADDR,  &flag, mem::size_of::<int>());
        //     // errno_assert (rc == 0);
        // // #else
        //     rc = setsockopt (_s, SOL_SOCKET, SO_REUSEADDR, &flag, mem::size_of::<int>());
        //     // errno_assert (rc == 0);
        // #endif

        //  Bind the socket to the network interface and port.
        // #if defined ZMQ_HAVE_VXWORKS
        //     rc = Bind (_s, (sockaddr *) address.addr (), address.addrlen ());
        // #else
        unsafe {
            rc = bind(_s, address.addr(), address.addrlen());
        }
        // #endif
        // #ifdef ZMQ_HAVE_WINDOWS
        if (rc == SOCKET_ERROR) {
            unsafe {
                errno = wsa_error_to_errno(WSAGetLastError());
            }
            // goto error;
        }
        // #else
        if (rc != 0) {
            // goto
            // error;
        }
        // #endif

        //  Listen for incoming connections.
        unsafe {
            rc = listen(_s, options.backlog);
        }
        // #ifdef ZMQ_HAVE_WINDOWS
        if (rc == SOCKET_ERROR) {
            unsafe {
                errno = wsa_error_to_errno(WSAGetLastError());
            }
            // goto error;
        }
        // #else
        if (rc != 0) {
            // goto
            // error;
        }
        // #endif

        return 0;

        // TODO
        // error:
        //     let err: i32 = errno;
        //     close ();
        //     errno = err;
        //     return -1;
    }

    pub fn set_local_address(&mut self, addr_: &str) -> i32 {
        if options.use_fd != -1 {
            //  in this case, the addr_ passed is not used and ignored, since the
            //  socket was already created by the application
            _s = options.use_fd;
        } else {
            if (create_socket(addr_) == -1) {
                return -1;
            }
        }

        _endpoint = get_socket_name(_s, SocketEndLocal);

        self._socket
            .event_listening(make_unconnected_bind_endpoint_pair(_endpoint), _s);
        return 0;
    }

    pub fn accept(&mut self) -> ZmqFileDesc {
        //  The situation where connection cannot be accepted due to insufficient
        //  resources is considered valid and treated by ignoring the connection.
        //  Accept one connection and deal with different failure modes.
        // zmq_assert (_s != retired_fd);

        // struct sockaddr_storage ss;
        let mut ss: sockaddr_storage = sockaddr_storage::new();
        // memset (&ss, 0, mem::size_of::<ss>());
        // #if defined ZMQ_HAVE_HPUX || defined ZMQ_HAVE_VXWORKS
        //     let mut ss_len = mem::size_of::<ss>();
        // #else
        let mut ss_len: c_int = mem::size_of::<ss>() as c_int;
        // #endif
        // #if defined ZMQ_HAVE_SOCK_CLOEXEC && defined HAVE_ACCEPT4
        let mut sock: ZmqFileDesc = ::accept4(_s, (&ss), &ss_len, SOCK_CLOEXEC);
        // #else
        let sock = unsafe { accept(_s, (&mut ss), &mut ss_len) };
        // #endif

        unsafe {
            if (sock == retired_fd) {
                // #if defined ZMQ_HAVE_WINDOWS
                let last_error: i32 = WSAGetLastError() as i32;
                // wsa_assert(last_error == WSAEWOULDBLOCK || last_error == WSAECONNRESET || last_error == WSAEMFILE || last_error == WSAENOBUFS); # elif
                // defined
                // ZMQ_HAVE_ANDROID
                // errno_assert (errno == EAGAIN || errno == EWOULDBLOCK || errno == EINTR || errno == ECONNABORTED || errno == EPROTO || errno == ENOBUFS || errno == ENOMEM || errno == EMFILE || errno == ENFILE || errno == EINVAL);
                // #else
                // errno_assert (errno == EAGAIN || errno == EWOULDBLOCK || errno == EINTR
                // || errno == ECONNABORTED || errno == EPROTO || errno == ENOBUFS || errno == ENOMEM || errno == EMFILE || errno == ENFILE);
                // #endif
                return retired_fd;
            }
        }

        make_socket_noninheritable(sock);

        if (!options.tcp_accept_filters.empty()) {
            let mut matched = false;
            // for (ZmqOptions::tcp_accept_filters_t::size_type
            //        i = 0,
            //        size = options.tcp_accept_filters.size ();
            //      i != size; += 1i)
            for i in 0..options.tcp_accept_filters.len() {
                if (options.tcp_accept_filters[i].match_address((&ss), ss_len)) {
                    matched = true;
                    break;
                }
            }
            unsafe {
                if (!matched) {
                    // #ifdef ZMQ_HAVE_WINDOWS
                    let rc: i32 = closesocket(sock);
                    // wsa_assert (rc != SOCKET_ERROR);
                    // #else
                    //             int rc = ::close (sock);
                    // errno_assert (rc == 0);
                    // #endif
                    return retired_fd;
                }
            }
        }

        unsafe {
            if (set_nosigpipe(sock)) {
                // #ifdef ZMQ_HAVE_WINDOWS
                let rc: i32 = closesocket(sock);
                wsa_assert(rc != SOCKET_ERROR);
                // #else
                // let rc = ::close(sock);
                // errno_assert (rc == 0);
                // #endif
                return retired_fd;
            }
        }

        // Set the IP Type-Of-Service priority for this client socket
        if (options.tos != 0) {
            set_ip_type_of_service(sock, options.tos);
        }

        // Set the protocol-defined priority for this client socket
        if (options.priority != 0) {
            set_socket_priority(sock, options.priority);
        }

        return sock;
    }
}
