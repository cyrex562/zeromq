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

// #include "macros.hpp"
// #include "ws_connecter.hpp"
// #include "io_thread.hpp"
// #include "err.hpp"
// #include "ip.hpp"
// #include "tcp.hpp"
// #include "address.hpp"
// #include "ws_address.hpp"
// #include "ws_engine.hpp"
// #include "session_base.hpp"

// #ifdef ZMQ_HAVE_WSS
// #include "wss_engine.hpp"
// #include "wss_address.hpp"
// #endif

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

// #ifdef __APPLE__
// #include <TargetConditionals.h>
// #endif

use crate::address::get_socket_name;
use crate::address::SocketEnd::SocketEndLocal;
use crate::defines::ZmqFileDesc;
use crate::endpoint::EndpointType::Connect;
use crate::endpoint_uri::EndpointUriPair;
use crate::engine_interface::ZmqEngineInterface;
use crate::err::wsa_error_to_errno;
use crate::ip::{tune_socket, unblock_socket};
use crate::ops::zmq_errno;

use crate::context::ZmqContext;
use crate::session_base::ZmqSessionBase;
use crate::stream_connecter_base::StreamConnecterBase;
use crate::tcp::{tcp_open_socket, tune_tcp_maxrt, tune_tcp_socket};
use crate::tcp_address::TcpAddress;
use crate::thread_context::ZmqThreadContext;
use crate::ws_address::WsAddress;
use crate::ws_engine::ZmqWsEngine;
use crate::wss_engine::WssEngine;
use libc::{c_char, c_int, connect, getsockopt, EINPROGRESS, EINTR};
use windows::Win32::Networking::WinSock::{
    WSAGetLastError, SOL_SOCKET, SO_ERROR, WSAEBADF, WSAEINPROGRESS, WSAENOBUFS, WSAENOPROTOOPT,
    WSAENOTSOCK, WSAEWOULDBLOCK, WSA_ERROR,
};

pub const connect_timer_id: i32 = 2;

pub struct ZmqWsConnecter<'a> {
    //: public StreamConnecterBase
    pub base: StreamConnecterBase<'a>,
    //  If 'delayed_start' is true connecter first waits for a while,
    //  then starts connection process.
    // ZmqWsConnecter (ZmqIoThread *io_thread_,
    //                 ZmqSessionBase *session_,
    //                 options: &ZmqOptions,
    //                 Address *addr_,
    //                 delayed_start_: bool,
    //                 wss_: bool,
    //                 tls_hostname_: &str);
    // ~ZmqWsConnecter ();
    // void create_engine (fd: ZmqFileDesc, local_address_: &str);
    //
    //  ID of the timer used to check the connect timeout, must be different from stream_connecter_base_t::reconnect_timer_id.

    //  Handlers for incoming commands.
    // void process_term (linger: i32);

    //  Handlers for I/O events.
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

    //  Tunes a Connected socket.
    // bool tune_socket (ZmqFileDesc fd);

    //  True iff a timer has been started.
    pub _connect_timer_started: bool,

    pub _wss: bool,
    // const std::string &_hostname;
    pub _hostname: String,
    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqWsConnecter)
}

impl ZmqWsConnecter {
    // ZmqWsConnecter::ZmqWsConnecter (class ZmqIoThread *io_thread_,
    // pub struct ZmqSessionBase *session_,
    //                                      options: &ZmqOptions,
    //                                      Address *addr_,
    //                                      delayed_start_: bool,
    //                                      wss_: bool,
    //                                      tls_hostname_: &str) :
    //     StreamConnecterBase (
    //       io_thread_, session_, options_, addr_, delayed_start_),
    //     _connect_timer_started (false),
    //     _wss (wss_),
    //     _hostname (tls_hostname_)
    // {
    // }
    pub fn new(
        io_thread_: &mut ZmqThreadContext,
        session: &mut ZmqSessionBase,
        ctx: &mut ZmqContext,
        addr: &mut WsAddress,
        delayed_start: bool,
        wss: bool,
        tls_hostname: &str,
    ) -> Self {
        Self {
            base: StreamConnecterBase::new(io_thread, session, ctx, addr, delayed_start),
            _connect_timer_started: false,
            _wss: false,
            _hostname: "".to_string(),
        }
    }

    // ZmqWsConnecter::~ZmqWsConnecter ()
    // {
    //     // zmq_assert (!_connect_timer_started);
    // }

    pub fn process_term(&mut self, linger: i32) {
        if (_connect_timer_started) {
            cancel_timer(connect_timer_id);
            _connect_timer_started = false;
        }

        self.base.process_term(linger);
    }

    pub fn out_event(&mut self) {
        if (_connect_timer_started) {
            cancel_timer(connect_timer_id);
            _connect_timer_started = false;
        }

        //  TODO this is still very similar to (t)ipc_connecter_t, maybe the
        //  differences can be factored out

        rm_handle();

        let mut fd = self.connect();

        //  Handle the error condition by attempt to reconnect.
        if (fd == retired_fd || !self.tune_socket(&mut fd)) {
            self.close();
            add_reconnect_timer();
            return;
        }

        if (_wss) {
            // #ifdef ZMQ_HAVE_WSS
            create_engine(fd, get_socket_name(fd, SocketEndLocal));
        }
        // #else
        //         assert (false);
        // #endif
        else {
            create_engine(fd, get_socket_name(fd, SocketEndLocal));
        }
    }

    pub fn timer_event(&mut self, d_: i32) {
        if (id_ == connect_timer_id) {
            _connect_timer_started = false;
            rm_handle();
            self.close();
            add_reconnect_timer();
        } else {
            self.base.timer_event(id_);
        }
    }

    pub fn start_connecting(&mut self) {
        //  Open the connecting socket.
        let rc: i32 = self.open();

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
                self.close();
            }
            add_reconnect_timer();
        }
    }

    pub fn add_connect_timer(&mut self) {
        if (self.options.connect_timeout > 0) {
            add_timer(self.options.connect_timeout, connect_timer_id);
            _connect_timer_started = true;
        }
    }

    pub fn open(&mut self) -> i32 {
        // zmq_assert (_s == retired_fd);

        let mut tcp_addr = TcpAddress::default();
        _s = tcp_open_socket(_addr.address, self.options, false, true, &mut tcp_addr);
        if (_s == retired_fd) {
            return -1;
        }

        // Set the socket to non-blocking mode so that we get async connect().
        unblock_socket(_s);

        //  Connect to the remote peer.
        // #ifdef ZMQ_HAVE_VXWORKS
        //     let rc = connect (_s,  tcp_addr.addr (), tcp_addr.addrlen ());
        // #else
        let rc: i32 = unsafe { connect(_s, tcp_addr.addr(), tcp_addr.addrlen()) };
        // #endif
        //  Connect was successful immediately.
        if (rc == 0) {
            return 0;
        }

        //  Translate error codes indicating asynchronous connect has been
        //  launched to a uniform EINPROGRESS.
        // #ifdef ZMQ_HAVE_WINDOWS
        let last_error: i32 = unsafe { WSAGetLastError() } as i32;
        if last_error == WSAEINPROGRESS || last_error == WSAEWOULDBLOCK {
            errno = EINPROGRESS;
        } else {
            errno = wsa_error_to_errno(last_error as WSA_ERROR);
        }
        // #else
        if (errno == EINTR) {
            errno = EINPROGRESS;
        }
        // #endif
        return -1;
    }

    pub fn connect(&mut self) -> ZmqFileDesc {
        //  Async connect has finished. Check whether an error occurred
        let mut err = 0;
        // #if defined ZMQ_HAVE_HPUX || defined ZMQ_HAVE_VXWORKS
        //     int len = sizeof err;
        // #else
        //     socklen_t len = sizeof err;
        // #endif
        let mut len = 4usize;

        let rc: i32 = unsafe {
            getsockopt(
                _s,
                SOL_SOCKET,
                SO_ERROR,
                (&mut err) as *mut c_char,
                &mut (len as c_int),
            )
        };

        //  Assert if the error was caused by 0MQ bug.
        //  Networking problems are OK. No need to assert.
        // #ifdef ZMQ_HAVE_WINDOWS
        // zmq_assert (rc == 0);
        if (err != 0) {
            if (err == WSAEBADF || err == WSAENOPROTOOPT || err == WSAENOTSOCK || err == WSAENOBUFS)
            {
                wsa_assert_no(err);
            }
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

        //  Return the newly Connected socket.
        let result = _s;
        _s = retired_fd;
        return result;
    }

    pub fn tune_socket(&mut self, fd: &mut ZmqFileDesc) -> bool {
        let rc: i32 = tune_tcp_socket(fd) | tune_tcp_maxrt(fd, self.options.tcp_maxrt);
        return rc == 0;
    }

    pub fn create_engine(&mut self, fd: ZmqFileDesc, local_address_: &str) {
        let mut endpoint_pair =
            EndpointUriPair::new(local_address_, _endpoint, Connect);

        //  Create the engine object for this connection.
        let mut engine: ZmqEngine = ZmqEngine::new();
        if (_wss) {
            // #ifdef ZMQ_HAVE_WSS
            engine = WssEngine::new(
                fd,
                self.options,
                &mut endpoint_pair,
                *_addr.resolved.ws_addr,
                true,
                None,
                _hostname,
            );
        // #else
        //         LIBZMQ_UNUSED (_hostname);
        //         assert (false);
        // #endif
        } else {
            engine = ZmqWsEngine::new(
                fd,
                self.options,
                &endpoint_pair,
                *_addr.resolved.ws_addr,
                true,
            );
        }
        // alloc_assert (engine);

        //  Attach the engine to the corresponding session object.
        send_attach(_session, engine);

        //  Shut the connecter down.
        terminate();

        self._socket.event_connected(endpoint_pair, fd);
    }
}
