use std::mem::{size_of, size_of_val};
#[cfg(not(target_os="windows"))]
use libc::{sockaddr_in};

use libc::{c_int, size_t};
#[cfg(target_os = "windows")]
use windows::Win32::Networking::WinSock::{setsockopt, SOCKADDR_IN, SOCKET,sendto};


use crate::address::ip_address::ZmqIpAddress;
use crate::address::udp_address::UdpAddress;
use crate::address::ZmqAddress;
use crate::defines::{
    ZmqFd, ZmqIpMreq, ZmqIpv6Mreq, ZmqSockAddr, ZmqSockAddrIn, RETIRED_FD,
    ZMQ_MSG_MORE,
};
use crate::defines::{
    AF_INET, AF_INET6, INADDR_NONE, IPV6_ADD_MEMBERSHIP, IPV6_MULTICAST_IF, IPV6_MULTICAST_LOOP,
    IP_ADD_MEMBERSHIP, IP_MULTICAST_IF, IP_MULTICAST_LOOP, IP_MULTICAST_TTL, SOL_SOCKET,
    SO_REUSEADDR,
};
use crate::defines::{INADDR_ANY, IPPROTO_IP, IPPROTO_IPV6, IPPROTO_UDP, SOCK_DGRAM};
use crate::defines::err::ZmqError;
use crate::defines::err::ZmqError::EngineError;
use crate::endpoint::ZmqEndpointUriPair;
use crate::engine::ZmqEngine;
use crate::io::io_thread::ZmqIoThread;
use crate::ip::{bind_to_device, open_socket, unblock_socket};
use crate::msg::ZmqMsg;
use crate::net::platform_socket::{platform_bind, platform_recvfrom, platform_sendto, platform_setsockopt};
use crate::options::ZmqOptions;
use crate::session::ZmqSession;
use crate::utils::sock_utils::{zmq_ip_mreq_to_bytes, zmq_ipv6_mreq_to_bytes, zmq_sockaddr_to_sockaddr, zmq_sockaddr_to_wsa_sockaddr, zmq_sockaddrin_to_sockaddr};

// pub struct ZmqUdpEngine<'a> {
//     pub io_object: IoObject,
//     // pub i_engine: dyn i_engine_t,
//     pub fd: ZmqFd,
//     pub session: &'a mut ZmqSession<'a>,
//     pub handle: ZmqHandle,
//     pub address: ZmqAddress<'a>,
//     pub options: ZmqOptions,
//     pub raw_address: ZmqSockAddrIn,
//     pub out_address: ZmqSockAddr,
//     pub out_address_len: usize,
//     pub out_buffer: Vec<u8>,
//     pub in_buffer: Vec<u8>,
//     pub send_enabled: bool,
//     pub recv_enabled: bool
// }

// impl ZmqUdpEngine {
//     pub fn new(session: &'a mut ZmqSession, handle: ZmqHandle, options: ZmqOptions) -> ZmqUdpEngine<'a> {
//         ZmqUdpEngine {
//             io_object: IoObject::new2(),
//             // i_engine: i_engine_t::new(),
//             fd: fd_t::new(),
//             session: session,
//             handle: handle,
//             address: ZmqAddress::new(),
//             options: options,
//             raw_address: SOCKADDR_IN::default(),
//             out_address: sockaddr::new(),
//             out_address_len: 0,
//             out_buffer: Vec::new(),
//             in_buffer: Vec::new(),
//             send_enabled: false,
//             recv_enabled: false
//         }
//     }

pub fn udp_init(
    engine: &mut ZmqEngine,
    address: ZmqAddress,
    send: bool,
    recv: bool,
) -> Result<(), ZmqError> {
    engine.send_enabled = send;
    engine.recv_enabled = recv;
    engine.address = Some(address);
    engine.fd = open_socket(
        engine.address.unwrap().udp_addr.family(),
        SOCK_DGRAM,
        IPPROTO_UDP,
    )?;
    if engine.fd == RETIRED_FD {
        return Err(EngineError("open socket failed"));
    }

    unblock_socket(engine.fd)?;

    Ok(())
}

// void zmq::udp_engine_t::Plug (io_thread_t *io_thread_, session_base_t *session_)
pub fn udp_plug(
    options: &mut ZmqOptions,
    engine: &mut ZmqEngine,
    io_thread_: &mut ZmqIoThread,
    session_: &mut ZmqSession,
) -> Result<(),ZmqError> {
    // zmq_assert (!_plugged);
    engine.plugged = true;

    // zmq_assert (!_session);
    // zmq_assert (session_);
    engine.session = Some(session_);

    //  Connect to I/O threads poller object.
    engine.io_object.plug(io_thread_);
    engine.handle = engine.io_object.add_fd(engine.fd);

    let mut udp_addr = engine.address.unwrap().udp_addr;

    let mut rc = 0;

    // Bind the socket to a device if applicable
    if !options.bound_device.is_empty() {
        // rc = rc | engine.bind_to_device(engine.fd, &options.bound_device);
        bind_to_device(engine.fd, &options.bound_device)?;
        // if rc != 0 {
        //     // assert_success_or_recoverable (_fd, rc);
        //     // Error (ConnectionError);
        //     return;
        // }
    }

    if engine.send_enabled {
        if !options.raw_socket {
            let mut out = udp_addr.target_addr();
            engine.out_address = out.as_sockaddr().clone();
            engine.out_address_len = out.sockaddr_len();

            if out.is_multicast() {
                let is_ipv6 = (out.family() == AF_INET6);
                udp_set_udp_multicast_loop(options, engine,engine.fd, is_ipv6, options.multicast_loop)?;

                if options.multicast_hops > 0 {
                    udp_set_udp_multicast_ttl(options, engine,engine.fd, is_ipv6, options.multicast_hops)?;
                }

                udp_set_udp_multicast_iface(engine, engine.fd, is_ipv6, &mut udp_addr)?;
            }
        } else {
            engine.out_address = zmq_sockaddrin_to_sockaddr(&engine.raw_address);
            #[cfg(target_os="windows")]
            {
                engine.out_address_len = size_of::<SOCKADDR_IN>();
            }
            #[cfg(not(target_os="windows"))]
            {
                engine.out_address_len = size_of::<sockaddr_in>();
            }
        }
    }

    if engine.recv_enabled {
        udp_set_udp_reuse_address(engine, engine.fd, true)?;

        let mut bind_addr = udp_addr.bind_addr();
        let mut any = ZmqIpAddress::any(bind_addr.family())?;
        let mut real_bind_addr: &ZmqIpAddress;

        let multicast = udp_addr.is_mcast();

        if multicast {
            //  Multicast addresses should be allowed to Bind to more than
            //  one port as all ports should receive the message
            udp_set_udp_reuse_port(engine,engine.fd, true)?;

            //  In multicast we should Bind ANY and use the mreq struct to
            //  specify the interface
            any.set_port(bind_addr.port());

            real_bind_addr = &any;
        } else {
            real_bind_addr = bind_addr;
        }

        // if rc != 0 {
        //     // Error (ProtocolError);
        //     return;
        // }

        // #ifdef ZMQ_HAVE_VXWORKS
        //         rc = rc
        //              | Bind (_fd, (sockaddr *) real_bind_addr->as_sockaddr (),
        //                      real_bind_addr->sockaddr_len ());
        // #else
        // unsafe {
        //     // let sa = zmq_sockaddr_to_sockaddr(real_bind_addr.as_sockaddr());
        //     // rc = rc
        //     //     | bind(
        //     //         engine.fd as SOCKET,
        //     //         &sa,
        //     //         real_bind_addr.sockaddr_len() as c_int,
        //     //     );
        //
        // };

        platform_bind(engine.fd, real_bind_addr.as_sockaddr())?;

        // #endif
        // if rc != 0 {
        //     // assert_success_or_recoverable (_fd, rc);
        //     // Error (ProtocolError);
        //     return;
        // }

        if multicast {
            udp_add_membership(engine,engine.fd, &mut udp_addr)?;
        }
    }

    if rc != 0 {
        // Error (ProtocolError);
    } else {
        if engine.send_enabled {
            engine.io_object.set_pollout(engine.handle);
        }

        if engine.recv_enabled {
            engine.io_object.set_pollin(engine.handle);

            //  Call restart output to drop all join/leave commands
           udp_restart_output(options, engine);
        }
    }

    Ok(())
}

// int zmq::udp_engine_t::set_udp_multicast_loop (fd_t s_,
//                                            bool is_ipv6_,
//                                            bool loop_)
pub fn udp_set_udp_multicast_loop(
    options: &mut ZmqOptions,
    engine: &mut ZmqEngine,
    s_: ZmqFd,
    is_ipv6_: bool,
    loop_: bool,
) -> Result<(),ZmqError> {
    let mut level = 0i32;
    let mut optname = 0i32;

    if is_ipv6_ {
        level = IPPROTO_IPV6;
        optname = IPV6_MULTICAST_LOOP;
    } else {
        level = IPPROTO_IP;
        optname = IP_MULTICAST_LOOP;
    }

    let loop__: i32 = if loop_ { 1 } else { 0 };
    // engine.socket.unwrap().setsockopt(
    //     options,
    //     optname,
    //     &mut loop__.to_le_bytes(),
    //     size_of_val(&loop__)
    // )?;
    platform_setsockopt(s_, level, optname, &loop__.to_le_bytes(), size_of_val(&loop__) as i32)?; // TODO: check if this is correct
    // assert_success_or_recoverable (s_, rc);
    return Ok(());
}

// int zmq::udp_engine_t::set_udp_multicast_ttl (fd_t s_, bool is_ipv6_, int hops_)
pub fn udp_set_udp_multicast_ttl(
    options: &mut ZmqOptions,
    engine: &mut ZmqEngine,
    s_: ZmqFd,
    is_ipv6_: bool,
    hops_: i32,
) -> Result<(),ZmqError> {
    let mut level = 0i32;

    if is_ipv6_ {
        level = IPPROTO_IPV6;
    } else {
        level = IPPROTO_IP;
    }

    platform_setsockopt(s_, level, IP_MULTICAST_TTL, &hops_.to_le_bytes(), size_of_val(&hops_) as i32)?;

    // assert_success_or_recoverable (s_, rc);
    Ok(())
}

// int zmq::udp_engine_t::set_udp_multicast_iface (fd_t s_,
//                                             bool is_ipv6_,
//                                             const udp_address_t *addr_)
pub fn udp_set_udp_multicast_iface(
    engine: &mut ZmqEngine,
    fd: ZmqFd,
    is_ipv6: bool,
    addr: &mut UdpAddress,
) -> Result<(),ZmqError> {
    let mut rc = 0;

    if is_ipv6 {
        let mut bind_if = addr.bind_if();

        if bind_if > 0 {
            //  If a Bind interface is provided we tell the
            //  kernel to use it to send multicast packets
            // rc = setsockopt(
            //     fd,
            //     IPPROTO_IPV6,
            //     IPV6_MULTICAST_IF,
            //     Some(&bind_if.to_le_bytes()),
            // );
            platform_setsockopt(fd, IPPROTO_IPV6, IPV6_MULTICAST_IF, &bind_if.to_le_bytes(), 4)?;
        }
    } else {
        let bind_addr = addr.bind_addr().ipv4.sin_addr;

        if bind_addr != INADDR_ANY {
            // rc = setsockopt(
            //     fd,
            //     IPPROTO_IP,
            //     IP_MULTICAST_IF,
            //     Some(&bind_addr.to_le_bytes()),
            // );
            platform_setsockopt(fd, IPPROTO_IP, IP_MULTICAST_IF, &bind_addr.to_le_bytes(), 4)?;
        }
    }

    // assert_success_or_recoverable (s_, rc);
    return Ok(());
}

// int zmq::udp_engine_t::set_udp_reuse_address (fd_t s_, bool on_)
pub fn udp_set_udp_reuse_address(engine: &mut ZmqEngine, s_: ZmqFd, on_: bool) -> Result<(),ZmqError> {
    let on: i32 = if on_ { 1 } else { 0 };
    // let rc = setsockopt(s_, SOL_SOCKET as i32, SO_REUSEADDR, Some(&on.to_le_bytes()));
    platform_setsockopt(s_, SOL_SOCKET as i32, SO_REUSEADDR, &on.to_le_bytes(), 4)?;
    // assert_success_or_recoverable (s_, rc);
    Ok(())
}

// int zmq::udp_engine_t::set_udp_reuse_port (fd_t s_, bool on_)
pub fn udp_set_udp_reuse_port(engine: &mut ZmqEngine, s_: ZmqFd, on_: bool) -> Result<(),ZmqError> {
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
pub fn udp_add_membership(engine: &mut ZmqEngine, s_: ZmqFd, addr_: &mut UdpAddress) -> Result<(),ZmqError> {
    let mut mcast_addr = addr_.target_addr();

    if mcast_addr.family() == AF_INET {
        // struct ip_mreq mreq;
        let mut mreq = ZmqIpMreq::default();
        mreq.imr_multiaddr.s_addr = mcast_addr.ipv4.sin_addr;
        mreq.imr_interface.s_addr = addr_.bind_addr().ipv4.sin_addr;

        // rc = setsockopt(
        //     s_,
        //     IPPROTO_IP,
        //     IP_ADD_MEMBERSHIP,
        //     Some(&zmq_ip_mreq_to_bytes(&mreq)),
        // );
        platform_setsockopt(s_, IPPROTO_IP, IP_ADD_MEMBERSHIP, &zmq_ip_mreq_to_bytes(&mreq), size_of::<ZmqIpMreq>() as i32)?;
    } else if mcast_addr.family() == AF_INET6 {
        // struct ipv6_mreq mreq;
        let mut mreq = ZmqIpv6Mreq::default();
        let iface = addr_.bind_if();

        // zmq_assert (iface >= -1);

        mreq.ipv6mr_multiaddr
            .s6_addr
            .clone_from_slice(&mcast_addr.ipv6.sin6_addr);
        mreq.ipv6mr_interface = iface as u32;

        // rc = setsockopt(
        //     s_,
        //     IPPROTO_IPV6,
        //     IPV6_ADD_MEMBERSHIP,
        //     Some(&zmq_ipv6_mreq_to_bytes(&mreq)),
        // );
        platform_setsockopt(s_, IPPROTO_IPV6, IPV6_ADD_MEMBERSHIP, &zmq_ipv6_mreq_to_bytes(&mreq), size_of::<ZmqIpv6Mreq>() as i32)?;
    }

    // assert_success_or_recoverable (s_, rc);
    Ok(())
}

// void zmq::udp_engine_t::Error (error_reason_t reason_)
pub fn udp_error(engine: &mut ZmqEngine, reason_: &str) {
    // // zmq_assert (_session);
    // engine.session.engine_error (false, reason_);
    // engine.terminate ();
    todo!()
}

// void zmq::udp_engine_t::terminate ()
pub fn udp_terminate(engine: &mut ZmqEngine) {
    // zmq_assert (_plugged);
    engine.plugged = false;

    engine.io_object.rm_fd(engine.handle);

    //  Disconnect from I/O threads poller object.
    engine.io_object.unplug();

    // delete this;
}

// void zmq::udp_engine_t::sockaddr_to_msg (zmq::msg_t *msg_, const sockaddr_in *addr_)
pub fn udp_sockaddr_to_msg(
    engine: &mut ZmqEngine,
    msg_: &mut ZmqMsg,
    addr_: &ZmqSockAddrIn,
) {
    let name = (addr_.sin_addr.to_string());

    // char port[6];
    let mut port: String = String::new();
    // const int port_len =
    //   snprintf (port, 6, "%d", static_cast<int> (ntohs (addr_->sin_port)));
    port = addr_.sin_port.to_string();
    let port_len = port.len();
    // zmq_assert (port_len > 0 && port_len < 6);

    // const size_t name_len = strlen (name);
    let name_len = name.len();
    let size = (name_len) + 1 /* colon */ + port_len + 1; //  terminating NUL
    let rc = msg_.init_size(size);
    // errno_assert (rc == 0);
    msg_.set_flags(ZMQ_MSG_MORE);

    //  use memcpy instead of strcpy/strcat, since this is more efficient when
    //  we already know the lengths, which we calculated above
    let mut address = (msg_.data_mut());
    // libc::memcpy (address, name.as_ptr() as *const c_void, name_len);
    address.copy_from_slice(name.as_bytes());
    address = &mut address[name_len..];
    // *address++ = ':';
    address[0] = ':' as u8;
    address = &mut address[1..];
    // libc::memcpy(address, port.as_ptr() as * const c_void, (port_len));
    address.copy_from_slice(port.as_bytes());
    address = &mut address[port_len..];
    // address = address.add(port_len);
    // *address = 0;
}

// int zmq::udp_engine_t::resolve_raw_address (const char *name_, size_t length_)
pub fn udp_resolve_raw_address(engine: &mut ZmqEngine, name_: &str, length_: usize) -> Result<(),ZmqError> {
    let mut addr_str = String::new();
    let mut port_str = String::new();
    // memset (&_raw_address, 0, sizeof _raw_address);
    // libc::memset(&engine.raw_address, 0, size_of::<SOCKADDR_IN>());
    engine.raw_address = ZmqSockAddrIn::default();

    // const char *delimiter = NULL;
    let mut delimiter: *const char = std::ptr::null();

    // Find delimiter, cannot use memrchr as it is not supported on windows
    if length_ != 0 {
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
            if chars_left == 0 {
                break;
            }
            chars_left -= 1;
        }
    }

    if delimiter.is_null() {
        // errno = EINVAL;
        return Err(EngineError("EINVAL"));
    }

    // TODO get addr_str
    // const std::string addr_str (name_, delimiter - name_);

    // TODO get port_str
    // const std::string port_str (delimiter + 1, name_ + length_ - delimiter - 1);

    //  Parse the port number (0 is not a valid port).
    let port = u16::from_str_radix(&port_str, 10).unwrap();
    if port == 0 {
        // errno = EINVAL;
        return Err(EngineError("EINVAL"));
    }

    engine.raw_address.sin_family = AF_INET as u16;
    engine.raw_address.sin_port = (port.to_be());
    // TODO convert IPv4 CIDR string to u32
    // engine.raw_address.sin_addr.s_addr = inet_addr (addr_str.c_str ());

    if engine.raw_address.sin_addr == INADDR_NONE {
        // errno = EINVAL;
        return Err(EngineError("EINVAL"));
    }

    return Ok(());
}

// void zmq::udp_engine_t::out_event ()
pub fn udp_out_event(options: &ZmqOptions, engine: &mut ZmqEngine) -> Result<(),ZmqError> {
    // msg_t group_msg;
    let mut group_msg = ZmqMsg::default();
    let result = engine.session.unwrap().pull_msg(&mut group_msg);
    // errno_assert (rc == 0 || (rc == -1 && errno == EAGAIN));

    if result.is_ok() {
        // msg_t body_msg;
        let mut body_msg = ZmqMsg::default();
        engine.session.unwrap().pull_msg(&mut body_msg)?;
        //  If there's a group, there should also be a body
        // errno_assert (rc == 0);

        let group_size = group_msg.size();
        let body_size = body_msg.size();
        // size_t size;
        let mut size = 0usize;

        if options.raw_socket {

            let group_msg_str = unsafe{String::from_utf8_unchecked(group_msg.data.to_vec())};

            let result2 = udp_resolve_raw_address(
                engine,
                &group_msg_str,
                group_size
            );

            //  We discard the message if address is not valid
            if result2.is_err() {
                group_msg.close()?;
                // errno_assert (rc == 0);

                body_msg.close()?;
                // errno_assert (rc == 0);

                return Ok(());
            }

            size = body_size;

            // libc::memcpy (engine.out_buffer, body_msg.data (), body_size);
            engine.out_buffer.copy_from_slice(&body_msg.data);
        } else {
            size = group_size + body_size + 1;

            // TODO: check if larger than maximum size
            engine.out_buffer[0] = (group_size) as u8;
            // libc::memcpy (engine.out_buffer + 1, group_msg.data_mut(), group_size);
            let mut ptr = engine.out_buffer.as_mut_slice();
            ptr = &mut ptr[1..];
            ptr.copy_from_slice(&group_msg.data);
            ptr = &mut ptr[group_size..];
            ptr.copy_from_slice(&body_msg.data);
            // libc::memcpy (engine.out_buffer + 1 + group_size, body_msg.data (), body_size);
        }

        group_msg.close()?;
        // errno_assert (rc == 0);

        body_msg.close()?;
        // errno_assert (rc == 0);

        // #ifdef ZMQ_HAVE_WINDOWS
        #[cfg(target_os = "windows")]
        {
            let mut rc = 0i32;

            unsafe {
                rc = sendto(
                    SOCKET{0: engine.fd},
                    engine.out_buffer.as_mut_slice(),
                    0,
                    &zmq_sockaddr_to_wsa_sockaddr(&engine.out_address),
                    engine.out_address_len as c_int,
                );
            }
        }
        // #elif defined ZMQ_HAVE_VXWORKS
        //         rc = sendto (_fd, reinterpret_cast<caddr_t> (_out_buffer), size, 0,
        //                      (sockaddr *) _out_address, _out_address_len);
        // #else
        // #[cfg(not(target_os = "windows"))]
        // {
        //     rc = sendto(
        //         engine._fd,
        //         engine._out_buffer,
        //         size,
        //         0,
        //         engine._out_address,
        //         engine._out_address_len,
        //     );
        // }
        // // #endif
        // if rc < 0 {
        //     // #ifdef ZMQ_HAVE_WINDOWS
        //     #[cfg(target_os = "windows")]
        //     unsafe {
        //         if WSAGetLastError() != WSAEWOULDBLOCK {
        //             // assert_success_or_recoverable(_fd, rc);
        //             // Error(ConnectionError);
        //         }
        //     }
        //     // #endif
        // }
        platform_sendto(engine.fd, engine.out_buffer.as_mut_slice(), size, 0, &engine.out_address)?;
    } else {
        engine.io_object.reset_pollout(engine.handle);
    }

    Ok(())
}

// const zmq::endpoint_uri_pair_t &zmq::udp_engine_t::get_endpoint () const
pub fn udp_get_endpoint(engine: &mut ZmqEngine) -> ZmqEndpointUriPair {
    return engine.endpoint_uri_pair.unwrap();
}

// void zmq::udp_engine_t::restart_output ()
pub fn udp_restart_output(options: &ZmqOptions, engine: &mut ZmqEngine) -> Result<(),ZmqError> {
    //  If we don't support send we just drop all messages
    if !engine.send_enabled {
        let mut msg: ZmqMsg = ZmqMsg::default();
        while engine.session.unwrap().pull_msg(&mut msg).is_ok() {
            msg.close()?;
        }
    } else {
        engine.io_object.set_pollout(engine.handle);
        engine.out_event(options);
    }
    Ok(())
}

// void zmq::udp_engine_t::in_event ()
pub fn udp_in_event(options: &ZmqOptions, engine: &mut ZmqEngine) -> Result<(),ZmqError> {
    // sockaddr_storage in_address;
    // let mut in_address = SOCKADDR_STORAGE::default();
    let mut in_address = ZmqSockAddr::default();
    let mut in_addrlen = (size_of_val(&in_address)) as c_int;

    // let nbytes = unsafe {
    //     recvfrom(
    //         engine.fd as SOCKET,
    //         engine.in_buffer.as_ptr() as *mut libc::c_char,
    //         MAX_UDP_MSG as c_int,
    //         0,
    //         &mut zmq_sockaddr_to_sockaddr(&mut in_address),
    //         &mut in_addrlen as *mut c_int,
    //     )
    // };
    //
    // if nbytes < 0 {
    //     // #ifdef ZMQ_HAVE_WINDOWS
    //     #[cfg(target_os = "windows")]
    //     {
    //         unsafe {
    //             if WSAGetLastError() != WSAEWOULDBLOCK {
    //                 // assert_success_or_recoverable(_fd, nbytes);
    //                 // error(connection_error);
    //             }
    //         }
    //     }
    //     // #else
    //     #[cfg(not(target_os = "windows"))]
    //     {
    //         if (nbytes != EWOULDBLOCK) {
    //             // assert_success_or_recoverable (_fd, nbytes); Error (ConnectionError);
    //         }
    //     }
    //     // #endif
    //     return;
    // }
    let nbytes = platform_recvfrom(engine.fd, engine.in_buffer.as_mut_slice(), &mut in_address)?;

    let mut rc = 0i32;
    let mut body_size = 0u32;
    let mut body_offset = 0u32;
    let mut msg = ZmqMsg::default();

    if options.raw_socket {
        // zmq_assert (in_address.ss_family == AF_INET);
        engine.sockaddr_to_msg(&msg, (&in_address));

        body_size = nbytes as u32;
        body_offset = 0;
    } else {
        // TODO in out_event, the group size is an *unsigned* char. what is
        // the maximum value?
        let group_buffer = engine.in_buffer[1..];
        let group_size = engine.in_buffer[0];

        msg.init_size(group_size as size_t)?;
        // errno_assert (rc == 0);
        msg.set_flags(ZMQ_MSG_MORE);
        // libc::memcpy (msg.data_mut(), group_buffer, group_size);
        let mut ptr = msg.data_mut();
        ptr.copy_from_slice(&group_buffer);

        //  This doesn't fit, just ignore
        if nbytes - 1 < group_size as i32 {
            return;
        }

        body_size = (nbytes - 1 - group_size) as u32;
        body_offset = (1 + group_size) as u32;
    }
    // Push group description to session
    rc = engine.session.push_msg(&mut msg);
    // errno_assert (rc == 0 || (rc == -1 && errno == EAGAIN));

    //  Group description message doesn't fit in the pipe, drop
    if rc != 0 {
        msg.close()?;
        // errno_assert (rc == 0);

        engine.reset_pollin(engine.handle);
        return;
    }

    msg.close()?;
    // errno_assert (rc == 0);
    msg.init_size(body_size as size_t)?;
    // errno_assert (rc == 0);
    // libc::memcpy (msg.data_mut(), engine.in_buffer[body_offset..], body_size);
    let mut ptr = msg.data_mut();
    ptr.copy_from_slice(&engine.in_buffer[body_offset..]);

    // Push message body to session
    rc = engine.session.push_msg(&mut msg);
    // Message body doesn't fit in the pipe, drop and reset session state
    if rc != 0 {
        msg.close()?;
        // errno_assert (rc == 0);

        engine.session.reset();
        engine.reset_pollin(engine.handle);
        return;
    }

    msg.close()?;
    // errno_assert (rc == 0);
    engine.session.flush();
    Ok(())
}

// bool zmq::udp_engine_t::restart_input ()
pub fn restart_input(options: &ZmqOptions, engine: &mut ZmqEngine) -> bool {
    if engine.recv_enabled {
        engine.set_pollin(engine.handle);
        engine.in_event(options);
    }

    return true;
}
