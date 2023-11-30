use std::mem::size_of_val;
use std::ptr::null_mut;
use libc::c_int;
use windows::Win32::Networking::WinSock::{setsockopt, SOCKET_ERROR};

use crate::address::{get_socket_name, SocketEnd};
use crate::address::SocketEnd::{SocketEndLocal, SocketEndRemote};
use crate::address::tcp_address::ZmqTcpAddress;
use crate::defines::{SO_REUSEADDR, SOL_SOCKET, ZmqFd, ZmqHandle, ZmqSockaddrStorage};
use crate::defines::err::ZmqError;
use crate::defines::err::ZmqError::SocketError;
use crate::defines::RETIRED_FD;
use crate::endpoint::{make_unconnected_bind_endpoint_pair, make_unconnected_connect_endpoint_pair, ZmqEndpointUriPair};
use crate::endpoint::ZmqEndpointType::EndpointTypeBind;
use crate::engine::ZmqEngine;
use crate::io::io_object::IoObject;
use crate::io::io_thread::ZmqIoThread;
use crate::ip::{set_ip_type_of_service, set_nosigpipe, set_socket_priority};
use crate::platform::{platform_make_socket_noninheritable, platform_setsockopt};
use crate::options::ZmqOptions;
use crate::own::ZmqOwn;
use crate::session::ZmqSession;
use crate::socket::ZmqSocket;
use crate::tcp::{tcp_open_socket, tune_tcp_keepalives, tune_tcp_maxrt, tune_tcp_socket};
use crate::utils::get_errno;
use crate::utils::sock_utils::zmq_sockaddr_storage_to_sockaddr;

pub struct ZmqStreamListener<'a> {
    pub own: ZmqOwn<'a>,
    pub io_object: IoObject<'a>,
    pub _s: ZmqFd,
    pub _handle: ZmqHandle,
    pub _socket: &'a mut ZmqSocket<'a>,
    pub _endpoint: String,
    pub _address: Option<ZmqTcpAddress>,
}

impl<'a> ZmqStreamListener<'a> {
    pub fn new(
        io_thread_: &mut ZmqIoThread,
        socket_: &mut ZmqSocket,
    ) -> Self {
        Self {
            own: ZmqOwn::from_io_thread(io_thread_),
            io_object: IoObject::new(io_thread_),
            _s: RETIRED_FD,
            _handle: null_mut(),
            _socket: socket_,
            _endpoint: "".to_string(),
            _address: None,
        }
    }

    pub fn get_local_address(&mut self, addr_: &mut String) -> Result<(),ZmqError> {
        *addr_ = get_socket_name(self._s, SocketEndLocal)?;
        if addr_.is_empty() {
            return Err(SocketError("failed to get local address"));
        }
        return Ok(());
    }

    pub fn process_plug(&mut self) {
        self._handle = self.add_fd(self._s);
        self.set_pollin();
    }

    pub fn process_term(&mut self, options: &ZmqOptions, linger_: i32) {
        self.rm_fd(self._handle);
        self.close(options);
        self._handle = null_mut();
        self.own.process_term(linger_);
    }

    pub fn close(&mut self, options: &ZmqOptions) {
        #[cfg(target_os = "windows")]
        {
            closeseocket(self._s);
        }
        #[cfg(not(target_os = "windows"))]
        {
            libc::close(self._s);
        }
        self._socket.event_closed(options, &make_unconnected_connect_endpoint_pair(&self._endpoint), self._s);
        self._s = RETIRED_FD
    }

    pub fn create_engine(&mut self, options: &ZmqOptions, fd_: ZmqFd) {
        let endpoint_pair = ZmqEndpointUriPair::new(
            &get_socket_name(fd_, SocketEndLocal)?,
            &get_socket_name(fd_, SocketEndRemote)?,
            EndpointTypeBind,
        );

        // i_engine *engine;
        // let mut engine: dyn IEngine;
        // if options.raw_socket {
        //     engine = raw_engine_t::new(fd_, options, endpoint_pair);
        // } else {
        //     engine = zmtp_engine_t::new(fd_, options, endpoint_pair);
        // }
        let mut engine = ZmqEngine::new();
        engine.endpoint_uri_pair = Some(endpoint_pair);
        engine.fd = fd_;
        // alloc_assert (engine);

        //  Choose I/O thread to run connecter in. Given that we are already
        //  running in an I/O thread, there must be at least one available.
        let io_thread = self.choose_io_thread(options.affinity);
        // zmq_assert (io_thread);

        //  Create and launch a session object.
        let mut session = ZmqSession::create(io_thread, false, self._socket, options, None);
        // errno_assert (session);
        session.inc_seqnum();
        self.launch_child(&session);
        self.send_attach(&session, engine, false);

        self._socket.event_accepted(options, &endpoint_pair, fd_);
    }

    pub fn in_event(&mut self, options: &ZmqOptions) {
        let fd = self.accept(options);

        //  If connection was reset by the peer in the meantime, just ignore it.
        //  TODO: Handle specific errors like ENFILE/EMFILE etc.
        if fd == RETIRED_FD {
            self._socket.event_accept_failed(options,
                                             &make_unconnected_bind_endpoint_pair(&self._endpoint), get_errno());
            return;
        }

        let mut rc = tune_tcp_socket(fd);
        rc = rc | tune_tcp_keepalives(
            fd, options.tcp_keepalive, options.tcp_keepalive_cnt,
            options.tcp_keepalive_idle, options.tcp_keepalive_intvl);
        rc = rc | tune_tcp_maxrt(fd, options.tcp_maxrt);
        if rc != 0 {
            self._socket.event_accept_failed(options,
                                             &make_unconnected_bind_endpoint_pair(&self._endpoint), get_errno());
            return;
        }

        //  Create the engine object for this connection.
        self.create_engine(options, fd);
    }

    pub fn get_socket_name(&self, fd_: ZmqFd, socket_end_: SocketEnd) -> String {
        return get_socket_name::<ZmqTcpAddress>(fd_, socket_end_)?;
    }

    pub fn create_socket(&mut self, options: &ZmqOptions, addr_: &str) -> Result<(),ZmqError> {
        self._s = tcp_open_socket(&mut addr_.to_string(), options, true, true, &mut self._address.unwrap());
        if self._s == RETIRED_FD {
            return Err(SocketError("failed to open socket"));
        }

        //  TODO why is this only Done for the listener?
        platform_make_socket_noninheritable(self._s)?;

        //  Allow reusing of the address.
        let mut flag = 1;
        let mut rc = 0;
        // #ifdef ZMQ_HAVE_WINDOWS
        #[cfg(target_os = "windows")]
        unsafe {
            //  TODO this was changed for Windows from SO_REUSEADDRE to
            //  SE_EXCLUSIVEADDRUSE by 0ab65324195ad70205514d465b03d851a6de051c,
            //  so the comment above is no longer correct; also, now the settings are
            //  different between listener and connecter with a src address.
            //  is this intentional?
            rc = setsockopt(
                self._s,
                SOL_SOCKET as i32,
                SO_REUSEADDR,
                Some(&flag.to_le_bytes())
            );
            // wsa_assert(rc != SOCKET_ERROR);
        }
        // #elif defined ZMQ_HAVE_VXWORKS
        //     rc =
        //       setsockopt (_s, SOL_SOCKET, SO_REUSEADDR, (char *) &flag, sizeof (int));
        //     errno_assert (rc == 0);
        // #else
        #[cfg(not(target_os = "windows"))]
        {
            platform_setsockopt(self._s, SOL_SOCKET as i32, SO_REUSEADDR, &flag.to_le_bytes(), 4)?;
            // errno_assert(rc == 0);
        }
        // #endif

        //  Bind the socket to the network interface and port.
        // #if defined ZMQ_HAVE_VXWORKS
        //     rc = Bind (_s, (sockaddr *) _address.addr (), _address.addrlen ());
        // #else
        rc = self.bind(self._s, self._address.addr(), self._address.addrlen());
        // #endif
        // #ifdef ZMQ_HAVE_WINDOWS
        #[cfg(target_os = "windows")]
        {
            if (rc == SOCKET_ERROR) {
                // errno = wsa_error_to_errno(WSAGetLastError());
                // goto Error;
            }
        }
        // #else
        if rc != 0 {}
        // goto Error;
        // #endif

        //  Listen for incoming connections.
        rc = self.listen(self._s, options.backlog);
        // #ifdef ZMQ_HAVE_WINDOWS
        #[cfg(target_os = "windows")]
        {
            if (rc == SOCKET_ERROR) {
                // errno = wsa_error_to_errno(WSAGetLastError());
                // goto
                // Error;
            }
        }
        // #else
        #[cfg(not(target_os = "windows"))]
        {
            if rc != 0 {}
            // goto
            // Error;
        }
        // #endif

        return Ok(());

        // Error:
        //     const int err = errno;
        self.close(options);
        // errno = err;
        return -1;
    }

    pub fn set_local_address(&mut self, options: &ZmqOptions, addr_: &str) -> Result<(),ZmqError> {
        if options.use_fd != -1 {
            //  in this case, the addr_ passed is not used and ignored, since the
            //  socket was already created by the application
            self._s = options.use_fd;
        } else {
            if self.create_socket(options, addr_) == -1 {
                return Err(SocketError("failed to create socket"));
            }
        }

        self._endpoint = get_socket_name(
            self._s,
            SocketEndLocal,
        )?;

        self._socket.event_listening(
            options,
            &make_unconnected_bind_endpoint_pair(&self._endpoint),
            self._s,
        );
        return Ok(());
    }

    pub fn accept(&mut self, options: &ZmqOptions) -> ZmqFd {
        //  The situation where connection cannot be accepted due to insufficient
        //  resources is considered valid and treated by ignoring the connection.
        //  Accept one connection and deal with different failure modes.
        // zmq_assert (_s != retired_fd);

        // struct sockaddr_storage ss;
        // memset (&ss, 0, sizeof (ss));
        let ss = ZmqSockaddrStorage::default();
        // #if defined ZMQ_HAVE_HPUX || defined ZMQ_HAVE_VXWORKS
        //     int ss_len = sizeof (ss);
        // #else
        let mut ss_len = size_of_val(&ss) as c_int;
        // #endif
        // #if defined ZMQ_HAVE_SOCK_CLOEXEC && defined HAVE_ACCEPT4
        //     fd_t sock = ::accept4 (_s, reinterpret_cast<struct sockaddr *> (&ss),
        //                            &ss_len, SOCK_CLOEXEC);
        // #else
        let sock = unsafe {
            libc::accept(
                self._s,
                &mut zmq_sockaddr_storage_to_sockaddr(&ss),
                &mut ss_len,
            )
        };
        // #endif

        if sock == RETIRED_FD {
            // #if defined ZMQ_HAVE_WINDOWS
            //         const int last_error = WSAGetLastError ();
            //         // wsa_assert (last_error == WSAEWOULDBLOCK || last_error == WSAECONNRESET
            //         //             || last_error == WSAEMFILE || last_error == WSAENOBUFS);
            // #elif defined ZMQ_HAVE_ANDROID
            //         // errno_assert (errno == EAGAIN || errno == EWOULDBLOCK || errno == EINTR
            //         //               || errno == ECONNABORTED || errno == EPROTO
            //         //               || errno == ENOBUFS || errno == ENOMEM || errno == EMFILE
            //         //               || errno == ENFILE || errno == EINVAL);
            // #else
            //         // errno_assert (errno == EAGAIN || errno == EWOULDBLOCK || errno == EINTR
            //         //               || errno == ECONNABORTED || errno == EPROTO
            //         //               || errno == ENOBUFS || errno == ENOMEM || errno == EMFILE
            //         //               || errno == ENFILE);
            // #endif
            return RETIRED_FD;
        }

        platform_make_socket_noninheritable(sock);

        if !options.tcp_accept_filters.empty() {
            let mut matched = false;
            // for (options_t::tcp_accept_filters_t::size_type
            //        i = 0,
            //        size = options.tcp_accept_filters.size ();
            //      i != size; ++i)
            for i in 0..options.tcp_accept_filters.len() {
                if options.tcp_accept_filters[i].match_address(
                    &zmq_sockaddr_storage_to_sockaddr(&ss),
                    ss_len as libc::socklen_t,
                ) {
                    matched = true;
                    break;
                }
            }
            if !matched {
                // #ifdef ZMQ_HAVE_WINDOWS
                let mut rc = 0i32;
                #[cfg(target_os = "windows")]{
                    rc = self.closesocket(sock);
                }
                // wsa_assert (rc != SOCKET_ERROR);
                // #else
                #[cfg(not(target_os = "windows"))]{
                    rc = libc::close(sock);
                }
                // errno_assert (rc == 0);
                // #endif
                return RETIRED_FD;
            }
        }

        if set_nosigpipe(sock) {
            // #ifdef ZMQ_HAVE_WINDOWS
            #[cfg(target_os = "windows")]
            {
                rc = self.closesocket(sock);
            }
            // wsa_assert (rc != SOCKET_ERROR);
            // #else
            #[cfg(not(target_os = "windows"))]
            {
                let rc = libc::close(sock);
            }
            // errno_assert (rc == 0);
            // #endif
            return RETIRED_FD;
        }

        // Set the IP Type-Of-Service priority for this client socket
        if options.tos != 0 {
            set_ip_type_of_service(sock, options.tos)?;
        }

        // Set the protocol-defined priority for this client socket
        if options.priority != 0 {
            set_socket_priority(sock, options.priority)?;
        }

        return sock;
    }
}
