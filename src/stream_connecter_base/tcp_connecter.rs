use std::ptr::null_mut;
use libc::{ECONNREFUSED, EINPROGRESS};
use windows::Win32::Networking::WinSock::{SO_REUSEADDR, SOL_SOCKET};
use crate::address::{get_socket_name, ZmqAddress};
use crate::address::SocketEnd::SocketEndLocal;
use crate::address::tcp_address::ZmqTcpAddress;
use crate::defines::{SO_ERROR, ZMQ_RECONNECT_STOP_CONN_REFUSED, ZmqFd, ZmqSocklen};
use crate::endpoint::make_unconnected_connect_endpoint_pair;
use crate::defines::RETIRED_FD;
use crate::io::io_thread::ZmqIoThread;

use crate::ip::unblock_socket;

use crate::options::ZmqOptions;
use crate::session::ZmqSession;

use crate::stream_connecter_base::ZmqStreamConnecterBase;
use crate::tcp::{tcp_open_socket, tune_tcp_keepalives, tune_tcp_maxrt, tune_tcp_socket};
use crate::utils::get_errno;
use crate::utils::sock_utils::zmq_sockaddr_to_sockaddr;


pub const CONNECT_TIMER_ID: i32 = 2;
pub struct ZmqTcpConnecter<'a>
{
    pub stream_connecter_base: ZmqStreamConnecterBase<'a>,
    pub _connect_timer_started: bool,
    pub _addr: ZmqAddress<'a>,
}

impl ZmqTcpConnecter
{
    pub fn new(io_thread_: &mut ZmqIoThread, session_: &mut ZmqSession, addr_: ZmqAddress, delayed_start_: bool) -> Self
    {
        Self {
            stream_connecter_base: ZmqStreamConnecterBase::new(io_thread_, session_, addr_, delayed_start_),
            _connect_timer_started: false,
            _addr: Default::default(),
        }
    }

    pub unsafe fn process_term(&mut self, options: &ZmqOptions, linger_: i32) {
        if self._connect_timer_started {
            self.cancel_timer (CONNECT_TIMER_ID);
            self._connect_timer_started = false;
        }

        self.stream_connecter_base.process_term (options, linger_);
    }

    // void zmq::tcp_connecter_t::out_event ()
    pub fn out_event(&mut self, options: &ZmqOptions)
    {
        if self._connect_timer_started {
            self.cancel_timer (CONNECT_TIMER_ID);
            self._connect_timer_started = false;
        }

        //  TODO this is still very similar to (t)ipc_connecter_t, maybe the
        //  differences can be factored out

        self.rm_handle ();

        let fd = self.connect ();

        if fd == RETIRED_FD
            && ((options.reconnect_stop & ZMQ_RECONNECT_STOP_CONN_REFUSED) != 0
                && get_errno() == ECONNREFUSED) {
            self.send_conn_failed (self.stream_connecter_base._session);
            self.close ();
            self.terminate ();
            return;
        }

        //  Handle the Error condition by attempt to reconnect.
        if fd == RETIRED_FD || !self.tune_socket (options, fd) {
            self.close ();
            self.add_reconnect_timer ();
            return;
        }

        self.create_engine (fd, get_socket_name::<ZmqTcpAddress> (fd, SocketEndLocal));
    }

    // void zmq::tcp_connecter_t::timer_event (int id_)
    pub unsafe fn timer_event(&mut self, id_: i32)
    {
        if id_ == CONNECT_TIMER_ID {
            self._connect_timer_started = false;
            self.rm_handle ();
            self.close ();
            self.add_reconnect_timer ();
        } else {
            self.stream_connecter_base.timer_event(id_);
        }
    }

    // void zmq::tcp_connecter_t::start_connecting ()
    pub unsafe fn start_connecting(&mut self, options: &ZmqOptions)
    {
        //  Open the connecting socket.
        let rc = self.open (options);

        //  Connect may succeed in synchronous manner.
        if rc == 0 {
            self.stream_connecter_base._handle = self.add_fd (self.stream_connecter_base._s);
            self.out_event (options);
        }

        //  Connection establishment may be delayed. Poll for its completion.
        else if rc == -1 && get_errno() == EINPROGRESS {
            self.stream_connecter_base._handle = self.add_fd (self.stream_connecter_base._s);
            self.set_pollout (self.stream_connecter_base._handle);
            self.stream_connecter_base._socket.event_connect_delayed (options,
              &make_unconnected_connect_endpoint_pair (&self.stream_connecter_base._endpoint), get_errno ());

            //  add userspace connect timeout
            self.add_connect_timer (options);
        }

        //  Handle any other Error condition by eventual reconnect.
        else {
            if self.stream_connecter_base._s != RETIRED_FD {
                self.close();
            }
            self.add_reconnect_timer ();
        }
    }

    // void zmq::tcp_connecter_t::add_connect_timer ()
    pub unsafe fn add_connect_timer(&mut self, options: &ZmqOptions)
    {
        if options.connect_timeout > 0 {
            self.add_timer (options.connect_timeout, CONNECT_TIMER_ID);
            self._connect_timer_started = true;
        }
    }


    // int zmq::tcp_connecter_t::open ()
    pub unsafe fn open(&mut self, options: &ZmqOptions) -> i32
    {
        // zmq_assert (_s == retired_fd);

        //  Resolve the address
        if self._addr.tcp_addr != null_mut() {
            // LIBZMQ_DELETE (self._addr.resolved.tcp_addr);
        }

        self._addr.tcp_addr = ZmqTcpAddress::default();
        // alloc_assert (_addr->resolved.tcp_addr);
        self.stream_connecter_base._s = tcp_open_socket (
            &self._addr.address,
            options,
            false,
            true,
            &mut self._addr.tcp_addr
        );
        if self.stream_connecter_base._s == RETIRED_FD {
            //  TODO we should emit some event in this case!

            // LIBZMQ_DELETE (_addr->resolved.tcp_addr);
            return -1;
        }
        // zmq_assert (_addr->resolved.tcp_addr != NULL);

        // Set the socket to non-blocking mode so that we get async connect().
        unblock_socket (self.stream_connecter_base._s)?;

        let mut tcp_addr = self._addr.tcp_addr.clone();

        let mut rc = 0i32;

        // Set a source address for conversations
        if tcp_addr.has_src_addr () {
            //  Allow reusing of the address, to connect to different servers
            //  using the same source port on the client.
            let mut flag = 1;
    // #ifdef ZMQ_HAVE_WINDOWS
            #[cfg(target_os="windows")]
            {
                rc = self.setsockopt(self._s, SOL_SOCKET, SO_REUSEADDR, &flag, 4);
            }
            // wsa_assert (rc != SOCKET_ERROR);
    // #elif defined ZMQ_HAVE_VXWORKS
    //         rc = setsockopt (_s, SOL_SOCKET, SO_REUSEADDR, (char *) &flag,
    //                          sizeof (int));
    //         errno_assert (rc == 0);
    // #else
            #[cfg(not(target_os="windows"))]
            {
                rc = self.setsockopt(self.stream_connecter_base._s, SOL_SOCKET, SO_REUSEADDR, &flag, 4);
            }
            // errno_assert (rc == 0);
    // #endif

    // #if defined ZMQ_HAVE_VXWORKS
    //         rc = ::Bind (_s, (sockaddr *) tcp_addr->src_addr (),
    //                      tcp_addr->src_addrlen ());
    // #else
            let sa = zmq_sockaddr_to_sockaddr(tcp_addr.src_addr());
            rc = libc::bind (self.stream_connecter_base._s, &sa, tcp_addr.src_addrlen ());
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
        rc = libc::connect (self.stream_connecter_base._s, &sa, tcp_addr.addrlen () as ZmqSocklen);
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

    pub fn connect (&mut self) -> ZmqFd
    {
        //  Async connect has finished. Check whether an Error occurred
        let mut err = 0;
    // #if defined ZMQ_HAVE_HPUX || defined ZMQ_HAVE_VXWORKS
    //     int len = sizeof err;
    // #else
        let mut len = 4;
    // #endif

        let mut err_bytes: [u8;4] = [0;4];
        let  rc = unsafe {
            libc::getsockopt(self.stream_connecter_base._s, SOL_SOCKET, SO_ERROR,
                             err_bytes.as_mut_ptr() as *mut libc::c_void, &mut len)
        };

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
       let result = self.stream_connecter_base._s;
        self.stream_connecter_base._s = RETIRED_FD;
        return result;
    }

    // bool zmq::tcp_connecter_t::tune_socket (const fd_t fd_)
    pub fn tune_socket(&mut self, options: &ZmqOptions, fd_: ZmqFd) -> bool
    {
        let rc = tune_tcp_socket (fd_)
                       | tune_tcp_keepalives (
                         fd_, options.tcp_keepalive, options.tcp_keepalive_cnt,
                         options.tcp_keepalive_idle, options.tcp_keepalive_intvl)
                       | tune_tcp_maxrt (fd_, options.tcp_maxrt);
        return rc == 0;
    }
}
