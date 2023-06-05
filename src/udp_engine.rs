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

// #if !defined ZMQ_HAVE_WINDOWS
// #include <sys/types.h>
// #include <unistd.h>
// #include <sys/socket.h>
// #include <netinet/in.h>
// #include <arpa/inet.h>
// #ifdef ZMQ_HAVE_VXWORKS
// #include <sockLib.h>
// #endif
// #endif

// #include "udp_address.hpp"
// #include "udp_engine.hpp"
// #include "session_base.hpp"
// #include "err.hpp"
// #include "ip.hpp"

//  OSX uses a different name for this socket option
// #ifndef IPV6_ADD_MEMBERSHIP
// #define IPV6_ADD_MEMBERSHIP IPV6_JOIN_GROUP
// #endif

use crate::address::ZmqAddress;
use crate::address_family::{AF_INET, AF_INET6};
use crate::decoder_allocators::size;
use crate::defines::ZmqHandle;
use crate::endpoint::EndpointUriPair;
use crate::engine_interface::ZmqEngineInterface;
use crate::fd::ZmqFileDesc;
use crate::io_object::ZmqIoObject;
use crate::ip::{assert_success_or_recoverable, bind_to_device, open_socket, unblock_socket};
use crate::mechanism::name_len;
use crate::message::{ZmqMessage, ZMQ_MSG_MORE};
use crate::options::ZmqOptions;
use crate::pgm_receiver::_empty_endpoint;
use crate::session_base::ZmqSessionBase;
use crate::thread_context::ZmqThreadContext;
use crate::udp_address::UdpAddress;
use crate::unix_sockaddr::sockaddr_in;
use libc::{
    atoi, bind, c_char, c_int, memcpy, memset, recvfrom, sendto, setsockopt, sockaddr, EINVAL,
    EWOULDBLOCK,
};
use std::mem;
use std::net::SocketAddr;
use std::ptr::null_mut;
use windows::Win32::Networking::WinSock::{
    htons, inet_addr, inet_ntoa, ntohs, WSAGetLastError, INADDR_NONE, IPPROTO_IP, IPPROTO_IPV6,
    IPPROTO_UDP, IPV6_ADD_MEMBERSHIP, IPV6_MULTICAST_IF, IPV6_MULTICAST_LOOP, IP_ADD_MEMBERSHIP,
    IP_MULTICAST_IF, IP_MULTICAST_LOOP, IP_MULTICAST_TTL, SOCK_DGRAM, SOL_SOCKET, SO_REUSEADDR,
    WSAEWOULDBLOCK,
};

// #ifdef __APPLE__
// #include <TargetConditionals.h>
// #endif
pub struct ZmqUdpEngine {
    // : public ZmqIoObject, public ZmqEngineInterface
    pub io_object: ZmqIoObject,
    pub engine_interface: ZmqEngineInterface,

    //     ZmqUdpEngine (options: &ZmqOptions);
    //     ~ZmqUdpEngine ();

    // int init (Address *address_, send_: bool, recv_: bool);

    // bool has_handshake_stage ()  { return false; };

    //  ZmqIEngine interface implementation.
    //  Plug the engine to the session.
    // void plug (ZmqIoThread *io_thread_, class ZmqSessionBase *session_);

    //  Terminate and deallocate the engine. Note that 'detached'
    //  events are not fired on termination.
    // void terminate ();

    //  This method is called by the session to signalise that more
    //  messages can be written to the pipe.
    // bool restart_input ();

    //  This method is called by the session to signalise that there
    //  are messages to send available.
    // void restart_output ();

    // void zap_msg_available (){};

    // void in_event ();
    // void out_event ();

    // const endpoint_uri_ZmqPair &get_endpoint () const;

    //
    //   int resolve_raw_address (name: &str, length_: usize);
    //   static void sockaddr_to_msg (msg: &mut ZmqMessage const sockaddr_in *addr_);

    // static int set_udp_reuse_address (s_: ZmqFileDesc, on_: bool);
    // static int set_udp_reuse_port (s_: ZmqFileDesc, on_: bool);
    // Indicate, if the multicast data being sent should be looped back
    // static int set_udp_multicast_loop (s_: ZmqFileDesc, is_ipv6_: bool, loop_: bool);
    // Set multicast TTL
    // static int set_udp_multicast_ttl (s_: ZmqFileDesc, is_ipv6_: bool, hops_: i32);
    // Set multicast address/interface
    // int set_udp_multicast_iface (s_: ZmqFileDesc,
    //                              is_ipv6_: bool,
    //                              const UdpAddress *addr_);
    // Join a multicast group
    // int add_membership (s_: ZmqFileDesc, const UdpAddress *addr_);

    //  Function to handle network issues.
    // void // error (ZmqErrorReason reason_);

    // const endpoint_uri_ZmqPair _empty_endpoint;
    pub _empty_endpoint: EndpointUriPai,
    pub _plugged: bool,

    // ZmqFileDesc _fd;
    pub _fd: ZmqFileDesc,
    // ZmqSessionBase *_session;
    pub _session: ZmqSessionBase,
    // let mut _handle: ZmqHandle;
    pub _handle: ZmqHandle,
    // Address *address;
    pub address: UdpAddress,
    // ZmqOptions self._options;
    pub options: ZmqOptions,
    // sockaddr_in _raw_address;
    pub _raw_address: Option<SocketAddr>,
    // const struct sockaddr *_out_address;
    pub _out_address: Option<SocketAddr>,
    // ZmqSocklen _out_address_len;
    pub _out_address_len: usize,
    // char _out_buffer[MAX_UDP_MSG];
    pub _out_buffer: Vec<u8>,
    // char _in_buffer[MAX_UDP_MSG];
    pub _in_buffer: Vec<u8>,
    pub _send_enabled: bool,
    pub _recv_enabled: bool,
}

impl ZmqUdpEngine {
    // ZmqUdpEngine::ZmqUdpEngine (options: &ZmqOptions) :
    //     _plugged (false),
    //     _fd (-1),
    //     _session (null_mut()),
    //     _handle ( (null_mut())),
    //     address (null_mut()),
    //     self._options (options_),
    //     _send_enabled (false),
    //     _recv_enabled (false)
    // {
    // }
    pub fn new(options: &mut ZmqOptions) -> Self {
        Self {
            io_object: Default::default(),
            engine_interface: (),
            _empty_endpoint: (),
            _plugged: false,
            _fd: 0,
            _session: Default::default(),
            _handle: 0,
            address: Default::default(),
            options: Default::default(),
            _raw_address: None,
            _out_address: None,
            _out_address_len: 0,
            _out_buffer: vec![],
            _in_buffer: vec![],
            _send_enabled: false,
            _recv_enabled: false,
        }
    }

    // ZmqUdpEngine::~ZmqUdpEngine ()
    // {
    //     // zmq_assert (!_plugged);
    //
    //     if (_fd != retired_fd) {
    // // #ifdef ZMQ_HAVE_WINDOWS
    //         let rc: i32 = closesocket (_fd);
    //         wsa_assert (rc != SOCKET_ERROR);
    // // #else
    //         int rc = close (_fd);
    //         // errno_assert (rc == 0);
    // // #endif
    //         _fd = retired_fd;
    //     }
    // }

    pub fn init(&mut self, address_: &mut UdpAddress, send_: bool, recv_: bool) -> i32 {
        // zmq_assert (address_);
        // zmq_assert (send_ || recv_);
        _send_enabled = send_;
        _recv_enabled = recv_;
        address = address_;

        _fd = open_socket(
            address.resolved.udp_addr.family(),
            SOCK_DGRAM as i32,
            IPPROTO_UDP as i32,
        );
        if _fd == retired_fd {
            return -1;
        }

        unblock_socket(_fd);

        return 0;
    }

    pub fn plug(&mut self, io_thread_: &mut ZmqThreadContext, session_: &mut ZmqSessionBase) {
        // zmq_assert (!_plugged);
        _plugged = true;

        // zmq_assert (!_session);
        // zmq_assert (session_);
        _session = session_;

        //  Connect to I/O threads poller object.
        // ZmqIoObject::plug (io_thread_);
        // TODO:
        _handle = add_fd(_fd);

        // const UdpAddress *const udp_addr = address.resolved.udp_addr;

        let mut rc = 0;

        // Bind the socket to a device if applicable
        if !_options.bound_device.empty() {
            rc = rc | bind_to_device(_fd, self._options.bound_device);
            if (rc != 0) {
                assert_success_or_recoverable(_fd, rc);
                // error (connection_error);
                return;
            }
        }

        if (_send_enabled) {
            if (!_options.raw_socket) {
                let out = self.address.target_addr();
                _out_address = out.as_sockaddr();
                _out_address_len = out.sockaddr_len();

                if (out.is_multicast()) {
                    let is_ipv6 = (out.family() == AF_INET6);
                    rc = rc | set_udp_multicast_loop(_fd, is_ipv6, self._options.multicast_loop);

                    if (self._options.multicast_hops > 0) {
                        rc = rc | set_udp_multicast_ttl(_fd, is_ipv6, self._options.multicast_hops);
                    }

                    rc = rc | set_udp_multicast_iface(_fd, is_ipv6, udp_addr);
                }
            } else {
                /// XXX fixme ?
                _out_address = (&_raw_address);
                _out_address_len = (mem::size_of::<sockaddr_in>());
            }
        }

        if (_recv_enabled) {
            rc = rc | set_udp_reuse_address(_fd, true);

            let bind_addr = udp_addr.bind_addr();
            // let any = ip_addr_t::any (bind_addr.family ());
            // let real_bind_addr;

            let multicast = udp_addr.is_mcast();

            if (multicast) {
                //  Multicast addresses should be allowed to bind to more than
                //  one port as all ports should receive the message
                rc = rc | set_udp_reuse_port(_fd, true);

                //  In multicast we should bind ANY and use the mreq struct to
                //  specify the interface
                any.set_port(bind_addr.port());

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
            //              | bind (_fd, (sockaddr *) real_bind_addr.as_sockaddr (),
            //                      real_bind_addr.sockaddr_len ());
            // #else
            unsafe {
                rc = bind(
                    _fd,
                    real_bind_addr.as_sockaddr(),
                    real_bind_addr.sockaddr_len(),
                );
            }
            // #endif
            if (rc != 0) {
                assert_success_or_recoverable(_fd, rc);
                // error (connection_error);
                return;
            }

            if (multicast) {
                rc = rc | add_membership(_fd, udp_addr);
            }
        }

        if (rc != 0) {
            // error (protocol_error);
        } else {
            if (_send_enabled) {
                set_pollout(_handle);
            }

            if (_recv_enabled) {
                set_pollin(_handle);

                //  Call restart output to drop all join/leave commands
                restart_output();
            }
        }
    }

    pub fn set_udp_multicast_loop(s_: ZmqFileDesc, is_ipv6_: bool, loop_in: bool) -> i32 {
        level: i32;
        optname: i32;

        if (is_ipv6_) {
            level = IPPROTO_IPV6;
            optname = IPV6_MULTICAST_LOOP;
        } else {
            level = IPPROTO_IP;
            optname = IP_MULTICAST_LOOP;
        }

        // int loop = loop_ ? 1 : 0;
        let loop_ = if loop_in { 1 } else { 0 };
        let rc: i32 = unsafe { setsockopt(s_, level, optname, (&loop_), 1) };
        assert_success_or_recoverable(s_, rc);
        return rc;
    }

    pub fn set_udp_multicast_ttl(s_: ZmqFileDesc, is_ipv6_: bool, hops_: i32) -> i32 {
        level: i32;

        if (is_ipv6_) {
            level = IPPROTO_IPV6;
        } else {
            level = IPPROTO_IP;
        }

        let rc: i32 =
            unsafe { setsockopt(s_, level, IP_MULTICAST_TTL, (&hops_) as *mut c_char, 4) };
        assert_success_or_recoverable(s_, rc);
        return rc;
    }

    pub fn set_udp_multicast_iface(
        &mut self,
        s_: ZmqFileDesc,
        is_ipv6_: bool,
        addr: &UdpAddress,
    ) -> i32 {
        let mut rc = 0;

        if (is_ipv6_) {
            let bind_if = addr_.bind_if();

            if (bind_if > 0) {
                //  If a bind interface is provided we tell the
                //  kernel to use it to send multicast packets
                unsafe {
                    rc = setsockopt(s_, IPPROTO_IPV6 as c_int, IPV6_MULTICAST_IF, (&bind_if), 4);
                }
            }
        } else {
            let bind_addr = addr_.bind_addr().ipv4.sin_addr;

            if (bind_addr.s_addr != INADDR_ANY) {
                unsafe {
                    rc = setsockopt(s_, IPPROTO_IP as c_int, IP_MULTICAST_IF, (&bind_addr), 4);
                }
            }
        }

        assert_success_or_recoverable(s_, rc);
        return rc;
    }

    pub fn set_udp_reuse_address(&mut self, s_: ZmqFileDesc, on_: bool) -> i32 {
        // int on = on_ ? 1 : 0;
        let on = if on_ { 1 } else { 0 };
        let rc: i32 = unsafe { setsockopt(s_, SOL_SOCKET, SO_REUSEADDR, (&on), 1) };
        assert_success_or_recoverable(s_, rc);
        return rc;
    }

    pub fn set_udp_reuse_port(&mut self, s_: ZmqFileDesc, on_: bool) -> i32 {
        // #ifndef SO_REUSEPORT
        //     return 0;
        // #else
        //     int on = on_ ? 1 : 0;
        let on = if on_ { 1 } else { 0 };
        let rc = unsafe { setsockopt(s_, SOL_SOCKET, SO_REUSEPORT, (&on), 1) };
        assert_success_or_recoverable(s_, rc);
        return rc;
        // #endif
    }

    pub fn add_membership(&mut self, s_: ZmqFileDesc, addr_: &mut UdpAddress) -> i32 {
        let mcast_addr = addr_.target_addr();
        let mut rc = 0;

        if (mcast_addr.family() == AF_INET) {
            // struct  mreq;
            let mreq: ip_mreq = ip_mreq {};
            mreq.imr_multiaddr = mcast_addr.ipv4.sin_addr;
            mreq.imr_interface = addr_.bind_addr().ipv4.sin_addr;

            unsafe {
                rc = setsockopt(
                    s_,
                    IPPROTO_IP as i32,
                    IP_ADD_MEMBERSHIP,
                    (&mreq),
                    mem::size_of::<ip_mreq>() as c_int,
                );
            }
        } else if (mcast_addr.family() == AF_INET6) {
            // struct ipv6_mreq mreq;
            let mreq: ipv6_mreq = ipv6_mreq {};
            let iface: i32 = addr_.bind_if();

            // zmq_assert (iface >= -1);

            mreq.ipv6mr_multiaddr = mcast_addr.ipv6.sin6_addr;
            mreq.ipv6mr_interface = iface;

            unsafe {
                rc = setsockopt(
                    s_,
                    IPPROTO_IPV6 as i32,
                    IPV6_ADD_MEMBERSHIP,
                    (&mreq),
                    mem::size_of::<ipv6_mreq>() as c_int,
                );
            }
        }

        assert_success_or_recoverable(s_, rc);
        return rc;
    }

    // void ZmqUdpEngine::// error (ZmqErrorReason reason_)
    pub fn error(&mut self, reason_: ZmqErrorReason) {
        // zmq_assert (_session);
        self._session.engine_error(false, reason_);
        self.terminate();
    }

    pub fn terminate(&mut self) {
        // zmq_assert (_plugged);
        _plugged = false;

        rm_fd(_handle);

        //  Disconnect from I/O threads poller object.
        self.io_object.unplug();

        // delete this;
    }

    pub fn sockaddr_to_msg(&mut self, msg: &mut ZmqMessage, addr_: &sockaddr_in) {
        // const char *const name = inet_ntoa (addr_.sin_addr);
        let name = addr_.sin_addr.to_string();

        // char port[6];
        let mut port: String = String::new();
        // let port_len: i32 =
        //   sprintf (port, "%d",  (ntohs (addr_.sin_port)));
        port = addr_.sin_port.to_string();
        // zmq_assert (port_len > 0);

        // const size_t name_len = strlen (name);
        let size: i32 = (name.len()) + 1 /* colon */ + port_len + 1; //  terminating NUL
        let rc: i32 = msg.init_size(size as usize);
        // errno_assert (rc == 0);
        msg.set_flags(ZMQ_MSG_MORE);

        //  use memcpy instead of strcpy/strcat, since this is more efficient when
        //  we already know the lengths, which we calculated above
        // char *address =  (msg.data ());
        let mut address = msg.data_mut();
        // memcpy (address, name, name_len);
        // address += name_len;
        // *address+= 1 = ':';
        // memcpy (address, port,  (port_len));
        // address += port_len;
        // *address = 0;
        unsafe {
            address = format!("{}:{}", name, port).as_bytes_mut();
        }
    }

    pub fn resolve_raw_address(&mut self, name: &str, length_: usize) -> i32 {
        // // memset (&_raw_address, 0, sizeof _raw_address);
        // self._raw_address = None;
        // // const char *delimiter = null_mut();
        //
        // // Find delimiter, cannot use memrchr as it is not supported on windows
        // if (length_ != 0) {
        //     int chars_left =  (length_);
        //     const char *current_char = name + length_;
        //     do {
        //         if (*(--current_char) == ':') {
        //             delimiter = current_char;
        //             break;
        //         }
        //     } while (--chars_left != 0);
        // }
        //
        // if (!delimiter) {
        //     errno = EINVAL;
        //     return -1;
        // }
        //
        // const std::string addr_str (name, delimiter - name);
        // const std::string port_str (delimiter + 1, name + length_ - delimiter - 1);
        //
        // //  Parse the port number (0 is not a valid port).
        // const uint16_t port = static_cast<uint16_t> (atoi (port_str.c_str ()));
        // if (port == 0) {
        //     errno = EINVAL;
        //     return -1;
        // }
        //
        // _raw_address.sin_family = AF_INET;
        // _raw_address.sin_port = htons (port);
        // _raw_address.sin_addr.s_addr = inet_addr (addr_str.c_str ());
        //
        // if (_raw_address.sin_addr.s_addr == INADDR_NONE) {
        //     errno = EINVAL;
        //     return -1;
        // }
        //
        // return 0;
        todo!()
    }

    pub fn out_event(&mut self) {
        // ZmqMessage group_msg;
        let mut group_msg: ZmqMessage = Default::default();
        let rc = _session.pull_msg(&group_msg);
        // errno_assert (rc == 0 || (rc == -1 && errno == EAGAIN));

        if (rc == 0) {
            // ZmqMessage body_msg;
            let mut body_msg: ZmqMessage = Default::default();
            rc = _session.pull_msg(&body_msg);
            //  If there's a group, there should also be a body
            // errno_assert (rc == 0);

            let group_size = group_msg.size();
            let body_size = body_msg.size();
            let mut size_: usize;

            if (self._options.raw_socket) {
                rc = resolve_raw_address((group_msg.data()), group_size);

                //  We discard the message if address is not valid
                if (rc != 0) {
                    rc = group_msg.close();
                    // errno_assert (rc == 0);

                    rc = body_msg.close();
                    // errno_assert (rc == 0);

                    return;
                }

                size_ = body_size;

                // TODO
                // memcpy (_out_buffer, body_msg.data (), body_size);
            } else {
                size_ = group_size + body_size + 1;

                // TODO: check if larger than maximum size
                _out_buffer[0] = (group_size);
                // TODO
                // memcpy (_out_buffer + 1, group_msg.data (), group_size);
                // memcpy (_out_buffer + 1 + group_size, body_msg.data (), body_size);
            }

            rc = group_msg.close();
            // errno_assert (rc == 0);

            body_msg.close();
            // errno_assert (rc == 0);

            // #ifdef ZMQ_HAVE_WINDOWS
            unsafe {
                rc = sendto(
                    _fd,
                    _out_buffer,
                    (size_) as c_int,
                    0,
                    _out_address,
                    _out_address_len,
                );
            }
            // #elif defined ZMQ_HAVE_VXWORKS
            //         unsafe {
            //             rc = sendto(_fd, _out_buffer, size, 0,
            //                          _out_address, _out_address_len);
            //         }
            // #else
            //         unsafe { rc = sendto(_fd, _out_buffer, size, 0, _out_address, _out_address_len); }
            // #endif
            if (rc < 0) {
                // #ifdef ZMQ_HAVE_WINDOWS
                unsafe {
                    if (WSAGetLastError() != WSAEWOULDBLOCK) {
                        assert_success_or_recoverable(_fd, rc);
                        // error (connection_error);
                    }
                }
                // #else
                if (rc != EWOULDBLOCK) {
                    assert_success_or_recoverable(_fd, rc);
                    // error (connection_error);
                }
                // #endif
            }
        } else {
            reset_pollout(_handle);
        }
    }

    pub fn get_endpoint() -> EndpointUriPair {
        return _empty_endpoint;
    }

    pub fn restart_output(&mut self) {
        //  If we don't support send we just drop all messages
        if (!_send_enabled) {
            let mut msg = ZmqMessage::default();
            while (_session.pull_msg(&msg) == 0) {
                msg.close();
            }
        } else {
            set_pollout(_handle);
            out_event();
        }
    }

    pub fn in_event(&mut self) {
        let in_address: sockaddr_storage = sockaddr_storage::default();
        let in_addrlen = (mem::size_of::<sockaddr_storage>());

        let nbytes: i32 = unsafe {
            recvfrom(
                _fd,
                _in_buffer,
                MAX_UDP_MSG,
                0,
                (&mut in_address),
                &mut (in_addrlen as c_int),
            )
        };

        if (nbytes < 0) {
            // #ifdef ZMQ_HAVE_WINDOWS
            unsafe {
                if (WSAGetLastError() != WSAEWOULDBLOCK) {
                    assert_success_or_recoverable(_fd, nbytes);
                    // error (connection_error);
                }
            }
            // #else
            if (nbytes != EWOULDBLOCK) {
                assert_success_or_recoverable(_fd, nbytes);
                // error (connection_error);
            }
            // #endif
            return;
        }

        rc: i32;
        body_size: i32;
        body_offset: i32;
        let mut msg = ZmqMessage::default();

        if (self._options.raw_socket) {
            // zmq_assert (in_address.ss_family == AF_INET);
            sockaddr_to_msg(&msg, (&in_address));

            body_size = nbytes;
            body_offset = 0;
        } else {
            // TODO in out_event, the group size is an *unsigned* char. what is
            // the maximum value?
            let group_buffer = _in_buffer + 1;
            let group_size: i32 = _in_buffer[0];

            rc = msg.init_size(group_size as usize);
            // errno_assert (rc == 0);
            msg.set_flags(ZMQ_MSG_MORE);
            // TODO:
            // memcpy (msg.data (), group_buffer, group_size);

            //  This doesn't fit, just ignore
            if (nbytes - 1 < group_size) {
                return;
            }

            body_size = nbytes - 1 - group_size;
            body_offset = 1 + group_size;
        }
        // Push group description to session
        rc = _session.push_msg(&msg);
        // errno_assert (rc == 0 || (rc == -1 && errno == EAGAIN));

        //  Group description message doesn't fit in the pipe, drop
        if (rc != 0) {
            rc = msg.close();
            // errno_assert (rc == 0);

            reset_pollin(_handle);
            return;
        }

        rc = msg.close();
        // errno_assert (rc == 0);
        rc = msg.init_size(body_size);
        // errno_assert (rc == 0);
        // TODO:
        // memcpy (msg.data (), _in_buffer + body_offset, body_size);

        // Push message body to session
        rc = _session.push_msg(&msg);
        // Message body doesn't fit in the pipe, drop and reset session state
        if (rc != 0) {
            rc = msg.close();
            // errno_assert (rc == 0);

            _session.reset();
            reset_pollin(_handle);
            return;
        }

        rc = msg.close();
        // errno_assert (rc == 0);
        _session.flush();
    }

    pub fn restart_input(&mut self) -> bool {
        if (_recv_enabled) {
            set_pollin(_handle);
            in_event();
        }

        return true;
    }
}
