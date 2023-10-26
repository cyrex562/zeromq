use std::mem::size_of_val;
use windows::Win32::Networking::WinSock::{setsockopt, SOCKET_ERROR, SOL_SOCKET};
use crate::address::{get_socket_name, socket_end_t};
use crate::address::socket_end_t::socket_end_local;
use crate::defines::{RETIRED_FD, SockaddrStorage};
use crate::endpoint::make_unconnected_bind_endpoint_pair;
use crate::fd::fd_t;
use crate::io_thread::ZmqIoThread;
use crate::ip::{make_socket_noninheritable, set_ip_type_of_service, set_nosigpipe, set_socket_priority};
use crate::options::ZmqOptions;
use crate::socket_base::ZmqSocketBase;
use crate::stream_listener_base::ZmqStreamListenerBase;
use crate::tcp::{tcp_open_socket, tune_tcp_keepalives, tune_tcp_maxrt, tune_tcp_socket};
use crate::tcp_address::ZmqTcpAddress;

pub struct ZmqTcpListener<'a> {
    pub stream_listener_base: ZmqStreamListenerBase<'a>,
    pub _address: ZmqTcpAddress,
}

impl ZmqTcpListener {
    pub fn new(io_thread_: &mut ZmqIoThread, socket_: &mut ZmqSocketBase, options_: &ZmqOptions) -> ZmqTcpListener {
        ZmqTcpListener {
            stream_listener_base: ZmqStreamListenerBase::new(io_thread_, socket_, options_),
            _address: ZmqTcpAddress::default(),
        }
    }

    // void zmq::tcp_listener_t::in_event ()
    pub unsafe fn in_event(&mut self)
    {
        let fd = self.accept ();

        //  If connection was reset by the peer in the meantime, just ignore it.
        //  TODO: Handle specific errors like ENFILE/EMFILE etc.
        if (fd == RETIRED_FD) {
            self._socket.event_accept_failed (
              make_unconnected_bind_endpoint_pair (self._endpoint), zmq_errno ());
            return;
        }

        let rc = tune_tcp_socket (fd);
        rc = rc
             | tune_tcp_keepalives (
               fd, self.options.tcp_keepalive, self.options.tcp_keepalive_cnt,
               self.options.tcp_keepalive_idle, self.options.tcp_keepalive_intvl);
        rc = rc | tune_tcp_maxrt (fd, self.options.tcp_maxrt);
        if (rc != 0) {
            self._socket.event_accept_failed (
              make_unconnected_bind_endpoint_pair (self._endpoint), zmq_errno ());
            return;
        }

        //  Create the engine object for this connection.
        self.create_engine (fd);
    }

    // std::string
    // zmq::tcp_listener_t::get_socket_name (zmq::fd_t fd_,
    //                                       socket_end_t socket_end_) const
    pub fn get_socket_name(&self, fd_: fd_t, socket_end_: socket_end_t) -> String
    {
        return get_socket_name::<ZmqTcpAddress> (fd_, socket_end_);
    }

    // int zmq::tcp_listener_t::create_socket (const char *addr_)
    pub unsafe fn create_socket(&mut self, addr_: &str) -> i32
    {
        self._s = tcp_open_socket (addr_, self.options, true, true, &self._address);
        if (self._s == RETIRED_FD) {
            return -1;
        }

        //  TODO why is this only done for the listener?
        make_socket_noninheritable (self._s);

        //  Allow reusing of the address.
        let mut flag = 1;
        let mut rc = 0;
    // #ifdef ZMQ_HAVE_WINDOWS
        #[cfg(target_os="windows")]
        {
            //  TODO this was changed for Windows from SO_REUSEADDRE to
            //  SE_EXCLUSIVEADDRUSE by 0ab65324195ad70205514d465b03d851a6de051c,
            //  so the comment above is no longer correct; also, now the settings are
            //  different between listener and connecter with a src address.
            //  is this intentional?
            rc = setsockopt(self._s, SOL_SOCKET, SO_EXCLUSIVEADDRUSE,
                            Some(&flag.to_le_bytes()));
            // wsa_assert(rc != SOCKET_ERROR);
        }
    // #elif defined ZMQ_HAVE_VXWORKS
    //     rc =
    //       setsockopt (_s, SOL_SOCKET, SO_REUSEADDR, (char *) &flag, sizeof (int));
    //     errno_assert (rc == 0);
    // #else
        #[cfg(not(target_os="windows"))]
        {
            rc = setsockopt(_s, SOL_SOCKET, SO_REUSEADDR, &flag, sizeof(int));
            errno_assert(rc == 0);
        }
    // #endif

        //  Bind the socket to the network interface and port.
    // #if defined ZMQ_HAVE_VXWORKS
    //     rc = bind (_s, (sockaddr *) _address.addr (), _address.addrlen ());
    // #else
        rc = self.bind (self._s, self._address.addr (), self._address.addrlen ());
    // #endif
    // #ifdef ZMQ_HAVE_WINDOWS
        #[cfg(target_os="windows")]
        {
            if (rc == SOCKET_ERROR) {
                // errno = wsa_error_to_errno(WSAGetLastError());
                // goto Error;
            }
        }
    // #else
        if (rc != 0) {}
            // goto Error;
    // #endif

        //  Listen for incoming connections.
        rc = self.listen (self._s, self.options.backlog);
    // #ifdef ZMQ_HAVE_WINDOWS
    #[cfg(target_os="windows")]
    {
        if (rc == SOCKET_ERROR) {
            // errno = wsa_error_to_errno(WSAGetLastError());
            // goto
            // Error;
        }
    }
    // #else
    #[cfg(not(target_os="windows"))]
        {
            if (rc != 0) {}
            // goto
            // Error;
        }
    // #endif

        return 0;

    // Error:
    //     const int err = errno;
        self.close ();
        // errno = err;
        return -1;
    }

    // int zmq::tcp_listener_t::set_local_address (const char *addr_)
    pub unsafe fn set_local_address(&mut self, addr_: &str) -> i32
    {
        if (self.options.use_fd != -1) {
            //  in this case, the addr_ passed is not used and ignored, since the
            //  socket was already created by the application
            self._s = self.options.use_fd;
        } else {
            if (self.create_socket (addr_) == -1)
                return -1;
        }

        self._endpoint = get_socket_name (self._s, socket_end_local);

        self._socket.event_listening (make_unconnected_bind_endpoint_pair (self._endpoint),
                                  self._s);
        return 0;
    }

    // zmq::fd_t zmq::tcp_listener_t::accept ()
    pub unsafe fn accept(&mut self) -> fd_t
    {
        //  The situation where connection cannot be accepted due to insufficient
        //  resources is considered valid and treated by ignoring the connection.
        //  Accept one connection and deal with different failure modes.
        // zmq_assert (_s != retired_fd);

        // struct sockaddr_storage ss;
        // memset (&ss, 0, sizeof (ss));
        let ss = SockaddrStorage::default();
    // #if defined ZMQ_HAVE_HPUX || defined ZMQ_HAVE_VXWORKS
    //     int ss_len = sizeof (ss);
    // #else
        let ss_len = size_of_val(&ss);
    // #endif
    // #if defined ZMQ_HAVE_SOCK_CLOEXEC && defined HAVE_ACCEPT4
    //     fd_t sock = ::accept4 (_s, reinterpret_cast<struct sockaddr *> (&ss),
    //                            &ss_len, SOCK_CLOEXEC);
    // #else
        let sock = libc::accept (self._s, (&ss), &ss_len);
    // #endif

        if (sock == RETIRED_FD) {
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

        make_socket_noninheritable (sock);

        if (!self.options.tcp_accept_filters.empty ()) {
            let mut matched = false;
            // for (options_t::tcp_accept_filters_t::size_type
            //        i = 0,
            //        size = options.tcp_accept_filters.size ();
            //      i != size; ++i)
            for i in 0..self.options.tcp_accept_filters.len()
            {
                if (self.options.tcp_accept_filters[i].match_address (
                      (&ss), ss_len)) {
                    matched = true;
                    break;
                }
            }
            if (!matched) {
    // #ifdef ZMQ_HAVE_WINDOWS
                let mut rc = 0i32;
    #[cfg(target_os="windows")]{
       rc = self.closesocket(sock);
    }
                // wsa_assert (rc != SOCKET_ERROR);
    // #else
                #[cfg(not(target_os="windows"))]{
                    rc = ::close(sock);
                }
                // errno_assert (rc == 0);
    // #endif
                return RETIRED_FD;
            }
        }

        if (set_nosigpipe (sock)) {
    // #ifdef ZMQ_HAVE_WINDOWS
            #[cfg(target_os="windows")]
            {
                rc = self.closesocket(sock);
            }
            // wsa_assert (rc != SOCKET_ERROR);
    // #else
            #[cfg(not(target_os="windows"))]
            {
                int
                rc = ::close(sock);
            }
            // errno_assert (rc == 0);
    // #endif
            return RETIRED_FD;
        }

        // Set the IP Type-Of-Service priority for this client socket
        if (self.options.tos != 0) {
            set_ip_type_of_service(sock, self.options.tos);
        }

        // Set the protocol-defined priority for this client socket
        if (self.options.priority != 0) {
            set_socket_priority(sock, self.options.priority);
        }

        return sock;
    }
}
