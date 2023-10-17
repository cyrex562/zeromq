use std::intrinsics::size_of;
use std::mem::size_of_val;
use libc::{bind, sockaddr};
use windows::Win32::Networking::WinSock::{AF_INET, AF_INET6, IP_ADD_MEMBERSHIP, IP_MULTICAST_IF, IP_MULTICAST_LOOP, IP_MULTICAST_TTL, IPPROTO_IP, IPPROTO_IPV6, IPPROTO_UDP, IPV6_ADD_MEMBERSHIP, IPV6_MULTICAST_IF, IPV6_MULTICAST_LOOP, setsockopt, SO_REUSEADDR, SOCK_DGRAM, SOCKADDR_IN, SOL_SOCKET};
use crate::address::address_t;
use crate::defines::handle_t;
use crate::fd::{fd_t, retired_fd};
use crate::i_engine::error_reason_t;
use crate::io_object::io_object_t;
use crate::io_thread::io_thread_t;
use crate::ip::{open_socket, unblock_socket};
use crate::ip_resolver::ip_addr_t;
use crate::options::options_t;
use crate::session_base::session_base_t;
use crate::udp_address::udp_address_t;

pub struct udp_engine_t<'a> {
    pub io_object: io_object_t,
    // pub i_engine: dyn i_engine_t,
    pub _fd: fd_t,
    pub _session: &'a mut session_base_t<'a>,
    pub _handle: handle_t,
    pub _address: address_t,
    pub _options: options_t,
    pub _raw_address: sockaddr_in,
    pub _out_address: &'a mut sockaddr,
    pub _out_address_len: usize,
    pub _out_buffer: Vec<u8>,
    pub _in_buffer: Vec<u8>,
    pub _send_enabled: bool,
    pub _recv_enabled: bool
}

impl udp_engine_t {
    pub fn new(session: &'a mut session_base_t, handle: handle_t, options: options_t) -> udp_engine_t<'a> {
        udp_engine_t {
            io_object: io_object_t::new2(),
            // i_engine: i_engine_t::new(),
            _fd: fd_t::new(),
            _session: session,
            _handle: handle,
            _address: address_t::new(),
            _options: options,
            _raw_address: SOCKADDR_IN::default(),
            _out_address: sockaddr::new(),
            _out_address_len: 0,
            _out_buffer: Vec::new(),
            _in_buffer: Vec::new(),
            _send_enabled: false,
            _recv_enabled: false
        }
    }

    pub unsafe fn init(&mut self, address_: address_t, send_: bool, recv_: bool) -> i32 {
        self._send_enabled = send_;
        self._recv_enabled = recv_;
        self._address = address_;

        self._fd = open_socket (self._address.resolved.udp_addr.family (), SOCK_DGRAM,
                           IPPROTO_UDP);
        if (self._fd == retired_fd) {
            return -1;
        }

        unblock_socket (self._fd);

        return 0;
    }


    // void zmq::udp_engine_t::plug (io_thread_t *io_thread_, session_base_t *session_)
    pub unsafe fn plug(&mut self, io_thread_: &mut io_thread_t, session_: &mut session_base_t)
    {
        // zmq_assert (!_plugged);
        self._plugged = true;

        // zmq_assert (!_session);
        // zmq_assert (session_);
        self._session = session_;

        //  Connect to I/O threads poller object.
        self.io_object.plug (io_thread_);
        self._handle = self.add_fd (self._fd);

        let udp_addr = self._address.resolved.udp_addr;

        let mut rc = 0;

        // Bind the socket to a device if applicable
        if (!self._options.bound_device.empty ()) {
            rc = rc | self.bind_to_device (self._fd, self._options.bound_device);
            if (rc != 0) {
                // assert_success_or_recoverable (_fd, rc);
                // error (connection_error);
                return;
            }
        }

        if (self._send_enabled) {
            if (!self._options.raw_socket) {
                let out = udp_addr.target_addr ();
                self._out_address = out.as_sockaddr ();
                self._out_address_len = out.sockaddr_len ();

                if (out.is_multicast ()) {
                    let is_ipv6 = (out.family () == AF_INET6);
                    rc = rc
                         | self.set_udp_multicast_loop (self._fd, is_ipv6,
                                                   self._options.multicast_loop);

                    if (self._options.multicast_hops > 0) {
                        rc = rc
                             | self.set_udp_multicast_ttl (self._fd, is_ipv6,
                                                      self._options.multicast_hops);
                    }

                    rc = rc | self.set_udp_multicast_iface (self._fd, is_ipv6, udp_addr);
                }
            } else {
                /// XXX fixme ?
                self._out_address = (&self._raw_address);
                self._out_address_len = size_of::<SOCKADDR_IN>();
            }
        }

        if (self._recv_enabled) {
            rc = rc | self.set_udp_reuse_address (self._fd, true);

            let mut bind_addr = udp_addr.bind_addr ();
            let mut any = ip_addr_t::any (bind_addr.family ());
            let mut real_bind_addr: &ip_addr_t;

            let multicast = udp_addr.is_mcast ();

            if (multicast) {
                //  Multicast addresses should be allowed to bind to more than
                //  one port as all ports should receive the message
                rc = rc | self.set_udp_reuse_port (_fd, true);

                //  In multicast we should bind ANY and use the mreq struct to
                //  specify the interface
                any.set_port (bind_addr.port ());

                real_bind_addr = &any;
            } else {
                real_bind_addr = bind_addr;
            }

            if (rc != 0) {
                // error (protocol_error);
                return;
            }

    // #ifdef ZMQ_HAVE_VXWORKS
    //         rc = rc
    //              | bind (_fd, (sockaddr *) real_bind_addr->as_sockaddr (),
    //                      real_bind_addr->sockaddr_len ());
    // #else
            rc = rc
                 | bind (self._fd, real_bind_addr.as_sockaddr (),
                         real_bind_addr.sockaddr_len ());
    // #endif
            if rc != 0 {
                // assert_success_or_recoverable (_fd, rc);
                // error (protocol_error);
                return;
            }

            if (multicast) {
                rc = rc | self.add_membership (self._fd, udp_addr);
            }
        }

        if (rc != 0) {
            // error (protocol_error);
        } else {
            if (self._send_enabled) {
                self.set_pollout (self._handle);
            }

            if (self._recv_enabled) {
                self.set_pollin (self._handle);

                //  Call restart output to drop all join/leave commands
                self.restart_output ();
            }
        }
    }

    // int zmq::udp_engine_t::set_udp_multicast_loop (fd_t s_,
    //                                            bool is_ipv6_,
    //                                            bool loop_)
    pub unsafe fn set_udp_multicast_loop(&mut self, s_: fd_t, is_ipv6_: bool, loop_: bool) -> i32
    {
        let mut level = 0i32;
        let mut optname = 0i32;

        if (is_ipv6_) {
            level = IPPROTO_IPV6;
            optname = IPV6_MULTICAST_LOOP;
        } else {
            level = IPPROTO_IP;
            optname = IP_MULTICAST_LOOP;
        }

        let mut loop__ = if loop_ { 1 }else { 0 };
        let rc = self.setsockopt (s_, level, optname, (&loop__), size_of_val(&loop__));
        // assert_success_or_recoverable (s_, rc);
        return rc;

    }

    // int zmq::udp_engine_t::set_udp_multicast_ttl (fd_t s_, bool is_ipv6_, int hops_)
    pub unsafe fn set_udp_multicast_ttl(&mut self, s_: fd_t, is_ipv6_: bool, hops_: i32) -> i32
    {
        let mut level = 0i32;;

        if (is_ipv6_) {
            level = IPPROTO_IPV6;
        } else {
            level = IPPROTO_IP;
        }

        let rc =
          self.setsockopt (s_, level, IP_MULTICAST_TTL,
                      (&hops_), size_of_val(&hops_));
        // assert_success_or_recoverable (s_, rc);
        return rc;
    }

    // int zmq::udp_engine_t::set_udp_multicast_iface (fd_t s_,
    //                                             bool is_ipv6_,
    //                                             const udp_address_t *addr_)
    pub unsafe fn set_udp_multicast_iface(&mut self, s_: fd_t, is_ipv6_: bool, addr_: &address_t) -> i32
    {
        let mut rc = 0;

        if (is_ipv6_) {
            let mut bind_if = addr_.bind_if ();

            if (bind_if > 0) {
                //  If a bind interface is provided we tell the
                //  kernel to use it to send multicast packets
                rc = setsockopt (s_, IPPROTO_IPV6, IPV6_MULTICAST_IF,
                                 (&bind_if),
                                 size_of_val (bind_if));
            }
        } else {
            let bind_addr = addr_.bind_addr().ipv4.sin_addr;

            if (bind_addr.s_addr != INADDR_ANY) {
                rc = setsockopt (s_, IPPROTO_IP, IP_MULTICAST_IF,
                                 (&bind_addr),
                                 size_of_val(&bind_addr));
            }
        }

        // assert_success_or_recoverable (s_, rc);
        return rc;
    }

    // int zmq::udp_engine_t::set_udp_reuse_address (fd_t s_, bool on_)
    pub unsafe fn set_udp_reuse_address(&mut self, s_: fd_t, on_: bool) -> i32{
        let on = if on_ { 1 } else { 0 };
        let rc = setsockopt (s_, SOL_SOCKET, SO_REUSEADDR,
                                   (&on), size_of_val(&on));
        // assert_success_or_recoverable (s_, rc);
        return rc;
    }

    // int zmq::udp_engine_t::set_udp_reuse_port (fd_t s_, bool on_)
    pub unsafe fn set_udp_reuse_port(&mut self, s_: fd_t, on_: bool) -> i32
    {
        todo!()
    // #ifndef SO_REUSEPORT
    //     return 0;
    // #else
    //     int on = on_ ? 1 : 0;
    //     int rc = setsockopt (s_, SOL_SOCKET, SO_REUSEPORT,
    //                          reinterpret_cast<char *> (&on), sizeof (on));
    //     assert_success_or_recoverable (s_, rc);
    //     return rc;
    // #endif
    }

    // int zmq::udp_engine_t::add_membership (fd_t s_, const udp_address_t *addr_)
    pub unsafe fn add_membership(&mut self, s_: fd_t, addr_: &udp_address_t) -> i32
    {
        let mcast_addr = addr_.target_addr ();
        let mut rc = 0;

        if (mcast_addr.family () == AF_INET) {
            // struct ip_mreq mreq;
            let mut mreq = ip_mreq::default();
            mreq.imr_multiaddr = mcast_addr.ipv4.sin_addr;
            mreq.imr_interface = addr_.bind_addr ().ipv4.sin_addr;

            rc = setsockopt (s_, IPPROTO_IP, IP_ADD_MEMBERSHIP,
                             (&mreq), size_of_val (&mreq));

        } else if (mcast_addr.family () == AF_INET6) {
            // struct ipv6_mreq mreq;
            let mreq = ipv6_mreq::default();
            let iface = addr_.bind_if ();

            // zmq_assert (iface >= -1);

            mreq.ipv6mr_multiaddr = mcast_addr->ipv6.sin6_addr;
            mreq.ipv6mr_interface = iface;

            rc = setsockopt (s_, IPPROTO_IPV6, IPV6_ADD_MEMBERSHIP,
                             (&mreq), size_of_val(&mreq));
        }

        // assert_success_or_recoverable (s_, rc);
        return rc;
    }


    // void zmq::udp_engine_t::error (error_reason_t reason_)
    pub unsafe fn error(&mut self, reason_: error_reason_t)
    {
        // zmq_assert (_session);
        self._session.engine_error (false, reason_);
        self.terminate ();
    }

    // void zmq::udp_engine_t::terminate ()
    pub unsafe fn terminate(&mut self)
    {
        // zmq_assert (_plugged);
        self._plugged = false;

        self.rm_fd (self._handle);

        //  Disconnect from I/O threads poller object.
        io_object_t::unplug ();

        delete this;
    }
}
