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

// #include "tipc_address.hpp"

// #if defined ZMQ_HAVE_TIPC

// #include "err.hpp"

use libc::{c_char, EINVAL, memcpy, memset, strchr, strncmp};
use windows::s;
use windows::Win32::Networking::WinSock::socklen_t;
use crate::address_family::AF_TIPC;
use crate::unix_sockaddr::sockaddr;

// #include <string>
// #include <sstream>
pub struct ZmqTipcAddress {
//
//     ZmqTipcAddress ();
//     ZmqTipcAddress (const sockaddr *sa, socklen_t sa_len);

    //  This function sets up the address "{type, lower, upper}" for TIPC transport
    // int resolve (name: &str);

    //  The opposite to resolve()
    // int to_string (std::string &addr_) const;

    // Handling different TIPC address types
    // bool is_service () const;
    // bool is_random () const;
    // void set_random ();

    // const sockaddr *addr () const;
    // socklen_t addrlen () const;

    //
    pub _random: bool,
    // struct sockaddr_tipc address;
    pub address: sockaddr_tipc,
}

impl ZmqTipcAddress {
    // ZmqTipcAddress::ZmqTipcAddress ()
    pub fn new() -> Self {
        // memset (&address, 0, sizeof address);
        // _random = false;
        Self {
            _random: false,
            address: sockaddr_tipc {
                family: 0,
                addrtype: 0,
                addr: 0,
                scope: 0,
            },
        }
    }

    // ZmqTipcAddress::ZmqTipcAddress (const sockaddr *sa_, socklen_t sa_len_)
    pub fn new2(sa_: &sockaddr, sa_len_: socklen_t) -> Self {
        // // zmq_assert (sa_ && sa_len_ > 0);
        //
        // memset (&address, 0, sizeof address);
        // if (sa_.sa_family == AF_TIPC)
        //     memcpy (&address, sa_, sa_len_);
        //
        // _random = false;
        Self {
            _random: false,
            address: sockaddr_tipc {
                family: sa_.sa_family,
                addrtype: 0,
                addr: sa_.sa_data,
                scope: 0,
            },
        }
    }

    // void ZmqTipcAddress::set_random ()
    // {
    //     _random = true;
    // }
    // bool ZmqTipcAddress::is_random () const
    // {
    //     return _random;
    // }
    // bool ZmqTipcAddress::is_service () const
    pub fn is_service(&mut self) -> bool {
        if self.address.addrtype == TIPC_ADDR_ID {
            return false;
        }

        return true;
    }

    pub fn resolve(&mut self, name: &str) -> i32 {
        let mut type_ = 0;
        let mut lower = 0;
        let mut upper = 0;
        let mut ref_ = 0;
        let mut z = 1;
        let mut c = 0;
        let mut n = 0;
        // char eof;
        let mut eof = 0u8;
        // const char *domain;
        let mut domain: *mut c_char;
        let mut res = 0;


        // if (strncmp (name, "<*>", 3) == 0)
        if name.contains("<*>") {
            set_random();
            address.family = AF_TIPC;
            address.addrtype = TIPC_ADDR_ID;
            address.addr.id.node = 0;
            address.addr.id.ref_ = 0;
            address.scope = 0;
            return 0;
        }

        // res = sscanf (name, "{%u,%u,%u}", &type, &lower, &upper);
        /* Fetch optional domain suffix. */
        // if ((domain = strchr (name, '@'))) {
        //     if (sscanf (domain, "@%u.%u.%u%c", &z, &c, &n, &eof) != 3)
        //         return EINVAL;
        // }
        if (res == 3) {
            if (type_ < TIPC_RESERVED_TYPES || upper < lower) {
                return EINVAL;
            }
            address.family = AF_TIPC;
            address.addrtype = TIPC_ADDR_NAMESEQ;
            address.addr.nameseq.type_ = type_;
            address.addr.nameseq.lower = lower;
            address.addr.nameseq.upper = upper;
            address.scope = TIPC_ZONE_SCOPE;
            return 0;
        }
        if (res == 2 && type_ > TIPC_RESERVED_TYPES) {
            address.family = AF_TIPC;
            address.addrtype = TIPC_ADDR_NAME;
            address.addr.name.name.type_ = type_;
            address.addr.name.name.instance = lower;
            address.addr.name.domain = tipc_addr(z, c, n);
            address.scope = 0;
            return 0;
        } else if (res == 0) {
            res = sscanf(name, "<%u.%u.%u:%u>", &z, &c, &n, &ref_);
            if (res == 4) {
                address.family = AF_TIPC;
                address.addrtype = TIPC_ADDR_ID;
                address.addr.id.node = tipc_addr(z, c, n);
                address.addr.id.ref_ = ref_;
                address.scope = 0;
                return 0;
            }
        }
        return EINVAL;
    }

    pub fn to_string(addr_: &mut str) -> i32 {
        // TODO
        // if (address.family != AF_TIPC) {
        //     addr_.clear ();
        //     return -1;
        // }
        // std::stringstream s;
        // if (address.addrtype == TIPC_ADDR_NAMESEQ
        //     || address.addrtype == TIPC_ADDR_NAME) {
        //     s << "tipc://"
        //       << "{" << address.addr.nameseq.type;
        //     s << ", " << address.addr.nameseq.lower;
        //     s << ", " << address.addr.nameseq.upper << "}";
        //     addr_ = s.str ();
        // } else if (address.addrtype == TIPC_ADDR_ID || is_random ()) {
        //     s << "tipc://"
        //       << "<" << tipc_zone (address.addr.id.node);
        //     s << "." << tipc_cluster (address.addr.id.node);
        //     s << "." << tipc_node (address.addr.id.node);
        //     s << ":" << address.addr.id.ref << ">";
        //     addr_ = s.str ();
        // } else {
        //     addr_.clear ();
        //     return -1;
        // }
        // return 0;
        todo!()
    }

    // const sockaddr *ZmqTipcAddress::addr () const
    // {
    //     return (sockaddr *) &address;
    // }

    // socklen_t ZmqTipcAddress::addrlen () const
    // {
    //     return  (sizeof address);
    // }
}

// #endif
