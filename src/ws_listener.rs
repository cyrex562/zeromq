/*
    Copyright (c) 2007-2019 Contributors as noted in the AUTHORS file

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

// #include "ws_listener.hpp"
// #include "io_thread.hpp"
// #include "config.hpp"
// #include "err.hpp"
// #include "ip.hpp"
// #include "tcp.hpp"
// #include "socket_base.hpp"
// #include "address.hpp"
// #include "ws_engine.hpp"
// #include "session_base.hpp"

// #ifdef ZMQ_HAVE_WSS
// #include "wss_engine.hpp"
// #include "wss_address.hpp"
// #endif

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

use crate::address::SocketEnd::{SocketEndLocal, SocketEndRemote};
use crate::address::{get_socket_name, SocketEnd};
use crate::context::{choose_io_thread, ZmqContext};
use crate::endpoint::EndpointType::endpoint_type_bind;
use crate::endpoint::{make_unconnected_bind_endpoint_pair, EndpointUriPair};
use crate::engine_interface::ZmqEngineInterface;
use crate::err::wsa_error_to_errno;
use crate::fd::ZmqFileDesc;
use crate::ip::make_socket_noninheritable;
use crate::ops::zmq_errno;

use crate::session_base::ZmqSessionBase;
use crate::socket::ZmqSocket;
use crate::stream_listener_base::ZmqStreamListenerBase;
use crate::tcp::{tcp_open_socket, tune_tcp_maxrt, tune_tcp_socket};
use crate::tcp_address::TcpAddress;
use crate::thread_context::ZmqThreadContext;
use crate::ws_address::WsAddress;
use crate::ws_engine::ZmqWsEngine;
use crate::wss_engine::WssEngine;
use libc::{bind, listen, setsockopt};
use windows::Win32::Networking::WinSock::{
    WSAGetLastError, SOCKET_ERROR, SOL_SOCKET, SO_REUSEADDR,
};

// #ifdef ZMQ_HAVE_OPENVMS
// #include <ioctl.h>
// #endif
pub struct ZmqWsListener {
    // : public ZmqStreamListenerBase
    pub base: ZmqStreamListenerBase,
    //     ZmqWsListener (ZmqIoThread *io_thread_,
    //                    socket: *mut ZmqSocketBase,
    //                    options: &ZmqOptions,
    //                    wss_: bool);

    // ~ZmqWsListener ();

    //  Set address to listen on.
    // int set_local_address (addr_: &str);

    // std::string get_socket_name (fd: ZmqFileDesc, SocketEnd socket_end_) const;
    // void create_engine (ZmqFileDesc fd);

    //
    //  Handlers for I/O events.
    // void in_event ();

    //  Accept the new connection. Returns the file descriptor of the
    //  newly created connection. The function may return retired_fd
    //  if the connection was dropped while waiting in the listen backlog
    //  or was denied because of accept filters.
    // ZmqFileDesc accept ();

    // int create_socket (addr_: &str);

    //  Address to listen on.
    // WsAddress address;
    pub address: WsAddress,

    pub _wss: bool,
    // #ifdef ZMQ_HAVE_WSS
    pub _tls_cred: gnutls_certificate_credentials_t,
    // #endif

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqWsListener)
}

impl ZmqWsListener {
    pub fn new(
        io_thread_: &mut ZmqThreadContext,
        socket: &mut ZmqSocket,
        ctx: &mut ZmqContext,
        wss_: bool,
    ) -> Self {
        let mut out = Self {
            base: ZmqStreamListenerBase::new(io_thread_, socket, ctx),
            address: Default::default(),
            _wss: false,
            _tls_cred: (),
        };
        // ZmqStreamListenerBase (io_thread_, socket, options_), _wss (wss_)
        // #ifdef ZMQ_HAVE_WSS
        if (_wss) {
            // TODO
            // int rc = gnutls_certificate_allocate_credentials (&_tls_cred);
            // // zmq_assert (rc == GNUTLS_E_SUCCESS);
            //
            // gnutls_datum_t cert = { options_.wss_cert_pem,
            //                        (unsigned int) options_.wss_cert_pem.length ()};
            // gnutls_datum_t key = { options_.wss_key_pem,
            //                       (unsigned int) options_.wss_key_pem.length ()};
            // rc = gnutls_certificate_set_x509_key_mem (_tls_cred, &cert, &key,
            //                                           GNUTLS_X509_FMT_PEM);
            // zmq_assert (rc == GNUTLS_E_SUCCESS);
        }

        out
        // #endif
    }

    // ZmqWsListener::~ZmqWsListener ()
    // {
    // // #ifdef ZMQ_HAVE_WSS
    //     if (_wss)
    //         gnutls_certificate_free_credentials (_tls_cred);
    // // #endif
    // }

    pub fn in_event(&mut self) {
        let mut fd = self.accept();

        //  If connection was reset by the peer in the meantime, just ignore it.
        //  TODO: Handle specific errors like ENFILE/EMFILE etc.
        if (fd == retired_fd) {
            self._socket
                .event_accept_failed(make_unconnected_bind_endpoint_pair(_endpoint), zmq_errno());
            return;
        }

        let mut rc = tune_tcp_socket(&mut fd);
        rc = rc | tune_tcp_maxrt(&mut fd, self.options.tcp_maxrt);
        if (rc != 0) {
            self._socket
                .event_accept_failed(make_unconnected_bind_endpoint_pair(_endpoint), zmq_errno());
            return;
        }

        //  Create the engine object for this connection.
        create_engine(fd);
    }

    pub fn get_socket_name(&mut self, fd: ZmqFileDesc, socket_end_: SocketEnd) -> String {
        socket_name: String;

        // #ifdef ZMQ_HAVE_WSS
        if (_wss) {
            socket_name = get_socket_name(fd, socket_end_);
        } else {
            // #endif
            socket_name = get_socket_name(fd, socket_end_);
        }

        return socket_name + address.path();
    }

    pub fn create_socket(&mut self, addr_: &mut str) -> i32 {
        // TcpAddress address;
        let mut address: TcpAddress = TcpAddress::default();
        _s = tcp_open_socket(addr_, self.options, true, true, &mut address);
        if (_s == retired_fd) {
            return -1;
        }

        //  TODO why is this only Done for the listener?
        make_socket_noninheritable(_s);

        //  Allow reusing of the address.
        let mut flag = 1;
        let mut rc = 0i32;
        // #ifdef ZMQ_HAVE_WINDOWS
        //  TODO this was changed for Windows from SO_REUSEADDRE to
        //  SE_EXCLUSIVEADDRUSE by 0ab65324195ad70205514d465b03d851a6de051c,
        //  so the comment above is no longer correct; also, now the settings are
        //  different between listener and connecter with a src address.
        //  is this intentional?
        unsafe {
            rc = setsockopt(_s, SOL_SOCKET, SO_EXCLUSIVEADDRUSE, (&flag), 4);
        }
        // wsa_assert (rc != SOCKET_ERROR);
        // #elif defined ZMQ_HAVE_VXWORKS
        //     rc =
        //       setsockopt (_s, SOL_SOCKET, SO_REUSEADDR,  &flag, mem::size_of::<int>());
        // errno_assert (rc == 0);
        // #else
        unsafe {
            rc = setsockopt(_s, SOL_SOCKET, SO_REUSEADDR, &flag, 4);
        }
        // errno_assert (rc == 0);
        // #endif

        //  Bind the socket to the network interface and port.
        // #if defined ZMQ_HAVE_VXWORKS
        unsafe {
            rc = bind(_s, address.addr(), address.addrlen());
        }
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
            rc = listen(_s, self.options.backlog);
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

        // error:
        //     let err: i32 = errno;
        //     close ();
        //     errno = err;
        //     return -1;
    }

    pub fn set_local_address(&mut self, addr_: &str) -> i32 {
        if (self.options.use_fd != -1) {
            //  in this case, the addr_ passed is not used and ignored, since the
            //  socket was already created by the application
            _s = self.options.use_fd;
        } else {
            let rc: i32 = address.resolve(addr_, true, self.options.ipv6);
            if (rc != 0) {
                return -1;
            }

            //  remove the path, otherwise resolving the port will fail with wildcard
            // TODO:
            // const char *delim = strrchr (addr_, '/');
            // host_address: String;
            // if (delim) {
            //     host_address = std::string (addr_, delim - addr_);
            // } else {
            //     host_address = addr_;
            // }

            if (create_socket(host_address) == -1) {
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

        //     struct sockaddr_storage ss;
        //     memset (&ss, 0, mem::size_of::<ss>());
        // // #if defined ZMQ_HAVE_HPUX || defined ZMQ_HAVE_VXWORKS
        //     int ss_len = mem::size_of::<ss>();
        // // #else
        //     socklen_t ss_len = mem::size_of::<ss>();
        // // #endif
        // // #if defined ZMQ_HAVE_SOCK_CLOEXEC && defined HAVE_ACCEPT4
        //      let mut sock: ZmqFileDesc = ::accept4 (_s, (&ss),
        //                            &ss_len, SOCK_CLOEXEC);
        // // #else
        //     const ZmqFileDesc sock =
        //       ::accept (_s, (&ss), &ss_len);
        // // #endif
        //
        //     if (sock == retired_fd) {
        // // #if defined ZMQ_HAVE_WINDOWS
        //         let last_error: i32 = WSAGetLastError ();
        //         wsa_assert (last_error == WSAEWOULDBLOCK || last_error == WSAECONNRESET
        //                     || last_error == WSAEMFILE || last_error == WSAENOBUFS);
        // #elif defined ZMQ_HAVE_ANDROID
        //         // errno_assert (errno == EAGAIN || errno == EWOULDBLOCK || errno == EINTR
        //                       || errno == ECONNABORTED || errno == EPROTO
        //                       || errno == ENOBUFS || errno == ENOMEM || errno == EMFILE
        //                       || errno == ENFILE || errno == EINVAL);
        // // #else
        //         // errno_assert (errno == EAGAIN || errno == EWOULDBLOCK || errno == EINTR
        //                       || errno == ECONNABORTED || errno == EPROTO
        //                       || errno == ENOBUFS || errno == ENOMEM || errno == EMFILE
        //                       || errno == ENFILE);
        // // #endif
        //         return retired_fd;
        //     }
        //
        //     make_socket_noninheritable (sock);
        //
        //     if (set_nosigpipe (sock)) {
        // // #ifdef ZMQ_HAVE_WINDOWS
        //         let rc: i32 = closesocket (sock);
        //         wsa_assert (rc != SOCKET_ERROR);
        // // #else
        //         int rc = ::close (sock);
        //         // errno_assert (rc == 0);
        // // #endif
        //         return retired_fd;
        //     }
        //
        //     // Set the IP Type-Of-Service priority for this client socket
        //     if (options.tos != 0)
        //         set_ip_type_of_service (sock, options.tos);
        //
        //     // Set the protocol-defined priority for this client socket
        //     if (options.priority != 0)
        //         set_socket_priority (sock, options.priority);
        //
        //     return sock;
        todo!()
    }

    pub fn create_engine(&mut self, fd: ZmqFileDesc) {
        let mut endpoint_pair = EndpointUriPair::new(
            &get_socket_name(fd, SocketEndLocal).unwrap(),
            &get_socket_name(fd, SocketEndRemote).unwrap(),
            endpoint_type_bind,
        );

        // ZmqEngineInterface *engine = null_mut();
        let mut engine: ZmqEngineInterface;
        if (_wss) {
            // #ifdef ZMQ_HAVE_WSS
            engine = WssEngine::new(
                fd,
                self.options,
                &mut endpoint_pair,
                address,
                false,
                _tls_cred,
                std::string(),
            );
            // #else
            // zmq_assert (false);
            // #endif
        } else {
            engine = ZmqWsEngine::new(fd, self.options, &mut endpoint_pair, address, false);
        }

        // alloc_assert (engine);

        //  Choose I/O thread to run connecter in. Given that we are already
        //  running in an I/O thread, there must be at least one available.
        let io_thread = self.choose_io_thread(self.options.affinity).unwrap();
        // zmq_assert (io_thread);

        //  Create and launch a session object.
        let mut session =
            ZmqSessionBase::create(self.ctx, io_thread, false, self._socket, self.options, None);
        // errno_assert (session);
        session.inc_seqnum();
        launch_child(&session);
        send_attach(session, engine, false);

        self._socket.event_accepted(endpoint_pair, fd);
    }
}
