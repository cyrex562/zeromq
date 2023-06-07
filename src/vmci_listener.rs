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

// #include "vmci_listener.hpp"

// #if defined ZMQ_HAVE_VMCI

// #include <new>

//#include "stream_engine.hpp"
// #include "vmci_address.hpp"
// #include "io_thread.hpp"
// #include "session_base.hpp"
// #include "config.hpp"
// #include "err.hpp"
// #include "ip.hpp"
// #include "socket_base.hpp"
// #include "vmci.hpp"

use crate::address::{get_socket_address, SocketEnd};
use crate::endpoint::make_unconnected_bind_endpoint_pair;
use crate::err::wsa_error_to_errno;
use crate::fd::ZmqFileDesc;
use crate::ip::open_socket;
use crate::mechanism::ZmqMechanismStatus::error;
use crate::ops::zmq_errno;

use crate::socket::ZmqSocket;
use crate::stream_listener_base::ZmqStreamListenerBase;
use crate::thread_context::ZmqThreadContext;
use crate::vmci::{tune_vmci_buffer_size, tune_vmci_connect_timeout};
use crate::vmci_address::ZmqVmciAddress;
use bincode::options;
use libc::{accept, close, ECONNABORTED, EMFILE, ENFILE, ENOBUFS, ENOMEM, EPROTO};
use std::ptr::null_mut;
use windows::s;
use windows::Win32::Foundation::{SetHandleInformation, BOOL, HANDLE_FLAG_INHERIT};
use windows::Win32::Networking::WinSock::{
    WSAGetLastError, INVALID_SOCKET, SOCKET_ERROR, SOCK_STREAM, WSAECONNRESET, WSAEMFILE,
    WSAENOBUFS, WSAEWOULDBLOCK,
};
use crate::context::ZmqContext;

// #if defined ZMQ_HAVE_WINDOWS
// #include "windows.hpp"
// #else
// #include <unistd.h>
// #include <fcntl.h>
// #endif
pub struct ZmqVmciListener {
    // : public ZmqStreamListenerBase
    pub stream_listener_base: ZmqStreamListenerBase,
    //
    //     ZmqVmciListener (ZmqIoThread *io_thread_,
    //                      socket: *mut ZmqSocketBase,
    //                      options: &ZmqOptions);

    //  Set address to listen on.
    // int set_local_address (addr_: &str);

    // std::string get_socket_name (fd: ZmqFileDesc, SocketEnd socket_end_) const;

    //
    //  Handlers for I/O events.
    // void in_event ();

    //  Accept the new connection. Returns the file descriptor of the
    //  newly created connection. The function may return retired_fd
    //  if the connection was dropped while waiting in the listen backlog.
    // ZmqFileDesc accept ();

    // int create_socket (addr_: &str);

    //  Address to listen on.
    pub address: ZmqVmciAddress,
    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqVmciListener)
}

impl ZmqVmciListener {
    // ZmqVmciListener::ZmqVmciListener (ZmqIoThread *io_thread_,
    //                                        ZmqSocketBase *socket,
    //                                        options: &ZmqOptions) :
    //     ZmqStreamListenerBase (io_thread_, socket, options_)
    pub fn new(
        io_thread_: &mut ZmqThreadContext,
        socket: &mut ZmqSocket,
        ctx: &ZmqContext,
    ) -> ZmqVmciListener {
        Self {
            stream_listener_base: ZmqStreamListenerBase {
                own: Default::default(),
                io_object: Default::default(),
                _s: 0,
                _handle: None,
                _socket: socket.clone(),
                _endpoint: "".to_string(),
            },
            address: ZmqVmciAddress {
                address: (),
                parent: Default::default(),
            },
        }
    }

    pub fn in_event(&mut self) {
        let mut fd: ZmqFileDesc = self.accept();

        //  If connection was reset by the peer in the meantime, just ignore it.
        if (fd == retired_fd) {
            self._socket
                .event_accept_failed(make_unconnected_bind_endpoint_pair(_endpoint), zmq_errno());
            return;
        }

        tune_vmci_buffer_size(
            this.get_ctx(),
            &mut fd,
            options.vmci_buffer_size,
            options.vmci_buffer_min_size,
            options.vmci_buffer_max_size,
        );

        if (options.vmci_connect_timeout > 0) {
            // #if defined ZMQ_HAVE_WINDOWS
            tune_vmci_connect_timeout(this.get_ctx(), &mut fd, options.vmci_connect_timeout);
            // #else
            //         struct timeval timeout = {0, options.vmci_connect_timeout * 1000};
            //         tune_vmci_connect_timeout (this.get_ctx (), fd, timeout);
            // #endif
        }

        //  Create the engine object for this connection.
        create_engine(fd);
    }

    pub fn get_socket_name(&mut self, fd: ZmqFileDesc, socket_end_: SocketEnd) -> String {
        // TODO
        // struct sockaddr_storage ss;
        // const ZmqSocklen sl = get_socket_address (fd, socket_end_, &ss);
        // if (sl == 0) {
        //     return std::string ();
        // }
        //
        // const ZmqVmciAddress addr ((&ss), sl,
        //                            this.get_ctx ());
        address_string: String;
        addr.to_string(address_string);
        return address_string;
    }

    pub fn set_local_address(&mut self, addr_: &str) -> i32 {
        //  Create addr on stack for auto-cleanup
        // std::string addr (addr_);

        //  Initialise the address structure.
        let mut address = ZmqVmciAddress::new2(self.get_ctx());
        let rc = address.resolve(addr.c_str());
        if (rc != 0) {
            return -1;
        }

        //  Create a listening socket.
        _s = open_socket(
            this.get_ctx().get_vmci_socket_family(),
            SOCK_STREAM as i32,
            0,
        );
        // #ifdef ZMQ_HAVE_WINDOWS
        if (s == INVALID_SOCKET) {
            // errno = wsa_error_to_errno (WSAGetLastError ());
            return -1;
        }
        // #if !defined _WIN32_WCE
        //  On Windows, preventing sockets to be inherited by child processes.
        // BOOL brc = SetHandleInformation ((HANDLE) _s, HANDLE_FLAG_INHERIT, 0);
        // win_assert (brc);
        // #endif
        // #else
        //     if (_s == -1) {
        //         return -1;
        //     }
        // #endif

        address.to_string(_endpoint);

        //  Bind the socket.
        rc = self.bind(_s, address.addr(), address.addrlen());
        // #ifdef ZMQ_HAVE_WINDOWS
        if (rc == SOCKET_ERROR) {
            // errno = wsa_error_to_errno (WSAGetLastError ());
            // goto error;
        }
        // #else
        if (rc != 0) {}
        // goto error;
        // #endif

        //  Listen for incoming connections.
        rc = self.listen(_s, options.backlog);
        // #ifdef ZMQ_HAVE_WINDOWS
        if (rc == SOCKET_ERROR) {
            // errno = wsa_error_to_errno (WSAGetLastError ());
            // goto error;
        }
        // #else
        if (rc != 0) {
            // goto
            // error;
        }
        // #endif

        self._socket
            .event_listening(make_unconnected_bind_endpoint_pair(_endpoint), _s);
        return 0;

        // error:
        //     int err = errno;
        //     close ();
        //     errno = err;
        //     return -1;
    }

    pub fn accept() -> ZmqFileDesc {
        //  Accept one connection and deal with different failure modes.
        //  The situation where connection cannot be accepted due to insufficient
        //  resources is considered valid and treated by ignoring the connection.
        // zmq_assert (_s != retired_fd);
        let mut sock: ZmqFileDesc = unsafe { accept(_s, null_mut(), null_mut()) };

        // #ifdef ZMQ_HAVE_WINDOWS
        if sock == INVALID_SOCKET as usize {
            // wsa_assert (WSAGetLastError () == WSAEWOULDBLOCK
            //             || WSAGetLastError () == WSAECONNRESET
            //             || WSAGetLastError () == WSAEMFILE
            //             || WSAGetLastError () == WSAENOBUFS);
            return retired_fd;
        }
        // #if !defined _WIN32_WCE
        //  On Windows, preventing sockets to be inherited by child processes.
        // let brc = SetHandleInformation ((HANDLE) sock, HANDLE_FLAG_INHERIT, 0);
        // win_assert (brc);
        // #endif
        // #else
        if sock == -1 {
            // errno_assert (errno == EAGAIN || errno == EWOULDBLOCK || errno == EINTR
            //               || errno == ECONNABORTED || errno == EPROTO
            //               || errno == ENOBUFS || errno == ENOMEM || errno == EMFILE
            //               || errno == ENFILE);
            return retired_fd;
        }
        // #endif

        //  Race condition can cause socket not to be closed (if fork happens
        //  between accept and this point).
        // #ifdef FD_CLOEXEC
        let rc = fcntl(sock, F_SETFD, FD_CLOEXEC);
        // errno_assert (rc != -1);
        // #endif

        return sock;
    }

    // #endif
}
