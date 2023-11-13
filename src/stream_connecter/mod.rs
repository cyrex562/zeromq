use std::ptr::null_mut;

use crate::address::{get_socket_name, ZmqAddress};
use crate::address::SocketEnd::SocketEndLocal;
use crate::address::tcp_address::ZmqTcpAddress;
use crate::defines::{ECONNREFUSED, EINPROGRESS, SO_ERROR, SO_REUSEADDR, SOL_SOCKET, ZMQ_RECONNECT_STOP_CONN_REFUSED, ZmqFd, ZmqHandle};
use crate::defines::RETIRED_FD;
use crate::endpoint::{make_unconnected_connect_endpoint_pair, ZmqEndpointUriPair};
use crate::endpoint::ZmqEndpointType::EndpointTypeConnect;
use crate::engine::ZmqEngine;
use crate::io::io_object::IoObject;
use crate::io::io_thread::ZmqIoThread;
use crate::ip::unblock_socket;
use crate::net::platform_socket::{platform_bind, platform_connect, platform_getsockopt};
use crate::options::ZmqOptions;
use crate::own::ZmqOwn;
use crate::session::ZmqSession;
use crate::socket::ZmqSocket;
use crate::tcp::{tcp_open_socket, tune_tcp_keepalives, tune_tcp_maxrt, tune_tcp_socket};
use crate::utils::get_errno;
use crate::utils::random::generate_random;
use crate::utils::sock_utils::zmq_sockaddr_to_sockaddr;

pub const RECONNECT_TIMER_ID: i32 = 1;
pub const CONNECT_TIMER_ID: i32 = 2;

pub struct ZmqStreamConnecterBase<'a> {
    pub own: ZmqOwn<'a>,
    pub io_object: IoObject<'a>,
    pub _addr: ZmqAddress<'a>,
    pub _s: ZmqFd,
    pub _handle: ZmqHandle,
    pub _endpoint: String,
    pub _socket: &'a ZmqSocket<'a>,
    pub _session: &'a ZmqSession<'a>,
    pub _delayed_start: bool,
    pub _reconnect_timer_started: bool,
    pub _current_reconnect_ivl: i32,
    pub _connect_timer_started: bool,
}

impl ZmqStreamConnecterBase {
    pub fn new(
        io_thread_: &mut ZmqIoThread,
        session_: &mut ZmqSession,
        addr_: ZmqAddress,
        delayed_start_: bool,
    ) -> Self {
        let mut out = Self {
            own: ZmqOwn::from_io_thread(io_thread_),
            io_object: IoObject::new(io_thread_),
            _addr: addr_,
            _s: RETIRED_FD,
            _handle: null_mut(),
            _endpoint: "".to_string(),
            _socket: session_.get_socket(),
            _session: session_,
            _delayed_start: delayed_start_,
            _reconnect_timer_started: false,
            _current_reconnect_ivl: 0,
            _connect_timer_started: false,
        };
        out
    }

    pub fn process_plug(&mut self, options: &ZmqOptions) {
        if self._delayed_start {
            self.add_reconnect_timer(options);
        } else {
            self.start_connecting(options);
        }
    }

    pub fn process_term(&mut self, options: &ZmqOptions, linger_: i32) {
        if self._connect_timer_started {
            self.cancel_timer(CONNECT_TIMER_ID);
            self._connect_timer_started = false;
        }

        if self._reconnect_timer_started {
            self.cancel_timer(RECONNECT_TIMER_ID);
            self._reconnect_timer_started = false;
        }

        if self._handle != null_mut() {
            self.rm_handle();
        }

        if self._s != RETIRED_FD {
            self.close(options);
        }

        self.own.process_term(linger_);
    }

    pub fn add_reconnect_timer(&mut self, options: &ZmqOptions) {
        if options.reconnect_ivl > 0 {
            let interval = self.get_new_reconnect_ivl(options);
            self.add_timer(interval, RECONNECT_TIMER_ID);
            self._socket.event_connect_retried(options,
                                               &make_unconnected_connect_endpoint_pair(&self._endpoint), interval,
            );
            self._reconnect_timer_started = true;
        }
    }

    pub fn get_new_reconnect_ivl(&mut self, options: &ZmqOptions) -> i32 {
        if options.reconnect_ivl_max > 0 {
            let mut candidate_interval = 0;
            if self._current_reconnect_ivl == -1 {
                candidate_interval = options.reconnect_ivl;
            } else if self._current_reconnect_ivl > i32::MAX / 2 {
                candidate_interval = i32::MAX;
            } else {
                candidate_interval = self._current_reconnect_ivl * 2;
            }

            if candidate_interval > options.reconnect_ivl_max {
                self._current_reconnect_ivl = options.reconnect_ivl_max;
            } else {
                self._current_reconnect_ivl = candidate_interval;
            }
            return self._current_reconnect_ivl;
        } else {
            if self._current_reconnect_ivl == -1 {
                self._current_reconnect_ivl = options.reconnect_ivl;
            }
            //  The new interval is the base interval + random value.
            let random_jitter = generate_random() % options.reconnect_ivl;
            let interval = if self._current_reconnect_ivl < i32::MAX - random_jitter {
                self._current_reconnect_ivl + random_jitter
            } else {
                i32::MAX
            };

            return interval;
        }
    }

    pub fn rm_handle(&mut self) {
        self.io_object.rm_fd(self._handle);
        self._handle = null_mut();
    }

    pub fn close(&mut self, options: &ZmqOptions) {
        #[cfg(target_os = "windows")]
        {
            closeseocket(self._s);
        }
        #[cfg(not(target_os = "windows"))]
        {
            unsafe { libc::close(self._s); }
        }
        self._socket.event_closed(options, &make_unconnected_connect_endpoint_pair(&self._endpoint), self._s);
        self._s = RETIRED_FD
    }

    pub unsafe fn in_event(&mut self, options: &ZmqOptions) {
        self.out_event(options);
    }

    pub fn create_engine(&mut self, options: &ZmqOptions, fd_: ZmqFd, local_address_: &str) {
        // const endpoint_uri_pair_t endpoint_pair (local_address_, _endpoint,
        //                                      EndpointTypeConnect);
        let mut endpoint_pair = ZmqEndpointUriPair::default();
        endpoint_pair.local = local_address_.to_string();
        endpoint_pair.remote = self._endpoint.clone();
        endpoint_pair.local_type = EndpointTypeConnect;

        //  Create the engine object for this connection.
        // i_engine *engine;
        let mut engine: ZmqEngine = ZmqEngine::default();
        // if (options.raw_socket) {
        //     // engine = new (std::nothrow) raw_engine_t (fd_, options, endpoint_pair);}
        //     engine = raw_engine_t::new(fd_, options, endpoint_pair);
        // }
        // else{
        //     // engine = new (std::nothrow) zmtp_engine_t (fd_, options, endpoint_pair);
        //     engine = zmtp_engine_t::new(fd_, options, endpoint_pair);
        //     }
        engine.fd = fd_;
        engine.endpoint_uri_pair = Some(endpoint_pair);
        // alloc_assert (engine);

        //  Attach the engine to the corresponding session object.
        self.send_attach(self._session, engine);

        //  Shut the connecter down.
        self.terminate();

        self._socket.event_connected(options, &endpoint_pair, fd_);
    }

    pub fn timer_event(&mut self, id_: i32, options: &ZmqOptions) {
        if id_ == CONNECT_TIMER_ID {
            self._connect_timer_started = false;
            self.rm_handle();
            self.close(options);
            self.add_reconnect_timer(options);
        } else {
            if id_ == RECONNECT_TIMER_ID {
                self._reconnect_timer_started = false;
                self.start_connecting(options);
            }
        }
    }

    pub fn out_event(&mut self, options: &ZmqOptions) {
        if self._connect_timer_started {
            self.cancel_timer(CONNECT_TIMER_ID);
            self._connect_timer_started = false;
        }

        //  TODO this is still very similar to (t)ipc_connecter_t, maybe the
        //  differences can be factored out

        self.rm_handle();

        let fd = self.connect();

        if fd == RETIRED_FD && ((options.reconnect_stop & ZMQ_RECONNECT_STOP_CONN_REFUSED) != 0 && get_errno() == ECONNREFUSED) {
            self.send_conn_failed(self._session);
            self.close(options);
            self.terminate();
            return;
        }

        //  Handle the Error condition by attempt to reconnect.
        if fd == RETIRED_FD || !self.tune_socket(options, fd) {
            self.close(options);
            self.add_reconnect_timer(options);
            return;
        }

        self.create_engine(
            options,
            fd,
            &get_socket_name::<ZmqTcpAddress>(fd, SocketEndLocal)?,
        );
    }

    pub fn start_connecting(&mut self, options: &ZmqOptions) {
        //  Open the connecting socket.
        let rc = self.open(options);

        //  Connect may succeed in synchronous manner.
        if rc == 0 {
            self._handle = self.add_fd(self._s);
            self.out_event(options);
        }

        //  Connection establishment may be delayed. Poll for its completion.
        else if rc == -1 && get_errno() == EINPROGRESS {
            self._handle = self.add_fd(self._s);
            self.set_pollout(self._handle);
            self._socket.event_connect_delayed(options,
                                               &make_unconnected_connect_endpoint_pair(&self._endpoint), get_errno());

            //  add userspace connect timeout
            self.add_connect_timer(options);
        }

        //  Handle any other Error condition by eventual reconnect.
        else {
            if self._s != RETIRED_FD {
                self.close(options);
            }
            self.add_reconnect_timer(options);
        }
    }

    pub fn add_connect_timer(&mut self, options: &ZmqOptions) {
        if options.connect_timeout > 0 {
            self.add_timer(options.connect_timeout, CONNECT_TIMER_ID);
            self._connect_timer_started = true;
        }
    }

    pub fn open(&mut self, options: &ZmqOptions) -> i32 {
        // zmq_assert (_s == retired_fd);

        //  Resolve the address
        if self._addr.tcp_addr != null_mut() {
            // LIBZMQ_DELETE (self._addr.resolved.tcp_addr);
        }

        self._addr.tcp_addr = ZmqTcpAddress::default();
        // alloc_assert (_addr->resolved.tcp_addr);
        self._s = tcp_open_socket(
            &mut self._addr.address,
            options,
            false,
            true,
            &mut self._addr.tcp_addr,
        );
        if self._s == RETIRED_FD {
            //  TODO we should emit some event in this case!

            // LIBZMQ_DELETE (_addr->resolved.tcp_addr);
            return -1;
        }
        // zmq_assert (_addr->resolved.tcp_addr != NULL);

        // Set the socket to non-blocking mode so that we get async connect().
        unblock_socket(self._s)?;

        let mut tcp_addr = self._addr.tcp_addr.clone();

        let mut rc = 0i32;

        // Set a source address for conversations
        if tcp_addr.has_src_addr() {
            //  Allow reusing of the address, to connect to different servers
            //  using the same source port on the client.
            let mut flag = 1;
            // #ifdef ZMQ_HAVE_WINDOWS
            #[cfg(target_os = "windows")]
            {
                rc = self.setsockopt(self._s, SOL_SOCKET, SO_REUSEADDR, &flag, 4);
            }
            // wsa_assert (rc != SOCKET_ERROR);
            // #elif defined ZMQ_HAVE_VXWORKS
            //         rc = setsockopt (_s, SOL_SOCKET, SO_REUSEADDR, (char *) &flag,
            //                          sizeof (int));
            //         errno_assert (rc == 0);
            // #else
            #[cfg(not(target_os = "windows"))]
            {
                rc = self.setsockopt(self._s, SOL_SOCKET, SO_REUSEADDR, &flag, 4);
            }
            // errno_assert (rc == 0);
            // #endif

            // #if defined ZMQ_HAVE_VXWORKS
            //         rc = ::Bind (_s, (sockaddr *) tcp_addr->src_addr (),
            //                      tcp_addr->src_addrlen ());
            // #else
            let sa = zmq_sockaddr_to_sockaddr(tcp_addr.src_addr());
            // rc = libc::bind(self._s, &sa, tcp_addr.src_addrlen());
            platform_bind(self._s, tcp_addr.src_addr())?;

            // #endif
            if rc == -1 {
                return -1;
            }
        }

        //  Connect to the remote peer.
        // #if defined ZMQ_HAVE_VXWORKS
        //     rc = ::connect (_s, (sockaddr *) tcp_addr->addr (), tcp_addr->addrlen ());
        // #else
        let sa = zmq_sockaddr_to_sockaddr(tcp_addr.addr());
        // rc = libc::connect(self._s, &sa, tcp_addr.addrlen() as ZmqSocklen);
        platform_connect(self._s, tcp_addr.addr())?;
        // #endif
        //  Connect was successful immediately.
        if rc == 0 {
            return 0;
        }

        //  Translate Error codes indicating asynchronous connect has been
        //  launched to a uniform EINPROGRESS.
        // #ifdef ZMQ_HAVE_WINDOWS
        //     const int last_error = WSAGetLastError ();
        //     if (last_error == WSAEINPROGRESS || last_error == WSAEWOULDBLOCK)
        //         errno = EINPROGRESS;
        //     else
        //         errno = wsa_error_to_errno (last_error);
        // #else
        //     if (errno == EINTR)
        //         errno = EINPROGRESS;
        // #endif
        return -1;
    }

    pub fn connect(&mut self) -> ZmqFd {
        //  Async connect has finished. Check whether an Error occurred
        let mut err = 0;
        // #if defined ZMQ_HAVE_HPUX || defined ZMQ_HAVE_VXWORKS
        //     int len = sizeof err;
        // #else
        let mut len = 4;
        // #endif

        let mut err_bytes: [u8; 4] = [0; 4];
        // let rc = unsafe {
        //     libc::getsockopt(self._s, SOL_SOCKET, SO_ERROR,
        //                      err_bytes.as_mut_ptr() as *mut libc::c_void, &mut len)
        // };
        let err_vec = platform_getsockopt(self._s, SOL_SOCKET as i32, SO_ERROR)?;
        err_bytes.clone_from_slice(err_vec.as_slice());

        err = i32::from_ne_bytes(err_bytes);

        //  Assert if the Error was caused by 0MQ bug.
        //  Networking problems are OK. No need to assert.
        // #ifdef ZMQ_HAVE_WINDOWS
        //     zmq_assert (rc == 0);
        //     if (err != 0) {
        //         if (err == WSAEBADF || err == WSAENOPROTOOPT || err == WSAENOTSOCK
        //             || err == WSAENOBUFS) {
        //             wsa_assert_no (err);
        //         }
        //         errno = wsa_error_to_errno (err);
        //         return retired_fd;
        //     }
        // #else
        //     //  Following code should handle both Berkeley-derived socket
        //     //  implementations and Solaris.
        //     if (rc == -1)
        //         err = errno;
        //     if (err != 0) {
        //         errno = err;
        // #if !defined(TARGET_OS_IPHONE) || !TARGET_OS_IPHONE
        //         errno_assert (errno != EBADF && errno != ENOPROTOOPT
        //                       && errno != ENOTSOCK && errno != ENOBUFS);
        // #else
        //         errno_assert (errno != ENOPROTOOPT && errno != ENOTSOCK
        //                       && errno != ENOBUFS);
        // #endif
        //         return retired_fd;
        //     }
        // #endif

        //  Return the newly connected socket.
        let result = self._s;
        self._s = RETIRED_FD;
        return result;
    }

    pub fn tune_socket(&mut self, options: &ZmqOptions, fd_: ZmqFd) -> bool {
        let rc = tune_tcp_socket(fd_) | tune_tcp_keepalives(
            fd_,
            options.tcp_keepalive,
            options.tcp_keepalive_cnt,
            options.tcp_keepalive_idle,
            options.tcp_keepalive_intvl,
        ) | tune_tcp_maxrt(fd_, options.tcp_maxrt);
        return rc == 0;
    }
}
