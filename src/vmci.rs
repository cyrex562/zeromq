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

// #include "ip.hpp"
// #include "vmci.hpp"
// #include "vmci_address.hpp"

// #if defined ZMQ_HAVE_VMCI

// #include <cassert>
// #include <vmci_sockets.h>

use std::os::raw::c_void;
use libc::{c_char, c_int, setsockopt, SOCKET};
use windows::Win32::Networking::WinSock::{SOCK_STREAM, SOCKET_ERROR};
use crate::context::ZmqContext;
use crate::fd::ZmqFileDesc;
use crate::ip::open_socket;
use crate::options::ZmqOptions;
use crate::vmci_address::ZmqVmciAddress;

pub fn tune_vmci_buffer_size(context: &mut ZmqContext,
                             sockfd_: &mut ZmqFileDesc,
                             default_size_: u64,
                             min_size_: u64,
                             max_size_: u64) {
    let family = context_.get_vmci_socket_family();
    // assert (family != -1);

    unsafe {
        if (default_size_ != 0) {
            let rc = setsockopt(sockfd_ as SOCKET, family, SO_VMCI_BUFFER_SIZE,
                                (&default_size_) as *const c_char, 8);
// #if defined ZMQ_HAVE_WINDOWS
//         wsa_assert (rc != SOCKET_ERROR);
// #else
            // errno_assert (rc == 0);
// #endif
        }
    }

    unsafe {
        if (min_size_ != 0) {
            let rc = setsockopt(sockfd_ as SOCKET, family, SO_VMCI_BUFFER_SIZE,
                                &min_size_ as *const c_char, 8);
// #if defined ZMQ_HAVE_WINDOWS
            wsa_assert(rc != SOCKET_ERROR);
// #else
            // errno_assert (rc == 0);
// #endif
        }
    }

    unsafe {
        if (max_size_ != 0) {
            let rc = setsockopt(sockfd_ as SOCKET, family, SO_VMCI_BUFFER_SIZE,
                                &max_size_ as *const c_char, 8);
// #if defined ZMQ_HAVE_WINDOWS
//             wsa_assert(rc != SOCKET_ERROR);
// #else
            // errno_assert (rc == 0);
// #endif
        }
    }
}

// #if defined ZMQ_HAVE_WINDOWS
pub fn tune_vmci_connect_timeout(context_: &mut ZmqContext,
                                 sockfd_: &mut ZmqFileDesc,
                                 timeout: u32) {
// #else
// void tune_vmci_connect_timeout (ZmqContext *context_,
//                                      sockfd_: &mut ZmqFileDesc,
//                                      struct timeval timeout)
// #endif {
    let family = context_.get_vmci_socket_family();
    // assert(family != -1);

    let rc = unsafe {
        // TODO
        // setsockopt(sockfd_, family, SO_VMCI_CONNECT_TIMEOUT,
        //            &timeout, sizeof timeout)
    };
// #if defined ZMQ_HAVE_WINDOWS
//     wsa_assert (rc != SOCKET_ERROR);
// #else
    // errno_assert (rc == 0);
// #endif
}

pub fn vmci_open_socket(address_: &str,
                        options: &ZmqOptions,
                        out_vmc_addr: &mut ZmqVmciAddress) -> ZmqFileDesc {
    //  Convert the textual address into address structure.
    let rc = out_vmci_addr_.resolve(address_);
    if (rc != 0) {
        return retired_fd;
    }

    //  Create the socket.
    let mut s: ZmqFileDesc = open_socket(out_vmci_addr_.family(), SOCK_STREAM as i32, 0);

    if (s == retired_fd) {
        return retired_fd;
    }

    return s;
}

// #endif
