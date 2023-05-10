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
// #include "compat.hpp"
// #include "ipc_address.hpp"

// #if defined ZMQ_HAVE_IPC

// #include "err.hpp"

use crate::utils::copy_bytes;
use libc::{sockaddr, sockaddr_un, socklen_t, AF_UNIX, EINVAL, ENAMETOOLONG};

// #include <string>
#[derive(Default, Debug, Clone)]
pub struct IpcAddress {
    // struct sockaddr_un address;
    pub address: sockaddr_un,
    // socklen_t _addrlen;
    pub addrlen: socklen_t,
}

impl IpcAddress {
    //
    // IpcAddress ();
    pub fn new() -> Self {
        Self::default()
    }

    // IpcAddress (const sockaddr *sa_, socklen_t sa_len_);
    pub fn new2(sa_: &sockaddr, sa_len_: socklen_t) -> Self {
        // zmq_assert (sa_ && sa_len_ > 0);
        // _addrlen (sa_len_)
        // memset (&address, 0, sizeof address);
        // if (sa_.sa_family == AF_UNIX) {
        //     memcpy(&address, sa_, sa_len_);
        // }
        Self {
            addrlen: sa_len_,
            address: sa_.clone() as sockaddr_un,
        }
    }

    // ~IpcAddress ();

    //  This function sets up the address for UNIX domain transport.
    // int resolve (path_: &str);
    pub fn resolve(&mut self, path_: &str) -> i32 {
        let path_len = path_.len();
        if (path_len >= self.address.sun_path.len()) {
            errno = ENAMETOOLONG;
            return -1;
        }
        if (path_[0] == '@' && !path_[1]) {
            errno = EINVAL;
            return -1;
        }

        address.sun_family = AF_UNIX;
        copy_bytes(
            address.sun_path,
            0,
            path_.as_bytes(),
            0,
            (path_len + 1) as i32,
        );
        /* Abstract sockets start with 0 */
        if (path_[0] == '@') {
            *address.sun_path = 0;
        }

        _addrlen = (offsetof(sockaddr_un, sun_path) + path_len);
        return 0;
    }

    //  The opposite to resolve()
    // int to_string (std::string &addr_) const;

    pub fn to_string(&mut self, addr_: &str) -> i32 {
        if (address.sun_family != AF_UNIX) {
            addr_.clear();
            return -1;
        }

        let prefix = "ipc://";
        // char buf[sizeof prefix + sizeof address.sun_path];
        // char *pos = buf;
        // memcpy (pos, prefix, sizeof prefix - 1);
        let mut buf = String::new();
        buf += prefix;
        let mut buf_pos = prefix.len();
        let src_pos = address.sun_path;

        if (!address.sun_path[0] && address.sun_path[1]) {
            // *pos+= 1 = '@';
            buf[buf_pos] = '@';
            src_pos += 1;
            buf_pos += 1;
        }
        // according to http://man7.org/linux/man-pages/man7/unix.7.html, NOTES
        // section, address.sun_path might not always be null-terminated; therefore,
        // we calculate the length based of addrlen
        // TODO:
        // let src_len =
        // strnlen (src_pos, _addrlen - offsetof (sockaddr_un, sun_path)
        //     - (src_pos - address.sun_path));
        // copy_bytes (pos, src_pos, src_len);
        // addr_.assign (buf, pos - buf + src_len);
        return 0;
    }

    // const sockaddr *addr () const;

    // socklen_t addrlen () const;
}

// IpcAddress::IpcAddress ()
// {
//     memset (&address, 0, sizeof address);
// }

// IpcAddress::~IpcAddress ()
// {
// }

// const sockaddr *IpcAddress::addr () const
// {
//     return reinterpret_cast<const sockaddr *> (&address);
// }
//
// socklen_t IpcAddress::addrlen () const
// {
//     return _addrlen;
// }

// #endif
