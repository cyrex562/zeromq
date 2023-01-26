/*
    Copyright (c) 2007-2016 Contributors as noted in the AUTHORS file

    This file is part of libzmq, the ZeroMQ core engine in C++.

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

// #ifndef __ZMQ_ADDRESS_HPP_INCLUDED__
// #define __ZMQ_ADDRESS_HPP_INCLUDED__

// #include "fd.hpp"

// #include <string>

// #ifndef ZMQ_HAVE_WINDOWS
// #include <sys/socket.h>
// #else
// #include <ws2tcpip.h>
// #endif

// namespace zmq
// {
// class ctx_t;
// class tcp_address_t;
// class udp_address_t;
// class ws_address_t;
// #ifdef ZMQ_HAVE_WSS
// class wss_address_t;
// #endif
// #if defined ZMQ_HAVE_IPC
// class ipc_address_t;
// #endif
// #if defined ZMQ_HAVE_LINUX || defined ZMQ_HAVE_VXWORKS
// class tipc_address_t;
// #endif
// #if defined ZMQ_HAVE_VMCI
// class vmci_address_t;
// #endif

// namespace protocol_name
// {
pub const inproc: String = String::from("inproc");
pub const tcp: String = String::from("tcp");
pub const udp: String = String::from("udp");
// #ifdef ZMQ_HAVE_OPENPGM
pub const pgm: String = String::from("pgm");
pub const epgm: String = String::from("epgm");
// #endif
// #ifdef ZMQ_HAVE_NORM
pub const norm: String = String::from("norm");
// #endif
// #ifdef ZMQ_HAVE_WS
pub const ws: String = String::from("ws");
// #endif
// #ifdef ZMQ_HAVE_WSS
pub const wss: String = String::from("wss");
// #endif
// #if defined ZMQ_HAVE_IPC
pub const ipc: String = String::from("ipc");
// #endif
// #if defined ZMQ_HAVE_TIPC
pub const tipc: String = String::from("tipc");
// #endif
// #if defined ZMQ_HAVE_VMCI
pub const vmci: String = String::from("vmci");
// #endif
// }

pub union AddressTResolved {
    pub dummy: *mut c_void,
    pub tcp_addr: *mut tcp_address_t,
    pub udp_addr: *mut udp_address_t,
    pub ws_addr: *mut ws_address_t,
    pub wss_addr: *mut wss_address_t,
    pub ipc_addr: *mut ipc_address_t,
    pub tipc_addr: *mut tipc_address_t,
    pub vmci_addr: *mut vmci_address_t,
}

#[derive(Default,Debug,Clone)]
pub struct address_t
{
    // const std::string protocol;
    pub protocol: String,
    // const std::string address;
    pub address: String,
    // ctx_t *const parent;
    pub parent: *const ctx_t,
    //  Protocol specific resolved address
    //  All members must be pointers to allow for consistent initialization
    pub resolved: AddressTResolved,
}

impl address_t {
    // address_t (const std::string &protocol_,
    //     const std::string &address_,
    //     ctx_t *parent_);
    pub fn new(protocol_: &str, address: &str, parent: *mut ctx_t) -> Self {
        todo!()
    }

    // ~address_t ();

    // int to_string (std::string &addr_) const;
    pub fn to_string(&self, addr_: &str) -> i32 {
        todo!()
    }
}

// #if defined(ZMQ_HAVE_HPUX) || defined(ZMQ_HAVE_VXWORKS)                        \
//   || defined(ZMQ_HAVE_WINDOWS)
// typedef int zmq_socklen_t;
type zmq_socklen_t = i32;
// #else
// typedef socklen_t zmq_socklen_t;
// #endif

pub enum socket_end_t
{
    socket_end_local,
    socket_end_remote
}

// zmq_socklen_t
// get_socket_address (fd_t fd_, socket_end_t socket_end_, sockaddr_storage *ss_);
// template <typename T>
pub fn get_socket_name<T>(fd_: fd_t, socket_end_: socket_end_t) -> String
{
    // struct sockaddr_storage ss;
    let mut ss: sockaddr_storage = sockaddr_storage{};
    let mut sl: zmq_socklen_t = get_socket_address (fd_, socket_end_, &ss);
    if (sl == 0) {
        return String::empty();
    }

    // const T addr (reinterpret_cast<struct sockaddr *> (&ss), sl);
    let mut addr = address_t::new();
    // std::string address_string;
    let mut address_string: String = String::new();
    addr.to_string(address_string);
    return address_string;
}
// }

// #endif
