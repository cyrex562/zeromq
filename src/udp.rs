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
use crate::context::ZmqContext;
use crate::defines::{INADDR_ANY, MAX_UDP_MSG, retired_fd, ZmqHandle};
use crate::endpoint::EndpointUriPair;
use crate::engine_interface::ZmqEngineInterface;
use crate::fd::ZmqFileDesc;
use crate::io_object::ZmqIoObject;
use crate::ip::{assert_success_or_recoverable, bind_to_device, open_socket, unblock_socket};
use crate::mechanism::name_len;
use crate::message::{ZmqMessage, ZMQ_MSG_MORE};

use crate::pgm_receiver::_empty_endpoint;
use crate::session_base::ZmqSessionBase;
use crate::thread_context::ZmqThreadContext;
use crate::unix_sockaddr::{in6_addr, in_addr, sockaddr_in};

#[cfg(target_os = "linux")]
use libc::{
    in6_addr, in_addr, ip_mreq, ipv6_mreq, sockaddr_storage, socklen_t, INADDR_ANY, SO_REUSEPORT,
};

use libc::{
    atoi, bind, c_char, c_int, c_uint, memcpy, memset,
    recvfrom, sendto, setsockopt, sockaddr, EINVAL, EWOULDBLOCK,
};
use std::mem;
use std::net::SocketAddr;
use std::os::raw::c_void;
use std::ptr::null_mut;
use anyhow::bail;
use windows::Win32::Networking::WinSock::{htons, inet_addr, inet_ntoa, ntohs, WSAGetLastError, INADDR_NONE, IPPROTO_IP, IPPROTO_IPV6, IPPROTO_UDP, IPV6_ADD_MEMBERSHIP, IPV6_MULTICAST_IF, IPV6_MULTICAST_LOOP, IP_ADD_MEMBERSHIP, IP_MULTICAST_IF, IP_MULTICAST_LOOP, IP_MULTICAST_TTL, SOCK_DGRAM, SOL_SOCKET, SO_REUSEADDR, WSAEWOULDBLOCK, socklen_t, IPPROTO, IP_MREQ, IN_ADDR, IPV6_MREQ, SOCKADDR_STORAGE};
use crate::engine::ZmqEngine;


pub fn udp_plug(engine: &mut ZmqEngine, io_thread_: &mut ZmqThreadContext, session_: &mut ZmqSessionBase) {
    engine.plugged = true;
    engine.session = Some(session_);

    //  Connect to I/O threads poller object.
    engine.io_object.plug(io_thread_);
    engine.handle = engine.io_object.add_fd(engine.fd);

    // const UdpAddress *const udp_addr = address.resolved.udp_addr;
    let udp_addr = engine.address.resolved.udp_addr;
    let mut rc = 0;

    // Bind the socket to a device if applicable
    if !engine.options.bound_device.empty() {
        bind_to_device(engine.fd, engine.options.bound_device)?;
        if rc != 0 {
            assert_success_or_recoverable(engine.fd, rc);
            // error (connection_error);
            return;
        }
    }

    if engine.send_enabled {
        if !engine.options.raw_socket {
            let out = engine.address.target_addr();
            engine.out_address = out.as_sockaddr();
            engine.out_address_len = out.sockaddr_len();

            if out.is_multicast() {
                let is_ipv6 = (out.family() == AF_INET6);
                rc = rc
                    | udp_set_multicast_loop(engine.fd, is_ipv6, engine.options.multicast_loop);

                if engine.options.multicast_hops > 0 {
                    rc = rc
                        | udp_set_multicast_ttl(
                        engine.fd,
                        is_ipv6,
                        engine.options.multicast_hops,
                    );
                }

                rc = rc | udp_set_multicast_iface(engine, engine.fd, is_ipv6, udp_addr);
            }
        } else {
            // XXX fixme ?
            // TODO convert socketaddr to zmqaddress
            // engine.out_address = (engine.raw_address.clone());
            // engine_out_address_len = (mem::size_of::<sockaddr_in>());
        }
    }

    if engine.recv_enabled {
        rc = rc | udp_set_reuse_address(engine, engine.fd, true);

        let bind_addr = udp_addr.bind_addr();
        // let any = ip_addr_t::any (bind_addr.family ());
        let mut any = ZmqAddress::default();
        let mut real_bind_addr = ZmqAddress::default();

        let multicast = udp_addr.is_mcast();

        if multicast {
            //  Multicast addresses should be allowed to Bind to more than
            //  one port as all ports should receive the message
            rc = rc | udp_set_reuse_port(engine, engine.fd, true);

            //  In multicast we should Bind ANY and use the mreq struct to
            //  specify the interface
            any.set_port(bind_addr.port());

            real_bind_addr = any.clone();
        } else {
            real_bind_addr = bind_addr;
        }

        if rc != 0 {
            // error (protocol_error);
            return;
        }

        unsafe {
            rc = bind(
                engine.fd,
                real_bind_addr.as_sockaddr(),
                real_bind_addr.sockaddr_len(),
            );
        }
        // #endif
        if rc != 0 {
            assert_success_or_recoverable(engine.fd, rc);
            // error (connection_error);
            return;
        }

        if multicast {
            rc = rc | udp_add_membership(engine, engine.fd, udp_addr);
        }
    }

    if rc != 0 {
        // error (protocol_error);
    } else {
        if engine.send_enabled {
            engine.set_pollout();
        }

        if engine.recv_enabled {
            engine.set_pollin();

            //  Call restart output to drop all join/leave commands
            udp_restart_output(engine);
        }
    }
}


pub fn udp_set_multicast_loop(s_: ZmqFileDesc, is_ipv6_: bool, loop_in: bool) -> i32 {
    let mut level: IPPROTO;
    let mut optname: i32;

    if is_ipv6_ {
        level = IPPROTO_IPV6;
        optname = IPV6_MULTICAST_LOOP;
    } else {
        level = IPPROTO_IP;
        optname = IP_MULTICAST_LOOP;
    }

    // int loop = loop_ ? 1 : 0;
    let loop_ = if loop_in { 1 } else { 0 };
    let rc: i32 = unsafe { setsockopt(s_, level as c_int, optname, (&loop_ as *const c_char), 1) };
    assert_success_or_recoverable(s_, rc);
    return rc;
}

pub fn udp_set_multicast_ttl(s_: ZmqFileDesc, is_ipv6_: bool, hops_: i32) -> i32 {
    let mut level: IPPROTO = 0 as IPPROTO;

    if is_ipv6_ {
        level = IPPROTO_IPV6;
    } else {
        level = IPPROTO_IP;
    }

    let rc: i32 =
        unsafe { setsockopt(s_, level as c_int, IP_MULTICAST_TTL, (&hops_) as *const c_char, 4) };
    assert_success_or_recoverable(s_, rc);
    return rc;
}


pub fn udp_set_multicast_iface(
    engine: &mut ZmqEngine,
    s_: ZmqFileDesc,
    is_ipv6_: bool,
    addr: &ZmqAddress,
) -> i32 {
    let mut rc = 0;

    if is_ipv6_ {
        let bind_if = addr.bind_if();

        if bind_if > 0 {
            //  If a Bind interface is provided we tell the
            //  kernel to use it to send multicast packets
            unsafe {
                rc = setsockopt(s_, IPPROTO_IPV6 as c_int, IPV6_MULTICAST_IF, (&bind_if), 4);
            }
        }
    } else {
        let bind_addr = addr.bind_addr.ipv4.sin_addr;

        if bind_addr.s_addr != INADDR_ANY {
            unsafe {
                rc = setsockopt(s_, IPPROTO_IP as c_int, IP_MULTICAST_IF, (&bind_addr), 4);
            }
        }
    }

    assert_success_or_recoverable(s_, rc);
    return rc;
}


pub fn udp_set_reuse_address(engine: &mut ZmqEngine, s_: ZmqFileDesc, on_: bool) -> i32 {
    // int on = on_ ? 1 : 0;
    let on = if on_ { 1 } else { 0 };
    let rc: i32 =
        unsafe { setsockopt(s_, SOL_SOCKET, SO_REUSEADDR, (&on) as *const c_char, 1) };
    assert_success_or_recoverable(s_, rc);
    return rc;
}


pub fn udp_set_reuse_port(engine: &mut ZmqEngine, s_: ZmqFileDesc, on_: bool) -> i32 {
    // #ifndef SO_REUSEPORT
    //     return 0;
    // #else
    //     int on = on_ ? 1 : 0;
    #[cfg(target_os = "linux")]
    {
        let on = if on_ { 1 } else { 0 };
        let rc = unsafe { setsockopt(s_, SOL_SOCKET, SO_REUSEPORT, (&on) as *const c_char, 1) };
        assert_success_or_recoverable(s_, rc);
        return rc;
        // #endif
    }
    #[cfg(target_os = "windows")]
    {
        return 0;
    }
}


pub fn udp_add_membership(engine: &mut ZmqEngine, s_: ZmqFileDesc, addr_: &mut ZmqAddress) -> anyhow::Result<()> {
    let mut mcast_addr = addr_.target_addr();
    let mut rc = 0;

    if mcast_addr.family() == AF_INET {
        // struct  mreq;
        let mut mreq = IP_MREQ {
            imr_multiaddr: IN_ADDR { S_un: Default::default() },
            imr_interface: IN_ADDR { S_un: Default::default() },
        };
        mreq.imr_multiaddr = mcast_addr.ipv4.sin_addr;
        mreq.imr_interface = addr_.bind_addr().ipv4.sin_addr;

        unsafe {
            rc = setsockopt(
                s_,
                IPPROTO_IP as i32,
                IP_ADD_MEMBERSHIP,
                (&mreq) as *const c_char,
                mem::size_of::<IP_MREQ>() as c_int,
            );
        }
    } else if mcast_addr.family() == AF_INET6 {
        // struct ipv6_mreq mreq;
        let mut mreq: IPV6_MREQ = IPV6_MREQ::default();
        let iface: i32 = addr_.bind_if();

        // zmq_assert (iface >= -1);

        mreq.ipv6mr_multiaddr = mcast_addr.ipv6.sin6_addr;
        mreq.ipv6mr_interface = iface as c_uint;

        unsafe {
            rc = setsockopt(
                s_,
                IPPROTO_IPV6 as i32,
                IPV6_ADD_MEMBERSHIP,
                (&mreq) as *const c_char,
                mem::size_of::<IPV6_MREQ>() as c_int,
            );
        }
    }

    // assert_success_or_recoverable(s_, rc);
    Ok(())
}


pub fn udp_terminate(engine: &mut ZmqEngine) {
    // zmq_assert (_plugged);
    engine.plugged = false;

    engine.io_thread.rm_fd(engine.handle);

    //  Disconnect from I/O threads poller object.
    engine.io_object.unplug();

    // delete this;
}


pub fn udp_sockaddr_to_msg(
    engine: &mut ZmqEngine,
    msg: &mut ZmqMessage,
    sa_in: &sockaddr_in,
) -> anyhow::Result<()> {
    // const char *const name = inet_ntoa (addr_.sin_addr);
    let name = sa_in.sin_addr.to_string();

    // char port[6];
    let mut port: String = String::new();
    // let port_len: i32 =
    //   sprintf (port, "%d",  (ntohs (addr_.sin_port)));
    port = sa_in.sin_port.to_string();
    // zmq_assert (port_len > 0);

    // const size_t name_len = strlen (name);
    let size: i32 = (name.len() + 1 /* colon */ + port.len() + 1) as i32; //  terminating NUL
    msg.init_size(size as usize)?;
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

    Ok(())
}


pub fn udp_resolve_raw_address(engine: &mut ZmqEngine, name: &str, length_: usize) -> i32 {
    // // memset (&_raw_address, 0, sizeof _raw_address);
    // self.raw_address = None;
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


pub fn udp_out_event(engine: &mut ZmqEngine) -> anyhow::Result<()> {
    // ZmqMessage group_msg;
    let mut group_msg: ZmqMessage = Default::default();
    let rc = engine.session.pull_msg(&group_msg);
    // errno_assert (rc == 0 || (rc == -1 && errno == EAGAIN));

    if rc == 0 {
        // ZmqMessage body_msg;
        let mut body_msg: ZmqMessage = Default::default();
        rc = engine.session.pull_msg(&body_msg);
        //  If there's a group, there should also be a body
        // errno_assert (rc == 0);

        let group_size = group_msg.size();
        let body_size = body_msg.size();
        let mut size_: usize;

        if engine.options.raw_socket {
            rc = udp_resolve_raw_address(engine,
                                         &String::from_utf8(group_msg.data().to_vec())?,
                                         group_size);

            //  We discard the message if address is not valid
            if rc != 0 {
                rc = group_msg.close();
                // errno_assert (rc == 0);

                rc = body_msg.close();
                // errno_assert (rc == 0);

                return Ok(());
            }

            size_ = body_size;

            // TODO
            // memcpy (_out_buffer, body_msg.data (), body_size);
        } else {
            size_ = group_size + body_size + 1;

            // TODO: check if larger than maximum size
            engine.out_buffer[0] = (group_size) as u8;
            // TODO
            // memcpy (_out_buffer + 1, group_msg.data (), group_size);
            // memcpy (_out_buffer + 1 + group_size, body_msg.data (), body_size);
        }

        rc = group_msg.close();
        // errno_assert (rc == 0);

        body_msg.close()?;
        // errno_assert (rc == 0);

        // #ifdef ZMQ_HAVE_WINDOWS
        unsafe {
            rc = sendto(engine.fd, &engine.out_buffer as *const c_char, engine.out_buffer.len() as c_int, 0, &engine.out_address.to_sockaddr(), engine.out_address_len);
        }
        // #elif defined ZMQ_HAVE_VXWORKS
        //         unsafe {
        //             rc = sendto(_fd, _out_buffer, size, 0,
        //                          _out_address, _out_address_len);
        //         }
        // #else
        //         unsafe { rc = sendto(_fd, _out_buffer, size, 0, _out_address, _out_address_len); }
        // #endif
        if rc < 0 {
            // #ifdef ZMQ_HAVE_WINDOWS
            #[cfg(target_os = "windows")]
            unsafe {
                if WSAGetLastError() != WSAEWOULDBLOCK {
                    assert_success_or_recoverable(engine.fd, rc);
                    // error (connection_error);
                }
            }
            // #else
            #[cfg(target_os = "linux")]
            if rc != EWOULDBLOCK {
                assert_success_or_recoverable(_fd, rc);
                // error (connection_error);
            }
            // #endif
        }
    } else {
        engine.reset_pollout();
    }

    Ok(())
}


pub fn udp_get_endpoint(engine: &mut ZmqEngine) -> EndpointUriPair {
    return engine.empty_endpoint.clone();
}

pub fn udp_restart_output(engine: &mut ZmqEngine) -> anyhow::Result<()> {
    //  If we don't support send we just drop all messages
    if !engine.send_enabled {
        let mut msg = ZmqMessage::default();
        while engine.session.pull_msg(&msg) == 0 {
            msg.close()?;
        }
    } else {
        engine.io_thread.set_pollout(engine.handle);
        engine.out_event();
    }
    Ok(())
}


pub fn udp_restart_input(engine: &mut ZmqEngine) -> bool {
    if engine.recv_enabled {
        engine.io_thread.set_pollin(engine.handle);
        engine.in_event();
    }

    return true;
}

pub fn udp_init(engine: &mut ZmqEngine, address_: &mut ZmqAddress, send_: bool, recv_: bool) -> anyhow::Result<()> {
    // zmq_assert (address_);
    // zmq_assert (send_ || recv_);
    engine.send_enabled = send_;
    engine.recv_enabled = recv_;
    engine.address = address_.clone();

    engine.fd = open_socket(
        engine.address.resolved.udp_addr.family(),
        SOCK_DGRAM as i32,
        IPPROTO_UDP as i32,
    );
    if engine.fd == retired_fd as usize {
        bail!("failed to open socket")
    }

    unblock_socket(engine.fd);

    Ok(())
}

pub fn udp_in_event(engine: &mut ZmqEngine) -> anyhow::Result<()> {
    let mut in_address = SOCKADDR_STORAGE::default();
    let in_addrlen = (mem::size_of::<SOCKADDR_STORAGE>());

    let nbytes = unsafe {
        recvfrom(
            engine.fd,
            engine.in_buffer,
            MAX_UDP_MSG as c_int,
            0,
            (&mut in_address) as *mut SOCKADDR_STORAGE as *mut sockaddr,
            &mut (in_addrlen as c_int) as *mut c_int,
        )
    };

    if nbytes < 0 {
        #[cfg(target_os = "windows")]
        unsafe {
            if WSAGetLastError() != WSAEWOULDBLOCK {
                assert_success_or_recoverable(engine.fd, nbytes);
                // error (connection_error);
            }
        }
        #[cfg(target_os = "linux")]
        if nbytes != EWOULDBLOCK as isize {
            // assert_success_or_recoverable(_fd, nbytes);
            // error (connection_error);
        }
        return Ok(());
    }

    let mut rc = 0;
    let mut body_size = 0;
    let mut body_offset = 0;
    let mut msg = ZmqMessage::default();

    if engine.options.raw_socket {
        udp_sockaddr_to_msg(engine, &mut msg, (&in_address) as &sockaddr_in);
        body_size = nbytes;
        body_offset = 0;
    } else {
        // TODO in out_event, the group size is an *unsigned* char. what is
        // the maximum value?
        let group_buffer = engine.in_buffer + 1;
        let group_size: i32 = engine.in_buffer[0];

        msg.init_size(group_size as usize)?;
        // errno_assert (rc == 0);
        msg.set_flags(ZMQ_MSG_MORE);
        // TODO:
        // memcpy (msg.data (), group_buffer, group_size);

        //  This doesn't fit, just ignore
        if nbytes - 1 < group_size {
            return Ok(());
        }

        body_size = nbytes - 1 - group_size;
        body_offset = 1 + group_size;
    }
    // Push group description to session
    rc = engine.session.push_msg(&mut msg);
    // errno_assert (rc == 0 || (rc == -1 && errno == EAGAIN));

    //  Group description message doesn't fit in the pipe, drop
    if rc != 0 {
        msg.close()?;
        engine.io_thread.reset_pollin(engine.handle);
        return Ok(());
    }

    msg.close()?;
    // errno_assert (rc == 0);
    msg.init_size(body_size as usize)?;
    // errno_assert (rc == 0);
    // TODO:
    // memcpy (msg.data (), _in_buffer + body_offset, body_size);

    // Push message body to session
    engine.session.push_msg(&mut msg).map_err(|x| {
        msg.close();
        engine.session.reset();
        engine.io_thread.reset_pollin(engine.handle);
        return Ok(());
    })?;
    // Message body doesn't fit in the pipe, drop and reset session state

    msg.close()?;
    // errno_assert (rc == 0);
    self.session.flush()?;
}