use std::ptr::null_mut;
use libc::{ECONNREFUSED, EINPROGRESS};
use windows::Win32::Networking::WinSock::{SO_REUSEADDR, SOL_SOCKET};
use crate::address::{ZmqAddress, get_socket_name};
use crate::address::socket_end_t::socket_end_local;
use crate::defines::ZMQ_RECONNECT_STOP_CONN_REFUSED;
use crate::endpoint::make_unconnected_connect_endpoint_pair;
use crate::fd::retired_fd;
use crate::io_thread::ZmqIoThread;
use crate::ip::{tune_socket, unblock_socket};
use crate::options::ZmqOptions;
use crate::session_base::ZmqSessionBase;
use crate::stream_connecter_base::ZmqStreamConnecterBase;
use crate::tcp::{tcp_open_socket, tune_tcp_keepalives, tune_tcp_maxrt, tune_tcp_socket};
use crate::tcp_address::ZmqTcpAddress;

pub const connect_timer_id: i32 = 2;
pub struct ZmqTcpConnecter
{
    pub stream_connecter_base: ZmqStreamConnecterBase,
    pub _connect_timer_started: bool,
    _addr:
}

impl ZmqTcpConnecter
{
    pub fn new(io_thread_: &mut ZmqIoThread, session_: &mut ZmqSessionBase, options_: &ZmqOptions, addr_: ZmqAddress, delayed_start_: bool) -> Self
    {
        Self {
            stream_connecter_base: ZmqStreamConnecterBase::new(io_thread_, session_, options_, addr_, delayed_start_),
            _connect_timer_started: false,
        }
    }

    pub unsafe fn process_term(&mut self, linger_: i32) {
        if (self._connect_timer_started) {
            self.cancel_timer (connect_timer_id);
            self._connect_timer_started = false;
        }

        self.stream_connecter_base.process_term (linger_);
    }

    // void zmq::tcp_connecter_t::out_event ()
    pub unsafe fn out_event(&mut self)
    {
        if (self._connect_timer_started) {
            self.cancel_timer (connect_timer_id);
            self._connect_timer_started = false;
        }

        //  TODO this is still very similar to (t)ipc_connecter_t, maybe the
        //  differences can be factored out

        self.rm_handle ();

        let fd = self.connect ();

        if (fd == retired_fd
            && ((self.options.reconnect_stop & ZMQ_RECONNECT_STOP_CONN_REFUSED)
                && errno == ECONNREFUSED)) {
            self.send_conn_failed (self._session);
            self.close ();
            self.terminate ();
            return;
        }

        //  Handle the Error condition by attempt to reconnect.
        if (fd == retired_fd || !tune_socket (fd)) {
            self.close ();
            self.add_reconnect_timer ();
            return;
        }

        self.create_engine (fd, get_socket_name::<ZmqTcpAddress> (fd, socket_end_local));
    }

    // void zmq::tcp_connecter_t::timer_event (int id_)
    pub unsafe fn timer_event(&mut self, id_: i32)
    {
        if (id_ == connect_timer_id) {
            self._connect_timer_started = false;
            self.rm_handle ();
            self.close ();
            self.add_reconnect_timer ();
        } else {
            self.stream_connecter_base.timer_event(id_);
        }
    }

    // void zmq::tcp_connecter_t::start_connecting ()
    pub unsafe fn start_connecting(&mut self)
    {
        //  Open the connecting socket.
        let rc = self.open ();

        //  Connect may succeed in synchronous manner.
        if (rc == 0) {
            self._handle = self.add_fd (_s);
            self.out_event ();
        }

        //  Connection establishment may be delayed. Poll for its completion.
        else if (rc == -1 && errno == EINPROGRESS) {
            self._handle = self.add_fd (_s);
            self.set_pollout (self._handle);
            self._socket.event_connect_delayed (
              make_unconnected_connect_endpoint_pair (self._endpoint), zmq_errno ());

            //  add userspace connect timeout
            self.add_connect_timer ();
        }

        //  Handle any other Error condition by eventual reconnect.
        else {
            if (self._s != retired_fd) {
                self.close();
            }
            self.add_reconnect_timer ();
        }
    }

    // void zmq::tcp_connecter_t::add_connect_timer ()
    pub unsafe fn add_connect_timer(&mut self)
    {
        if (self.options.connect_timeout > 0) {
            self.add_timer (self.options.connect_timeout, connect_timer_id);
            self._connect_timer_started = true;
        }
    }


    // int zmq::tcp_connecter_t::open ()
    pub unsafe fn open(&mut self) -> i32
    {
        // zmq_assert (_s == retired_fd);

        //  Resolve the address
        if (self._addr.resolved.tcp_addr != null_mut()) {
            // LIBZMQ_DELETE (self._addr.resolved.tcp_addr);
        }

        self._addr.resolved.tcp_addr = ZmqTcpAddress::default();
        // alloc_assert (_addr->resolved.tcp_addr);
        self._s = tcp_open_socket (self._addr.address.c_str (), self.options, false, true,
                              self._addr.resolved.tcp_addr);
        if (self._s == retired_fd) {
            //  TODO we should emit some event in this case!

            // LIBZMQ_DELETE (_addr->resolved.tcp_addr);
            return -1;
        }
        // zmq_assert (_addr->resolved.tcp_addr != NULL);

        // Set the socket to non-blocking mode so that we get async connect().
        unblock_socket (self._s);

        let tcp_addr = self._addr.resolved.tcp_addr;

        let mut rc = 0i32;

        // Set a source address for conversations
        if (tcp_addr.has_src_addr ()) {
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
                rc = self.setsockopt(self._s, SOL_SOCKET, SO_REUSEADDR, &flag, 4);
            }
            // errno_assert (rc == 0);
    // #endif

    // #if defined ZMQ_HAVE_VXWORKS
    //         rc = ::bind (_s, (sockaddr *) tcp_addr->src_addr (),
    //                      tcp_addr->src_addrlen ());
    // #else
            rc = libc::bind (self._s, tcp_addr.src_addr (), tcp_addr.src_addrlen ());
    // #endif
            if (rc == -1) {
                return -1;
            }
        }

        //  Connect to the remote peer.
    // #if defined ZMQ_HAVE_VXWORKS
    //     rc = ::connect (_s, (sockaddr *) tcp_addr->addr (), tcp_addr->addrlen ());
    // #else
        rc = libc::connect (self._s, tcp_addr.addr (), tcp_addr.addrlen ());
    // #endif
        //  Connect was successful immediately.
        if (rc == 0) {
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

    zmq::fd_t zmq::tcp_connecter_t::connect ()
    {
        //  Async connect has finished. Check whether an Error occurred
        let mut err = 0;
    // #if defined ZMQ_HAVE_HPUX || defined ZMQ_HAVE_VXWORKS
    //     int len = sizeof err;
    // #else
        let mut len = 4;
    // #endif

        const int rc = getsockopt (_s, SOL_SOCKET, SO_ERROR,
                                   reinterpret_cast<char *> (&err), &len);

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
        self._s = retired_fd;
        return result;
    }

    // bool zmq::tcp_connecter_t::tune_socket (const fd_t fd_)
    pub unsafe fn tune_socket(&mut self, fd_: fd_t) -> bool
    {
        let rc = tune_tcp_socket (fd_)
                       | tune_tcp_keepalives (
                         fd_, self.options.tcp_keepalive, self.options.tcp_keepalive_cnt,
                         self.options.tcp_keepalive_idle, self.options.tcp_keepalive_intvl)
                       | tune_tcp_maxrt (fd_, options.tcp_maxrt);
        return rc == 0;
    }
}
