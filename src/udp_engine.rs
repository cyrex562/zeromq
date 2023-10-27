use std::ffi::c_void;
use std::intrinsics::size_of;
use std::mem::size_of_val;
use libc::{bind, recvfrom, sendto, sockaddr};
use windows::Win32::Networking::WinSock::{AF_INET, AF_INET6, INADDR_NONE, IP_ADD_MEMBERSHIP, IP_MULTICAST_IF, IP_MULTICAST_LOOP, IP_MULTICAST_TTL, IPPROTO_IP, IPPROTO_IPV6, IPPROTO_UDP, IPV6_ADD_MEMBERSHIP, IPV6_MULTICAST_IF, IPV6_MULTICAST_LOOP, setsockopt, SO_REUSEADDR, SOCK_DGRAM, SOCKADDR_IN, SOCKADDR_STORAGE, SOL_SOCKET, WSAEWOULDBLOCK, WSAGetLastError};
use crate::address::ZmqAddress;
use crate::defines::{MSG_MORE, RETIRED_FD, ZmqHandle};
use crate::endpoint::ZmqEndpointUriPair;
use crate::fd::fd_t;
use crate::i_engine::ErrorReason;
use crate::io_object::IoObject;
use crate::io_thread::ZmqIoThread;
use crate::ip::{open_socket, unblock_socket};
use crate::ip_resolver::ZmqIpAddress;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::session_base::ZmqSessionBase;
use crate::udp_address::UdpAddress;

pub struct ZmqUdpEngine<'a> {
    pub io_object: IoObject,
    // pub i_engine: dyn i_engine_t,
    pub _fd: fd_t,
    pub _session: &'a mut ZmqSessionBase<'a>,
    pub _handle: ZmqHandle,
    pub _address: ZmqAddress,
    pub _options: ZmqOptions,
    pub _raw_address: sockaddr_in,
    pub _out_address: &'a mut sockaddr,
    pub _out_address_len: usize,
    pub _out_buffer: Vec<u8>,
    pub _in_buffer: Vec<u8>,
    pub _send_enabled: bool,
    pub _recv_enabled: bool
}

impl ZmqUdpEngine {
    pub fn new(session: &'a mut ZmqSessionBase, handle: ZmqHandle, options: ZmqOptions) -> ZmqUdpEngine<'a> {
        ZmqUdpEngine {
            io_object: IoObject::new2(),
            // i_engine: i_engine_t::new(),
            _fd: fd_t::new(),
            _session: session,
            _handle: handle,
            _address: ZmqAddress::new(),
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

    pub unsafe fn init(&mut self, address_: ZmqAddress, send_: bool, recv_: bool) -> i32 {
        self._send_enabled = send_;
        self._recv_enabled = recv_;
        self._address = address_;

        self._fd = open_socket (self._address.resolved.udp_addr.family (), SOCK_DGRAM,
                           IPPROTO_UDP);
        if (self._fd == RETIRED_FD) {
            return -1;
        }

        unblock_socket (self._fd);

        return 0;
    }


    // void zmq::udp_engine_t::plug (io_thread_t *io_thread_, session_base_t *session_)
    pub unsafe fn plug(&mut self, io_thread_: &mut ZmqIoThread, session_: &mut ZmqSessionBase)
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
                // Error (ConnectionError);
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
            let mut any = ZmqIpAddress::any (bind_addr.family ());
            let mut real_bind_addr: &ZmqIpAddress;

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
                // Error (ProtocolError);
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
                // Error (ProtocolError);
                return;
            }

            if (multicast) {
                rc = rc | self.add_membership (self._fd, udp_addr);
            }
        }

        if (rc != 0) {
            // Error (ProtocolError);
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
    pub unsafe fn set_udp_multicast_iface(&mut self, s_: fd_t, is_ipv6_: bool, addr_: &ZmqAddress) -> i32
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
    pub unsafe fn add_membership(&mut self, s_: fd_t, addr_: &UdpAddress) -> i32
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


    // void zmq::udp_engine_t::Error (error_reason_t reason_)
    pub unsafe fn error(&mut self, reason_: ErrorReason)
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
        self.io_object.unplug ();

        // delete this;
    }

    // void zmq::udp_engine_t::sockaddr_to_msg (zmq::msg_t *msg_, const sockaddr_in *addr_)
    pub unsafe fn sockaddr_to_msg(&mut self, msg_: &mut ZmqMsg, addr_: &SOCKADDR_IN
    {
        let name = (addr_.sin_addr.to_string ());

        // char port[6];
        let mut port: String = String::new();
        // const int port_len =
        //   snprintf (port, 6, "%d", static_cast<int> (ntohs (addr_->sin_port)));
        port = addr_.sin_port.to_string();
        let port_len = port.len();
        // zmq_assert (port_len > 0 && port_len < 6);

        // const size_t name_len = strlen (name);
        let name_len = name.len();
        let size =  (name_len) + 1 /* colon */
                         + port_len + 1;                 //  terminating NUL
        let rc = msg_.init_size (size);
        // errno_assert (rc == 0);
        msg_.set_flags (MSG_MORE);

        //  use memcpy instead of strcpy/strcat, since this is more efficient when
        //  we already know the lengths, which we calculated above
        let address = (msg_.data_mut());
        libc::memcpy (address, name.as_ptr() as *const c_void, name_len);
        address = address.add(name_len);
        // *address++ = ':';
        address[0] = ':';
        address = address.add(1);
        libc::memcpy(address, port.as_ptr() as * const c_void, (port_len));
        address = address.add(port_len);
        // *address = 0;
    }

    // int zmq::udp_engine_t::resolve_raw_address (const char *name_, size_t length_)
    pub unsafe fn resolve_raw_address(&mut self, name_: &str, length_: usize) -> i32
    {
        // memset (&_raw_address, 0, sizeof _raw_address);
        libc::memset(&self._raw_address, 0, size_of::<SOCKADDR_IN>());

        // const char *delimiter = NULL;
        let mut delimiter: *const char = std::ptr::null();

        // Find delimiter, cannot use memrchr as it is not supported on windows
        if (length_ != 0) {
            let mut chars_left = (length_);
            let current_char = &name_[length_..];
            loop {

                // if (*(--current_char) == ':') {
                //     delimiter = current_char;
                //     break;
                // }
                todo!();
                chars_left -= 1;
                if chars_left == 0 {
                    break;
                }
                if chars_left == 0 { break; }
                chars_left -= 1;
            }
        }

        if (!delimiter) {
            // errno = EINVAL;
            return -1;
        }

        // const std::string addr_str (name_, delimiter - name_);

        // const std::string port_str (delimiter + 1, name_ + length_ - delimiter - 1);


        //  Parse the port number (0 is not a valid port).
        let port = u16::from_str_radix (port_str, 10).unwrap();
        if (port == 0) {
            // errno = EINVAL;
            return -1;
        }

        self._raw_address.sin_family = AF_INET;
        self._raw_address.sin_port = htons (port);
        self._raw_address.sin_addr.s_addr = inet_addr (addr_str.c_str ());

        if (self._raw_address.sin_addr.s_addr == INADDR_NONE) {
            // errno = EINVAL;
            return -1;
        }

        return 0;
    }


    // void zmq::udp_engine_t::out_event ()
    pub unsafe fn out_event(&mut self)
    {

        // msg_t group_msg;
        let mut group_msg == ZmqMsg::new();
        let mut rc = self._session.pull_msg (&group_msg);
        // errno_assert (rc == 0 || (rc == -1 && errno == EAGAIN));

        if (rc == 0) {
            // msg_t body_msg;
            let mut body_msg = ZmqMsg::new();
            rc = self._session.pull_msg (&body_msg);
            //  If there's a group, there should also be a body
            // errno_assert (rc == 0);

            let group_size = group_msg.size ();
            let body_size = body_msg.size ();
            // size_t size;
            let mut size = 0usize;

            if (self._options.raw_socket) {
                rc = self.resolve_raw_address ((group_msg.data_mut()),
                                               group_size);

                //  We discard the message if address is not valid
                if (rc != 0) {
                    rc = group_msg.close ();
                    // errno_assert (rc == 0);

                    rc = body_msg.close ();
                    // errno_assert (rc == 0);

                    return;
                }

                size = body_size;

                libc::memcpy (self._out_buffer, body_msg.data (), body_size);
            } else {
                size = group_size + body_size + 1;

                // TODO: check if larger than maximum size
                self._out_buffer[0] = (group_size);
               libc::memcpy (self._out_buffer + 1, group_msg.data_mut(), group_size);
                libc::memcpy (self._out_buffer + 1 + group_size, body_msg.data (), body_size);
            }

            rc = group_msg.close ();
            // errno_assert (rc == 0);

            body_msg.close ();
            // errno_assert (rc == 0);

    // #ifdef ZMQ_HAVE_WINDOWS
            #[cfg(target_os="windows")]
            {
                rc = sendto(self._fd, self._out_buffer, (size), 0, self._out_address,
                            self._out_address_len);
            }
    // #elif defined ZMQ_HAVE_VXWORKS
    //         rc = sendto (_fd, reinterpret_cast<caddr_t> (_out_buffer), size, 0,
    //                      (sockaddr *) _out_address, _out_address_len);
    // #else
            #[cfg(not(target_os="windows"))]
            {
                rc = sendto (self._fd, self._out_buffer, size, 0, self._out_address, self._out_address_len);
            }
    // #endif
            if (rc < 0) {
    // #ifdef ZMQ_HAVE_WINDOWS
                #[cfg(target_os="windows")]
                {
                    if (WSAGetLastError() != WSAEWOULDBLOCK) {
                        // assert_success_or_recoverable(_fd, rc);
                        // Error(ConnectionError);
                    }
                }
    // #endif
            }
        } else {
            self.reset_pollout (self._handle);
        }
    }

    // const zmq::endpoint_uri_pair_t &zmq::udp_engine_t::get_endpoint () const
    pub unsafe fn get_endpoint(&mut self) -> ZmqEndpointUriPair
    {
        return self._empty_endpoint;
    }

    // void zmq::udp_engine_t::restart_output ()
    pub unsafe fn restart_output(&mut self){
        //  If we don't support send we just drop all messages
        if (!self._send_enabled) {
            let mut msg: ZmqMsg = ZmqMsg::new();
            while (self._session.pull_msg (&msg) == 0)
                msg.close ();
        } else {
            self.set_pollout (self._handle);
            self.out_event ();
        }
    }

    // void zmq::udp_engine_t::in_event ()
    pub unsafe fn in_event(&mut self)
    {
        // sockaddr_storage in_address;
        let mut in_address = SOCKADDR_STORAGE::default();
        let in_addrlen = (size_of_val(&sockaddr_storage));

        let nbytes = recvfrom (self._fd, self._in_buffer, MAX_UDP_MSG, 0,
                    (&in_address), &in_addrlen);

        if (nbytes < 0) {
    // #ifdef ZMQ_HAVE_WINDOWS
            #[cfg(target_os="windows")]
            {
                if (WSAGetLastError() != WSAEWOULDBLOCK) {
                    assert_success_or_recoverable(_fd, nbytes);
                    error(connection_error);
                }
            }
    // #else
            #[cfg(not(target_os="windows"))]
            {
            if (nbytes != EWOULDBLOCK) {
                // assert_success_or_recoverable (_fd, nbytes); Error (ConnectionError);
            }
            }
    // #endif
            return;
        }

        let mut rc = 0i32;
        let mut body_size = 0u32;
        let mut body_offset = 0u32
        let mut msg = ZmqMsg::default();

        if (self._options.raw_socket) {
            // zmq_assert (in_address.ss_family == AF_INET);
            self.sockaddr_to_msg (&msg, (&in_address));

            body_size = nbytes;
            body_offset = 0;
        } else {
            // TODO in out_event, the group size is an *unsigned* char. what is
            // the maximum value?
            let group_buffer = self._in_buffer[1..];
            let group_size = self._in_buffer[0];

            rc = msg.init_size (group_size);
            // errno_assert (rc == 0);
            msg.set_flags (MSG_MORE);
            libc::memcpy (msg.data_mut(), group_buffer, group_size);

            //  This doesn't fit, just ignore
            if (nbytes - 1 < group_size) {
                return;
            }

            body_size = nbytes - 1 - group_size;
            body_offset = 1 + group_size;
        }
        // Push group description to session
        rc = self._session.push_msg (&mut msg);
        // errno_assert (rc == 0 || (rc == -1 && errno == EAGAIN));

        //  Group description message doesn't fit in the pipe, drop
        if (rc != 0) {
            rc = msg.close ();
            // errno_assert (rc == 0);

            self.reset_pollin (self._handle);
            return;
        }

        rc = msg.close ();
        // errno_assert (rc == 0);
        rc = msg.init_size (body_size);
        // errno_assert (rc == 0);
        libc::memcpy (msg.data_mut(), self._in_buffer[body_offset..], body_size);

        // Push message body to session
        rc = self._session.push_msg (&mut msg);
        // Message body doesn't fit in the pipe, drop and reset session state
        if (rc != 0) {
            rc = msg.close ();
            // errno_assert (rc == 0);

            self._session.reset ();
            self.reset_pollin (self._handle);
            return;
        }

        rc = msg.close ();
        // errno_assert (rc == 0);
        self._session.flush ();
    }

    // bool zmq::udp_engine_t::restart_input ()
    pub unsafe fn restart_input(&mut self) -> bool
    {
        if (self._recv_enabled) {
            self.set_pollin (self._handle);
            self.in_event ();
        }

        return true;
    }
}
