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
use libc::{accept, bind, c_char, c_int, listen, setsockopt, SOCKET};
use windows::Win32::Networking::WinSock::{INVALID_SOCKET, SOCK_STREAM, SOCKET_ERROR};
use anyhow::bail;
use bincode::options;
use std::ptr::null_mut;
use crate::address::ZmqAddress;
use crate::address_family::AF_INET;
use crate::context::ZmqContext;
use crate::defines::RETIRED_FD;
use crate::endpoint::make_unconnected_bind_endpoint_pair;
use crate::defines::ZmqFileDesc;
use crate::ip::ip_open_socket;
use crate::listener::ZmqListener;
use crate::transport::ZmqTransport;

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
                        ctx: &ZmqContext,
                        out_vmc_addr: &mut ZmqVmciAddress) -> ZmqFileDesc {
    //  Convert the textual address into address structure.
    let rc = out_vmci_addr_.resolve(address_);
    if (rc != 0) {
        return RETIRED_FD;
    }

    //  Create the socket.
    let mut s: ZmqFileDesc = ip_open_socket(out_vmci_addr_.family(), SOCK_STREAM as i32, 0);

    if (s == RETIRED_FD) {
        return RETIRED_FD;
    }

    return s;
}

// #endif


pub fn vmci_in_event(listener: &mut ZmqListener) -> anyhow::Result<()> {
    let mut fd: ZmqFileDesc = vmci_accept(listener)?;

    //  If connection was reset by the peer in the meantime, just ignore it.
    if fd == RETIRED_FD as usize {
        listener.socket
            .event_accept_failed(&make_unconnected_bind_endpoint_pair(&listener.endpoint), -1);
        bail!("event accept failed")
    }

    tune_vmci_buffer_size(
        listener.socket.context,
        &mut fd,
        listener.socket.context.vmci_buffer_size,
        listener.socket.context.vmci_buffer_min_size,
        listener.socket.context.vmci_buffer_max_size,
    );

    if (options.vmci_connect_timeout > 0) {
        // #if defined ZMQ_HAVE_WINDOWS
        tune_vmci_connect_timeout(listener.socket.context, &mut fd, options.vmci_connect_timeout);
        // #else
        //         struct timeval timeout = {0, options.vmci_connect_timeout * 1000};
        //         tune_vmci_connect_timeout (this.get_ctx (), fd, timeout);
        // #endif
    }

    //  Create the engine object for this connection.
    listener.create_engine()?;

    Ok(())
}


pub fn vmci_set_local_address(listener: &mut ZmqListener, addr_: &mut String) -> anyhow::Result<()> {
    //  Create addr on stack for auto-cleanup
    // std::string addr (addr_);

    //  Initialise the address structure.
    let mut address = ZmqAddress::new(ZmqTransport::ZmqVmci, AF_INET as i32, 0, false, None, None, None, None, None, None, None, None, None, None);
    *addr_ = address.resolve()?;

    //  Create a listening socket.
    listener.fd = ip_open_socket(
        listener.socket.get_vmci_socket_family(),
        SOCK_STREAM as i32,
        0,
    );
    // #ifdef ZMQ_HAVE_WINDOWS
    if listener.fd == INVALID_SOCKET as usize{
        // errno = wsa_error_to_errno (WSAGetLastError ());
        bail!("invalid socket");
    }
    // #if !defined _WIN32_WCE
    //  On Windows, preventing sockets to be inherited by child processes.
    // BOOL brc = SetHandleInformation ((HANDLE) _s, HANDLE_FLAG_INHERIT, 0);
    // win_assert (brc);
    // #endif
    // #else
    //     if (_s == -1) {
    //         return -1;
    //     }
    // #endif

    listener.endpoint = address.to_string()?;

    //  Bind the socket.
    let bind_result = unsafe { bind(listener.fd, address.addr(), address.addrlen()) };
    // #ifdef ZMQ_HAVE_WINDOWS
    if (bind_result == SOCKET_ERROR) {
        // errno = wsa_error_to_errno (WSAGetLastError ());
        // goto error;
        bail!("bind failed")
    }
    // #else
    // if (rc != 0) {}
    // goto error;
    // #endif

    //  Listen for incoming connections.
    let listen_result = unsafe { listen(listener.fd, options.backlog) };
    // #ifdef ZMQ_HAVE_WINDOWS
    if (listen_result == SOCKET_ERROR) {
        // errno = wsa_error_to_errno (WSAGetLastError ());
        // goto error;
        bail!("listen failed")
    }
    // #else
    // if (rc != 0) {
    //     // goto
    //     // error;
    // }
    // #endif

    listener.socket
        .event_listening(&make_unconnected_bind_endpoint_pair(listen.endpoint), listen.fd);
    Ok(())

    // error:
    //     int err = errno;
    //     close ();
    //   // errno = err;
    //     return -1;
}

pub fn vmci_accept(listener: &mut ZmqListener) -> anyhow::Result<ZmqFileDesc> {
    //  Accept one connection and deal with different failure modes.
    //  The situation where connection cannot be accepted due to insufficient
    //  resources is considered valid and treated by ignoring the connection.
    // zmq_assert (_s != retired_fd);
    let mut sock: ZmqFileDesc = unsafe { accept(listener.fd, null_mut(), null_mut()) };

    // #ifdef ZMQ_HAVE_WINDOWS
    if sock == INVALID_SOCKET as usize {
        // wsa_assert (WSAGetLastError () == WSAEWOULDBLOCK
        //             || WSAGetLastError () == WSAECONNRESET
        //             || WSAGetLastError () == WSAEMFILE
        //             || WSAGetLastError () == WSAENOBUFS);
        return Ok(RETIRED_FD as ZmqFileDesc);
    }
    // #if !defined _WIN32_WCE
    //  On Windows, preventing sockets to be inherited by child processes.
    // let brc = SetHandleInformation ((HANDLE) sock, HANDLE_FLAG_INHERIT, 0);
    // win_assert (brc);
    // #endif
    // #else
    if sock == -1 {
        // errno_assert (errno == EAGAIN || errno == EWOULDBLOCK || errno == EINTR
        //               || errno == ECONNABORTED || errno == EPROTO
        //               || errno == ENOBUFS || errno == ENOMEM || errno == EMFILE
        //               || errno == ENFILE);
        return Ok(RETIRED_FD as ZmqFileDesc);
    }
    // #endif

    //  Race condition can cause socket not to be closed (if fork happens
    //  between accept and this point).
    // #ifdef FD_CLOEXEC
    #[cfg(target_os="linux")]
    let rc = fcntl(sock, F_SETFD, FD_CLOEXEC);
    // errno_assert (rc != -1);
    // #endif

    return Ok(sock);
}
