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
// #include "err.hpp"
// #include "macros.hpp"
// #include "config.hpp"
// #include "address.hpp"

// #if !defined ZMQ_HAVE_WINDOWS
// #include <fcntl.h>
// #include <sys/types.h>
// #include <sys/socket.h>
// #include <sys/stat.h>
// #include <netdb.h>
// #include <netinet/in.h>
// #include <netinet/tcp.h>
// #include <stdlib.h>
// #include <unistd.h>

// #include <vector>
// #else
// #include "tcp.hpp"
// #ifdef ZMQ_HAVE_IPC
// #include "ipc_address.hpp"
// #endif

// #include <direct.h>

// #define rmdir _rmdir
// #define unlink _unlink
// #endif

// #if defined ZMQ_HAVE_OPENVMS || defined ZMQ_HAVE_VXWORKS
// #include <ioctl.h>
// #endif

// #if defined ZMQ_HAVE_VXWORKS
// #include <unistd.h>
// #include <sockLib.h>
// #include <ioLib.h>
// #endif

// #if defined ZMQ_HAVE_EVENTFD
// #include <sys/eventfd.h>
// #endif

// #if defined ZMQ_HAVE_OPENPGM
// #ifdef ZMQ_HAVE_WINDOWS
// #define __PGM_WININT_H__
// #endif

// #include <pgm/pgm.h>
// #endif

// #ifdef __APPLE__
// #include <TargetConditionals.h>
// #endif

// ZmqFileDesc open_socket (domain_: i32, type_: i32, protocol_: i32);

//  Sets the socket into non-blocking mode.
// void unblock_socket (ZmqFileDesc s_);

//  Enable IPv4-mapping of addresses in case it is disabled by default.
// void enable_ipv4_mapping (ZmqFileDesc s_);

//  Returns string representation of peer's address.
//  Socket sockfd_ must be Connected. Returns true iff successful.
// int get_peer_ip_address (ZmqFileDesc sockfd_, std::string &ip_addr_);

// Sets the IP Type-Of-Service for the underlying socket
// void set_ip_type_of_service (ZmqFileDesc s_, iptos_: i32);

// Sets the protocol-defined priority for the underlying socket
// void set_socket_priority (ZmqFileDesc s_, priority_: i32);

// Sets the SO_NOSIGPIPE option for the underlying socket.
// Return 0 on success, -1 if the connection has been closed by the peer
// int set_nosigpipe (ZmqFileDesc s_);

// Binds the underlying socket to the given device, eg. VRF or interface
// int bind_to_device (ZmqFileDesc s_, bound_device_: &str);

// Initialize network subsystem. May be called multiple times. Each call must be matched by a call to shutdown_network.
// bool initialize_network ();

// Shutdown network subsystem. Must be called once for each call to initialize_network before terminating.
// void shutdown_network ();

// Creates a pair of sockets (using SIGNALER_PORT on OS using TCP sockets).
// Returns -1 if we could not make the socket pair successfully
// int make_fdpair (ZmqFileDesc *r_, ZmqFileDesc *w_);

// Makes a socket non-inheritable to child processes.
// Asserts on any failure.
// void make_socket_noninheritable (ZmqFileDesc sock_);

//  Asserts that:
//  - an internal 0MQ error did not occur,
//  - and, if a socket error occurred, it can be recovered from.
// void assert_success_or_recoverable (ZmqFileDesc s_, rc_: i32);

// #ifdef ZMQ_HAVE_IPC
// Create an IPC wildcard path address
// int create_ipc_wildcard_address (std::string &path_, std::string &file_);

use anyhow::bail;
use std::ffi::{c_void, CString};
use std::mem;
use std::ptr::null_mut;

use libc::{accept, c_char, connect, listen, rmdir, sockaddr, socket, unlink};

#[cfg(target_os = "linux")]
use libc::{
    eventfd, fcntl, mkdtemp, mkstemp, sockaddr_un, socklen_t, AF_UNIX, EFD_CLOEXEC, FD_CLOEXEC,
    F_GETFL, F_SETFD, F_SETFL, IPV6_TCLASS, O_NONBLOCK, SOCK_CLOEXEC, SO_BINDTODEVICE, SO_PRIORITY,
};

use crate::address::get_socket_address;
use crate::address::ZmqSocketEnd::SocketEndRemote;
#[cfg(target_os = "windows")]
use windows::core::PSTR;

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, SetHandleInformation, BOOL, ERROR_ACCESS_DENIED, FALSE, HANDLE,
    HANDLE_FLAGS, HANDLE_FLAG_INHERIT,  TRUE,
};
#[cfg(target_os = "windows")]
use windows::Win32::Networking::WinSock::{
    bind, getsockname, getsockopt, htonl, htons, AF_INET, INADDR_LOOPBACK, INVALID_SOCKET,
    SOCK_STREAM, SO_ERROR, SO_REUSEADDR,
};
#[cfg(target_os = "windows")]
use windows::Win32::Networking::WinSock::{
    closesocket, recv, send, AF_UNIX, SEND_RECV_FLAGS,
};
#[cfg(target_os = "windows")]
use windows::Win32::Networking::WinSock::{
    getnameinfo, ioctlsocket, setsockopt, WSACleanup, WSAGetLastError, WSAStartup, FIONBIO,
    IPPROTO_IP, IPPROTO_IPV6, IPPROTO_TCP, IPV6_V6ONLY, IP_TOS, NI_MAXHOST, NI_NUMERICHOST,
    SOCKADDR, SOCKET, SOCKET_ERROR, SOL_SOCKET, TCP_NODELAY, WSADATA, WSAEFAULT, WSAEINPROGRESS,
    WSAENOTSOCK, WSANOTINITIALISED, WSA_FLAG_NO_HANDLE_INHERIT, WSA_FLAG_OVERLAPPED,
};
use windows::Win32::Networking::WinSock::{WSASocketA, WSA_ERROR, SOCKADDR_IN, ADDRESS_FAMILY};
#[cfg(target_os = "windows")]
use windows::Win32::Security::{
    InitializeSecurityDescriptor, SetSecurityDescriptorDacl, PSECURITY_DESCRIPTOR,
    SECURITY_ATTRIBUTES, };
use windows::Win32::Storage::FileSystem::{
    CreateDirectoryA, GetTempPathA, FILE_ACCESS_RIGHTS, SYNCHRONIZE,
};
use windows::Win32::System::SystemServices::SECURITY_DESCRIPTOR_REVISION;
use windows::Win32::System::Threading::SYNCHRONIZATION_ACCESS_RIGHTS;
#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::{
    CreateEventA, CreateMutexA, OpenEventA, WaitForSingleObject, EVENT_MODIFY_STATE, INFINITE,
};
#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::{ReleaseMutex, SetEvent};
use windows::Win32::System::WindowsProgramming::OpenMutexA;

use crate::defines::{RETIRED_FD, signaler_port, ZmqFileDesc};
#[cfg(target_os = "windows")]
use crate::err::wsa_error_to_errno;
use crate::platform_socket::ZmqSockaddrStorage;
use crate::tcp::tcp_tune_loopback_fast_path;

#[cfg(target_os = "linux")]
use crate::unix_sockaddr::sockaddr_in;

use crate::utils::MAKEWORD;

// #ifndef ZMQ_HAVE_WINDOWS
// Acceptable temporary directory environment variables
pub const tmp_env_vars: [&'static str; 3] = ["TMPDIR", "TEMPDIR", "TMP"];
// #endif

pub fn ip_open_socket(domain: i32, mut type_: i32, protocol: i32) -> anyhow::Result<ZmqFileDesc> {
    //  Setting this option result in sane behaviour when exec() functions
    //  are used. Old sockets are closed and don't block TCP ports etc.
    #[cfg(target_feature = "sock_cloexec")]
    {
        type_ |= SOCK_CLOEXEC;
    }

    // #if defined ZMQ_HAVE_WINDOWS && defined WSA_FLAG_NO_HANDLE_INHERIT
    // if supported, create socket with WSA_FLAG_NO_HANDLE_INHERIT, such that
    // the race condition in making it non-inheritable later is avoided
    #[cfg(target_os = "windows")]
    let s = unsafe {
        WSASocketA(
            domain,
            type_,
            protocol,
            None,
            0,
            WSA_FLAG_OVERLAPPED | WSA_FLAG_NO_HANDLE_INHERIT,
        )
    };

    // #else
    #[cfg(target_os = "linux")]
    let s = unsafe { socket(domain_, type_, protocol_) };

    // #endif
    if s == RETIRED_FD {
        // #ifdef ZMQ_HAVE_WINDOWS
        // #[cfg(windows)]
        // {
        //     unsafe { errno = wsa_error_to_errno(WSAGetLastError()) };
        // }
        // // #endif
        // return retired_fd as ZmqFileDesc;
        bail!("failed to create socket")
    }

    make_socket_noninheritable(s as ZmqFileDesc);

    //  Socket is not yet Connected so EINVAL is not a valid networking error
    #[cfg(target_feature = "nosigpipe")]
    set_nosigpipe(s as ZmqFileDesc);
    // errno_assert (rc == 0);

    return Ok(s as ZmqFileDesc);
}

pub fn unblock_socket(s: ZmqFileDesc) {
    // #if defined ZMQ_HAVE_WINDOWS
    #[cfg(windows)]
    {
        let mut nonblock = 1u32;
        let rc: i32 = unsafe { ioctlsocket(s, FIONBIO, &mut nonblock) };
        // wsa_assert(rc != SOCKET_ERROR);
    }
    // #elif defined ZMQ_HAVE_OPENVMS || defined ZMQ_HAVE_VXWORKS
    //
    //     int nonblock = 1;
    //     int rc = ioctl (s_, FIONBIO, &nonblock);
    // errno_assert (rc != -1);
    // #else
    #[cfg(not(windows))]
    {
        let mut flags = unsafe { fcntl(s, F_GETFL, 0) };
        if (flags == -1) {
            flags = 0;
        }
        let rc = unsafe { fcntl(s, F_SETFL, flags | O_NONBLOCK) };
    }
}

pub fn enable_ipv4_mapping(s_: ZmqFileDesc) {
    // LIBZMQ_UNUSED (s_);

    // #if defined IPV6_V6ONLY && !defined ZMQ_HAVE_OPENBSD                           \
    //   && !defined ZMQ_HAVE_DRAGONFLY
    // #ifdef ZMQ_HAVE_WINDOWS
    //     DWORD flag = 0;
    // #else
    let mut flag = 0u32;
    // #endif
    let rc: i32 = unsafe {
        setsockopt(
            s_,
            IPPROTO_IPV6 as i32,
            IPV6_V6ONLY,
            Some(&flag.to_le_bytes()),
        )
    };
    // #ifdef ZMQ_HAVE_WINDOWS
    //     wsa_assert (rc != SOCKET_ERROR);
    // #else
    // errno_assert (rc == 0);
    // #endif
    // #endif
}

pub fn get_peer_ip_address(sockfd_: ZmqFileDesc, mut ip_addr_: &str) -> anyhow::Result<()> {
    let mut ss = ZmqSockaddrStorage::default();
    let addrlen = get_socket_address(sockfd_, SocketEndRemote, &mut ss)?;

    unsafe {
        if addrlen.is_err() == 0 {
            // #ifdef ZMQ_HAVE_WINDOWS
            if cfg!(windows) {
                let last_error = WSAGetLastError();
                // wsa_assert(last_error != WSANOTINITIALISED && last_error != WSAEFAULT
                //     && last_error != WSAEINPROGRESS
                //     && last_error != WSAENOTSOCK);
            }
            // # elif! defined(TARGET_OS_IPHONE) || !TARGET_OS_IPHONE
            // errno_assert (errno != EBADF && errno != EFAULT && errno != ENOTSOCK);
            // #else
            // errno_assert (errno != EFAULT && errno != ENOTSOCK);
            // #endif
            return Ok(());
        }
    }

    let mut host: [u8; NI_MAXHOST as usize] = [0; NI_MAXHOST];
    let sa = SOCKADDR {
        sa_family: ss.ss_family,
        sa_data: (ss as SOCKADDR).sa_data,
    };
    let rc = unsafe { getnameinfo(&sa, addrlen, Some(&mut host), None, 0) };
    if rc != 0 {
        return Ok(());
    }

    ip_addr_ = host.into();

    // union
    // {
    //     struct sockaddr sa;
    //     struct sockaddr_storage sa_stor;
    // } u;

    // u.sa_stor = ss;
    return Ok(sa.family);
}

pub fn set_ip_type_of_service(s_: ZmqFileDesc, iptos_: i32) -> anyhow::Result<()> {
    let mut rc = unsafe { setsockopt(s_, IPPROTO_IP as i32, IP_TOS, Some(&iptos_.to_le_bytes())) };

    // #ifdef ZMQ_HAVE_WINDOWS
    //     wsa_assert (rc != SOCKET_ERROR);
    // #else
    // errno_assert (rc == 0);
    // #endif

    //  Windows and Hurd do not support IPV6_TCLASS
    // #if !defined(ZMQ_HAVE_WINDOWS) && defined(IPV6_TCLASS)
    #[cfg(not(target_os = "windows"))]
    {
        rc = unsafe {
            setsockopt(
                s_,
                IPPROTO_IPV6 as i32,
                IPV6_TCLASS,
                Some(&iptos_.to_le_bytes()),
            )
        };
    }

    //  If IPv6 is not enabled ENOPROTOOPT will be returned on Linux and
    //  EINVAL on OSX
    // if (rc == -1) {
    //     // errno_assert (errno == ENOPROTOOPT || errno == EINVAL);
    // }
    // #endif
}

pub fn set_socket_priority(s_: ZmqFileDesc, priority_: i32) {
    #[cfg(target_feature = "so_priority")]
    {
        let rc = unsafe { setsockopt(s_, SOL_SOCKET, SO_PRIORITY, Some(&priority_.to_le_bytes())) };
    }
}

#[cfg(target_feature = "nosigpipe")]
pub fn set_nosigpipe(s_: ZmqFileDesc) -> anyhow::Result<()> {
    // #ifdef SO_NOSIGPIPE
    //  Make sure that SIGPIPE signal is not generated when writing to a
    //  connection that was already closed by the peer.
    //  As per POSIX spec, EINVAL will be returned if the socket was valid but
    //  the connection has been reset by the peer. Return an error so that the
    //  socket can be closed and the connection retried if necessary.
    let set = 1;
    let rc = unsafe { setsockopt(s_, SOL_SOCKET, SO_NOSIGPIPE, Some(&set.to_le_bytes())) };
    if rc != 0 {
        bail!("setsockopt(SO_NOSIGPIPE) failed");
    }
    Ok(())
}

pub fn bind_to_device(s_: ZmqFileDesc, bound_device_: &str) -> anyhow::Result<()> {
    #[cfg(target_feature = "so_bindtodevice")]
    {
        let rc = unsafe {
            setsockopt(
                s_,
                SOL_SOCKET,
                SO_BINDTODEVICE,
                Some(bound_device_.as_bytes()),
            )
        };

        if rc != 0 {
            assert_success_or_recoverable(s_, rc);
            return -1;
        }
    }
    Ok(())
}

pub fn initialize_network() -> bool {
    // #if defined ZMQ_HAVE_OPENPGM

    //  Init PGM transport. Ensure threading and timer are enabled. Find PGM
    //  protocol ID. Note that if you want to use gettimeofday and sleep for
    //  openPGM timing, set environment variables PGM_TIMER to "GTOD" and
    //  PGM_SLEEP to "USLEEP".
    #[cfg(feature = "openpgm")]
    {
        pgm_error_t * pgm_error = null_mut();
        let ok = pgm_init(&pgm_error);
        if (ok != TRUE) {
            //  Invalid parameters don't set pgm_error_t
            // zmq_assert (pgm_error != null_mut());
            if (pgm_error.domain == PGM_ERROR_DOMAIN_TIME && (pgm_error.code == PGM_ERROR_FAILED)) {
                //  Failed to access RTC or HPET device.
                pgm_error_free(pgm_error);
                errno = EINVAL;
                return false;
            }

            //  PGM_ERROR_DOMAIN_ENGINE: WSAStartup errors or missing WSARecvMsg.
            // zmq_assert (false);
        }
    }
    // #endif

    // #ifdef ZMQ_HAVE_WINDOWS
    //  Initialise Windows sockets. Note that WSAStartup can be called multiple
    //  times given that WSACleanup will be called for each WSAStartup.
    if cfg!(windows) {
        let version_requested = MAKEWORD(2, 2);
        let mut wsa_data = WSADATA {
            wVersion: 0,
            wHighVersion: 0,
            iMaxSockets: 0,
            iMaxUdpDg: 0,
            lpVendorInfo: PSTR::null(),
            szDescription: [0; 257],
            szSystemStatus: [0; 129],
        };
        let rc: i32 = unsafe { WSAStartup(version_requested, &mut wsa_data) };
        // zmq_assert (rc == 0);
        // zmq_assert (LOBYTE (wsa_data.wVersion) == 2
        // &&HIBYTE(wsa_data.wVersion) == 2);
    }
    // #endif

    return true;
}

pub fn shutdown_network() {
    // #ifdef ZMQ_HAVE_WINDOWS
    //  On Windows, uninitialise socket layer.
    #[cfg(windows)]
    {
        let rc: i32 = unsafe { WSACleanup() };
    }
    // wsa_assert (rc != SOCKET_ERROR);
    // #endif

    #[cfg(feature = "openpgm")]
    {
        // #if defined ZMQ_HAVE_OPENPGM
        //  Shut down the OpenPGM library.
        if pgm_shutdown() != TRUE {}
        // zmq_assert (false);
        // #endif
    }
}

// #if defined ZMQ_HAVE_WINDOWS
#[cfg(windows)]
pub fn tune_socket(socket: SOCKET) -> anyhow::Result<()> {
    let mut tcp_nodelay = 1;
    let rc: i32 = unsafe {
        setsockopt(
            socket,
            IPPROTO_TCP as i32,
            TCP_NODELAY,
            Some(&tcp_nodelay.to_le_bytes()),
        )
    };
    // wsa_assert(rc != SOCKET_ERROR);

    tcp_tune_loopback_fast_path(socket as ZmqFileDesc);
    Ok(())
}

#[cfg(windows)]
pub fn make_fdZmqPaircpip(fd_r: &mut ZmqFileDesc, fd_w: &mut ZmqFileDesc) -> i32 {
    // #if !defined _WIN32_WCE && !defined ZMQ_HAVE_WINDOWS_UWP
    //  Windows CE does not manage security attributes
    let mut sd: PSECURITY_DESCRIPTOR = PSECURITY_DESCRIPTOR::default();
    let mut sa: SECURITY_ATTRIBUTES = SECURITY_ATTRIBUTES {
        nLength: 0,
        lpSecurityDescriptor: null_mut(),
        bInheritHandle: Default::default(),
    };
    // memset (&sd, 0, sizeof sd);
    // memset (&sa, 0, sizeof sa);

    unsafe {
        InitializeSecurityDescriptor(sd, SECURITY_DESCRIPTOR_REVISION);
        SetSecurityDescriptorDacl(sd, TRUE, None, FALSE);
    }

    sa.nLength = mem::size_of::<SECURITY_ATTRIBUTES>() as u32;
    sa.lpSecurityDescriptor = sd.0;
    // #endif

    //  This function has to be in a system-wide critical section so that
    //  two instances of the library don't accidentally create signaler
    //  crossing the process boundary.
    //  We'll use named event object to implement the critical section.
    //  Note that if the event object already exists, the CreateEvent requests
    //  EVENT_ALL_ACCESS access right. If this fails, we try to open
    //  the event object asking for SYNCHRONIZE access only.
    let mut sync: HANDLE = HANDLE::default();

    //  Create critical section only if using fixed signaler port
    //  Use problematic Event implementation for compatibility if using old port 5905.
    //  Otherwise use Mutex implementation.
    let event_signaler_port: i32 = 5905;
    let val = "Global\\zmq-signaler-port-sync".to_string().as_ptr() as *const c_void;

    if signaler_port == event_signaler_port {
        // #if !defined _WIN32_WCE && !defined ZMQ_HAVE_WINDOWS_UWP
        // TODO
        // sync = unsafe { CreateEventA(Some(&sa), FALSE, TRUE, val).unwrap() };
        // #else
        sync = unsafe { CreateEventA(None, FALSE, TRUE, val).unwrap() };
        // #endif
        let mut last_err = unsafe { GetLastError() };
        if sync == 0 && last_err == ERROR_ACCESS_DENIED {
            sync = unsafe {
                OpenEventA(
                    (SYNCHRONIZE | EVENT_MODIFY_STATE as FILE_ACCESS_RIGHTS)
                        as SYNCHRONIZATION_ACCESS_RIGHTS,
                    FALSE,
                    val,
                )
                .unwrap()
            };
        }

        // win_assert (sync != null_mut());
    } else if signaler_port != 0 {
        let mut mutex_name_str = format!("Global\\zmq-signaler-port-{}", signaler_port);
        let mutex_name = unsafe { CString::from_vec_unchecked(mutex_name_str.into_bytes()) };

        // #if !defined _WIN32_WCE && !defined ZMQ_HAVE_WINDOWS_UWP
        sync = unsafe { CreateMutexA(Some(&sa), FALSE, mutex_name).unwrap() };
        // #else
        // TODO
        // sync = unsafe { CreateMutexA(None, FALSE, mutex_name).unwrap() };
        // #endif
        let last_err = unsafe { GetLastError() };
        if sync == null_mut() && last_err == ERROR_ACCESS_DENIED {
            sync = unsafe { OpenMutexA(SYNCHRONIZE as u32, FALSE, &mutex_name) };
        }
    }

    //  Windows has no 'socketpair' function. CreatePipe is no good as pipe
    //  handles cannot be polled on. Here we create the socketpair by hand.
    // TODO
    // *w_. = INVALID_SOCKET;
    // *r_ = INVALID_SOCKET;

    //  Create listening socket.
    let mut listener = ip_open_socket(AF_INET as i32, SOCK_STREAM as i32, 0)?;
    // wsa_assert (listener != INVALID_SOCKET);

    //  Set SO_REUSEADDR and TCP_NODELAY on listening socket.
    let mut so_reuseaddr: BOOL = TRUE;
    let mut rc = unsafe {
        setsockopt(
            listener,
            SOL_SOCKET,
            SO_REUSEADDR,
            Some(&so_reuseaddr.to_le_bytes()),
        )
    };
    // wsa_assert(rc != SOCKET_ERROR);

    tune_socket(listener as SOCKET)?;

    //  Init sockaddr to signaler port.
    #[cfg(linux)]
    let mut addr: sockaddr_in = sockaddr_in::default();
    #[cfg(windows)]
    let mut addr: SOCKADDR_IN = SOCKADDR_IN::default();
    // memset (&addr, 0, sizeof addr);
    addr.sin_family = AF_INET as ADDRESS_FAMILY;
    addr.sin_addr.s_addr = unsafe { htonl(INADDR_LOOPBACK) };
    addr.sin_port = unsafe { htons(signaler_port as u16) };

    //  Create the writer socket.
    *fd_w = ip_open_socket(AF_INET as i32, SOCK_STREAM as i32, 0)?;
    // wsa_assert (*w_ != INVALID_SOCKET);

    if sync != null_mut() {
        //  Enter the critical section.
        let dwrc = unsafe { WaitForSingleObject(sync, INFINITE) };
        // zmq_assert (dwrc == WAIT_OBJECT_0 || dwrc == WAIT_ABANDONED);
    }

    //  Bind listening socket to signaler port.
    rc = unsafe { bind(listener, &addr as *const SOCKADDR, addr.len()) };

    if rc != SOCKET_ERROR && signaler_port == 0 {
        //  Retrieve ephemeral port number
        let addrlen = addr.len();
        rc = unsafe { getsockname(listener, &mut addr as *mut SOCKADDR, &mut addrlen) };
    }

    //  Listen for incoming connections.
    if rc != SOCKET_ERROR {
        rc = unsafe { listen(listener, 1) };
    }

    //  Connect writer to the listener.
    if rc != SOCKET_ERROR {
        rc = unsafe { connect(*fd_w, &addr as *const sockaddr, addr.len()) };
    }

    //  Accept connection from writer.
    if rc != SOCKET_ERROR {
        //  Set TCP_NODELAY on writer socket.
        tune_socket(*fd_w as SOCKET)?;

        *fd_r = unsafe { accept(listener, null_mut(), null_mut()) };
    }

    //  Send/receive large chunk to work around TCP slow start
    //  This code is a workaround for #1608
    if *fd_r as SOCKET != INVALID_SOCKET {
        let mut dummy_size = 1024 * 1024; //  1M to overload default receive buffer
                                          // unsigned char *dummy =
                                          //    (malloc (dummy_size));
                                          // wsa_assert (dummy);
        let dummy: Vec<u8> = Vec::with_capacity(dummy_size as usize);

        let mut still_to_send = (dummy_size);
        let mut still_to_recv = (dummy_size);
        while still_to_send || still_to_recv {
            let mut nbytes: i32 = 0i32;
            if still_to_send > 0 {
                nbytes = unsafe {
                    send(
                        *fd_w,
                        (&dummy[dummy_size - still_to_send..]),
                        0 as SEND_RECV_FLAGS,
                    )
                };
                // wsa_assert(nbytes != SOCKET_ERROR);
                still_to_send -= nbytes;
            }
            nbytes = unsafe {
                recv(
                    *fd_r,
                    (&mut dummy[dummy_size - still_to_recv..]),
                    0 as SEND_RECV_FLAGS,
                )
            };
            // wsa_assert(nbytes != SOCKET_ERROR);
            still_to_recv -= nbytes;
        }
        // free(dummy);
    }

    //  Save errno if error occurred in Bind/listen/connect/accept.
    let mut saved_errno = WSA_ERROR::default();
    if *fd_r as SOCKET == INVALID_SOCKET {
        saved_errno = unsafe { WSAGetLastError() };
    }

    //  We don't need the listening socket anymore. Close it.
    rc = unsafe { closesocket(listener) };
    // wsa_assert (rc != SOCKET_ERROR);

    if sync != null_mut() {
        //  Exit the critical section.
        let mut brc: BOOL = TRUE;
        if signaler_port == event_signaler_port {
            brc = unsafe { SetEvent(sync) };
        } else {
            brc = unsafe { ReleaseMutex(sync) };
        }
        // win_assert (brc != 0);

        //  Release the kernel object
        brc = unsafe { CloseHandle(sync) };
        // win_assert (brc != 0);
    }

    if *fd_r as SOCKET != INVALID_SOCKET {
        make_socket_noninheritable(*fd_r);
        return 0;
    }
    //  Cleanup writer if connection failed
    if *fd_w as SOCKET != INVALID_SOCKET {
        rc = unsafe { closesocket(*fd_w) };
        // wsa_assert (rc != SOCKET_ERROR);
        *fd_w as SOCKET = INVALID_SOCKET;
    }
    //  Set errno from saved value
    let errno = wsa_error_to_errno(saved_errno);
    return -1;
}
// #endif

#[cfg(not(target_os = "windows"))]
pub fn make_fdpair(r_: &mut ZmqFileDesc, w_: &mut ZmqFileDesc) -> i32 {
    // #if defined ZMQ_HAVE_EVENTFD
    if cfg!(feature = "eventfd") {
        let mut flags = 0;
        let mut flags = 0;
        // #if defined ZMQ_HAVE_EVENTFD_CLOEXEC
        //  Setting this option result in sane behaviour when exec() functions
        //  are used. Old sockets are closed and don't block TCP ports, avoid
        //  leaks, etc.
        flags |= EFD_CLOEXEC;
        // #endif
        let fd = unsafe { eventfd(0, flags) };
        if (fd == -1) {
            // errno_assert (errno == ENFILE || errno == EMFILE);
            *w_ = -1;
            *r_ = -1;
            return -1;
        }
        *w_ = fd;
        *r_ = fd;
        return 0;
    }

    // #elif defined ZMQ_HAVE_WINDOWS
    // #ifdef ZMQ_HAVE_IPC
    if cfg!(target_os = "windows") {
        let mut address: IpcAddress = IpcAddress::default();
        let mut dirname = String::new();
        let mut filename = String::new();
        let mut lcladdr: sockaddr_un = sockaddr_un::default();
        let mut lcaddr_len = mem::size_of::<sockaddr_un>() as socklen_t;
        let mut rc = 0;
        let mut saved_errno = 0;

        // It appears that a lack of runtime AF_UNIX support
        // can fail in more than one way.
        // At least: open_socket can fail or later in Bind
        let mut ipc_fallback_on_tcpip = true;

        //  Create a listening socket. const SOCKET
        listener = ip_open_socket(AF_UNIX as i32, SOCK_STREAM as i32, 0);
        if (listener == RETIRED_FD) {
            //  This may happen if the library was built on a system supporting AF_UNIX, but the system running doesn't support it.
            // TODO
            // goto try_tcpip;
        }

        create_ipc_wildcard_address(&mut dirname, &mut filename);

        //  Initialise the address structure.
        rc = address.resolve(filename.c_str());
        if (rc != 0) {
            // goto error_closelistener;
        }

        //  Bind the socket to the file path.
        rc = unsafe { bind(listener, (address.addr()), address.addrlen()) };
        if (rc != 0) {
            let last_wsa_err = unsafe { WSAGetLastError() };
            errno = wsa_error_to_errno(last_wsa_err);
            // goto  error_closelistener;
        }
        // if we got here, ipc should be working,
        // so raise any remaining errors
        ipc_fallback_on_tcpip = false;

        //  Listen for incoming connections.
        rc = unsafe { listen(listener, 1) };
        if (rc != 0) {
            let last_wsa_err = unsafe { WSAGetLastError() };
            errno = wsa_error_to_errno(last_wsa_err);
            // goto error_closelistener;
        }

        rc = unsafe { getsockname(listener, (&mut lcladdr as &mut SOCKADDR), &mut lcladdr_len) };
        // wsa_assert(rc != -1);

        //  Create the client socket.
        *w_ = ip_open_socket(AF_UNIX as i32, SOCK_STREAM as i32, 0);
        if (*w_ == -1) {
            let last_wsa_err = unsafe { WSAGetLastError() };
            errno = wsa_error_to_errno(last_wsa_err);
            // goto error_closelistener;
        }

        //  Connect to the remote peer.
        rc = unsafe { connect(*w_, (&lcladdr as &sockaddr), lcladdr_len) };
        if (rc == -1) {
            // goto error_closeclient;
        }

        *r_ = unsafe { accept(listener, null_mut(), null_mut()) };
        // errno_assert (*r_ != -1);

        //  Close the listener socket, we don't need it anymore.
        rc = unsafe { closesocket(listener) };
        wsa_assert(rc == 0);

        //  Cleanup temporary socket file descriptor
        if (!filename.empty()) {
            rc = unsafe { unlink(filename.c_str()) };
            if ((rc == 0) && !dirname.empty()) {
                rc = unsafe { rmdir(dirname.c_str()) };
                dirname.clear();
            }
            filename.clear();
        }

        return 0;

        // error_closeclient: saved_errno = errno;
        // rc = closesocket(*w_);
        // wsa_assert(rc == 0);
        // errno = saved_errno;

        // error_closelistener: saved_errno = errno;
        // rc = closesocket(listener);
        // wsa_assert(rc == 0);

        //  Cleanup temporary socket file descriptor
        // if (!filename.empty()) {
        //     rc = ::unlink(filename.c_str());
        //     if ((rc == 0) && !dirname.empty()) {
        //         rc = ::rmdir(dirname.c_str());
        //         dirname.clear();
        //     }
        //     filename.clear();
        // }

        // ipc failed due to lack of AF_UNIX support, fallback on tcpip
        // if (ipc_fallback_on_tcpip) {
        //     goto
        //     try_tcpip;
        // }

        // errno = saved_errno;
        // return -1;

        // try_tcpip:
        // try to fallback to TCP/IP
        // TODO: maybe remember this decision permanently?
        // #endif
    }
    return make_fdZmqPaircpip(r_, w_);
    // #elif defined ZMQ_HAVE_OPENVMS
    if cfg!(target_os = "vms") {
        //  Whilst OpenVMS supports socketpair - it maps to AF_INET only.  Further,
        //  it does not set the socket options TCP_NODELAY and TCP_NODELACK which
        //  can lead to performance problems.
        //
        //  The bug will be fixed in V5.6 ECO4 and beyond.  In the meantime, we'll
        //  create the socket pair manually. struct sockaddr_in
        lcladdr;
        // memset(&lcladdr, 0, sizeof lcladdr);
        lcladdr.sin_family = AF_INET;
        // lcladdr.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
        lcladdr.sin_port = 0;

        // int
        // listener = open_socket(AF_INET, SOCK_STREAM, 0);
        // errno_assert (listener != -1);

        // int
        // on = 1;
        // int
        // rc = setsockopt(listener, IPPROTO_TCP, TCP_NODELAY, &on, sizeof on);
        // errno_assert (rc != -1);

        // rc = setsockopt(listener, IPPROTO_TCP, TCP_NODELACK, &on, sizeof on);
        // errno_assert (rc != -1);

        // rc = Bind(listener, (struct sockaddr
        // *) &lcladdr, sizeof
        // lcladdr);
        // errno_assert (rc != -1);

        // socklen_t
        // lcladdr_len = sizeof
        // lcladdr;

        // rc = getsockname(listener, (struct sockaddr
        // *) &lcladdr, &lcladdr_len);
        // errno_assert (rc != -1);

        // rc = listen(listener, 1);
        // errno_assert (rc != -1);

        // *w_ = open_socket(AF_INET, SOCK_STREAM, 0);
        // errno_assert (*w_ != -1);

        // rc = setsockopt(*w_, IPPROTO_TCP, TCP_NODELAY, &on, sizeof on);
        // errno_assert (rc != -1);

        // rc = setsockopt(*w_, IPPROTO_TCP, TCP_NODELACK, &on, sizeof on);
        // errno_assert (rc != -1);

        // rc = connect(*w_, (struct sockaddr
        // *) &lcladdr, sizeof
        // lcladdr);
        // errno_assert (rc != -1);

        // *r_ = accept(listener, null_mut(), null_mut());
        // errno_assert (*r_ != -1);

        // close(listener);

        return 0;
    }
    // #elif defined ZMQ_HAVE_VXWORKS
    if cfg!(target_os = "vxworks") {
        // struct sockaddr_in
        // lcladdr;
        // memset(&lcladdr, 0, sizeof lcladdr);
        // lcladdr.sin_family = AF_INET;
        // lcladdr.sin_addr.s_addr = htonl(INADDR_LOOPBACK);
        // lcladdr.sin_port = 0;
        //
        // int
        // listener = open_socket(AF_INET, SOCK_STREAM, 0);
        // // errno_assert (listener != -1);
        //
        // int
        // on = 1;
        // int
        // rc = setsockopt(listener, IPPROTO_TCP, TCP_NODELAY,  & on, sizeof on);
        // // errno_assert (rc != -1);
        //
        // rc = Bind(listener, (struct sockaddr
        // *) &lcladdr, sizeof
        // lcladdr);
        // // errno_assert (rc != -1);
        //
        // socklen_t
        // lcladdr_len = sizeof
        // lcladdr;
        //
        // rc = getsockname(listener, (struct sockaddr
        // *) &lcladdr,
        // (int *) & lcladdr_len);
        // // errno_assert (rc != -1);
        //
        // rc = listen(listener, 1);
        // // errno_assert (rc != -1);
        //
        // *w_ = open_socket(AF_INET, SOCK_STREAM, 0);
        // // errno_assert (*w_ != -1);
        //
        // rc = setsockopt(*w_, IPPROTO_TCP, TCP_NODELAY,  & on, sizeof on);
        // // errno_assert (rc != -1);
        //
        // rc = connect(*w_, (struct sockaddr
        // *) &lcladdr, sizeof
        // lcladdr);
        // // errno_assert (rc != -1);
        //
        // *r_ = accept(listener, null_mut(), null_mut());
        // // errno_assert (*r_ != -1);
        //
        // close(listener);
        //
        // return 0;
    }
    // #else
    else {
        //         // All other implementations support socketpair()
        //         int
        //         sv[2];
        //         int type = SOCK_STREAM;
        //         //  Setting this option result in sane behaviour when exec() functions
        //         //  are used. Old sockets are closed and don't block TCP ports, avoid
        //         //  leaks, etc.
        // // #if defined ZMQ_HAVE_SOCK_CLOEXEC type |= SOCK_CLOEXEC;
        // // #endif
        //         int
        //         rc = socketpair(AF_UNIX, type , 0, sv);
        //         if (rc == -1) {
        //             // errno_assert (errno == ENFILE || errno == EMFILE);
        //             *w_ = *r_ = -1;
        //             return -1;
        //         } else {
        //             make_socket_noninheritable(sv[0]);
        //             make_socket_noninheritable(sv[1]);
        //
        //             *w_ = sv[0];
        //             *r_ = sv[1];
        //             return 0;
        //         }
    }
    return 0;
    // #endif
}

pub fn make_socket_noninheritable(sock_: ZmqFileDesc) {
    // #if defined ZMQ_HAVE_WINDOWS && !defined _WIN32_WCE                            \
    //   && !defined ZMQ_HAVE_WINDOWS_UWP
    //  On Windows, preventing sockets to be inherited by child processes.
    let brc =
        unsafe { SetHandleInformation((sock_), HANDLE_FLAG_INHERIT as u32, 0 as HANDLE_FLAGS) };
    // win_assert (brc);
    // #elif (!defined ZMQ_HAVE_SOCK_CLOEXEC || !defined HAVE_ACCEPT4)                \
    //   && defined FD_CLOEXEC
    //  If there 's no SOCK_CLOEXEC, let's try the second best option.
    //  Race condition can cause socket not to be closed (if fork happens
    //  between accept and this point).
    #[cfg(target_feature = "fd_cloexec")]
    {
        let rc: i32 = unsafe { fcntl(sock_, F_SETFD, FD_CLOEXEC) };
    }
    // errno_assert (rc != -1);
    // #else
    //     LIBZMQ_UNUSED (sock_);
    // #endif
}

pub fn assert_success_or_recoverable(s_: ZmqFileDesc, rc_: i32) {
    // #ifdef ZMQ_HAVE_WINDOWS
    if rc_ != SOCKET_ERROR {
        return;
    }
    // #else
    if rc_ != -1 {
        return;
    }
    // #endif

    //  Check whether an error occurred
    let mut err = 0;
    // #if defined ZMQ_HAVE_HPUX || defined ZMQ_HAVE_VXWORKS
    let mut len = 4;
    // #else
    //     socklen_t len = sizeof err;
    // #endif

    let rc = unsafe { getsockopt(s_, SOL_SOCKET, SO_ERROR, (&mut err as PSTR), &mut len) };

    //  Assert if the error was caused by 0MQ bug.
    //  Networking problems are OK. No need to assert.
    // #ifdef ZMQ_HAVE_WINDOWS
    // zmq_assert (rc == 0);
    // if err != 0 {
    //     // wsa_assert (err == WSAECONNREFUSED || err == WSAECONNRESET
    //     //             || err == WSAECONNABORTED || err == WSAEINTR
    //     //             || err == WSAETIMEDOUT || err == WSAEHOSTUNREACH
    //     //             || err == WSAENETUNREACH || err == WSAENETDOWN
    //     //             || err == WSAENETRESET || err == WSAEACCES
    //     //             || err == WSAEINVAL || err == WSAEADDRINUSE);
    // }
    // #else
    //  Following code should handle both Berkeley-derived socket
    //  implementations and Solaris.
    // if rc == -1 {
    //     err = errno;
    // }
    // if err != 0 {
    //     errno = err;
    //     // errno_assert (errno == ECONNREFUSED || errno == ECONNRESET
    //     //               || errno == ECONNABORTED || errno == EINTR
    //     //               || errno == ETIMEDOUT || errno == EHOSTUNREACH
    //     //               || errno == ENETUNREACH || errno == ENETDOWN
    //     //               || errno == ENETRESET || errno == EINVAL);
    // }
    // #endif
}

// #ifdef ZMQ_HAVE_IPC

// #if defined ZMQ_HAVE_WINDOWS
// char *widechar_to_utf8 (const wchar_t *widestring)
// {
//     nch: i32, n;
//     char *utf8 = 0;
//     nch = WideCharToMultiByte (CP_UTF8, 0, widestring, -1, 0, 0, null_mut(), null_mut());
//     if (nch > 0) {
//         utf8 =  malloc ((nch + 1) * mem::size_of::<char>());
//         n = WideCharToMultiByte (CP_UTF8, 0, widestring, -1, utf8, nch, null_mut(),
//                                  null_mut());
//         utf8[nch] = 0;
//     }
//     return utf8;
// }
// #endif

pub fn create_ipc_wildcard_address(
    out_path: &mut String,
    out_file: &mut String,
) -> anyhow::Result<()> {
    // #if defined ZMQ_HAVE_WINDOWS
    #[cfg(target_os = "windows")]
    {
        // let mut buffer: [u8; MAX_PATH as usize] = [0; MAX_PATH as usize];
        let mut buffer: [u8; 260] = [0; 260];
        let mut get_temp_path_res = unsafe { GetTempPathA(Some(&mut buffer)) };
        if get_temp_path_res == 0 {
            return Err(anyhow::anyhow!("GetTempPathA failed"));
        }
        let create_dir_res = unsafe { CreateDirectoryA(buffer as PSTR, None) };
        if create_dir_res == false {
            return Err(anyhow::anyhow!("CreateDirectoryA failed"));
        }

        *out_path = String::from_utf8_lossy(&buffer).to_string();
        *out_file = out_path + "/socket";
        return Ok(());
    }
    // free (tmp);
    // #else
    #[cfg(not(target_os = "windows"))]
    {
        let mut tmp_path: String = String::new();

        // If TMPDIR, TEMPDIR, or TMP are available and are directories, create
        // the socket directory there.
        // TODO
        // const char **tmp_env = tmp_env_vars;
        // while (tmp_path.is_empty() && *tmp_env != 0) {
        //     const char *const tmpdir = getenv (*tmp_env);
        //     struct stat statbuf;
        //
        //     // Confirm it is actually a directory before trying to use
        //     if (tmpdir != 0 && ::stat (tmpdir, &statbuf) == 0
        //         && S_ISDIR (statbuf.st_mode)) {
        //         tmp_path.assign (tmpdir);
        //         if (*(tmp_path.rbegin ()) != '/') {
        //             tmp_path.push_back ('/');
        //         }
        //     }
        //
        //     // Try the next environment variable
        //     += 1tmp_env;
        // }

        // Append a directory name
        tmp_path.append("tmpXXXXXX");

        // We need room for tmp_path + trailing NUL
        // TODO
        // std::vector<char> buffer (tmp_path.length () + 1);
        // memcpy (&buffer[0], tmp_path, tmp_path.length () + 1);

        // #if defined HAVE_MKDTEMP
        // Create the directory.  POSIX requires that mkdtemp() creates the
        // directory with 0700 permissions, meaning the only possible race
        // with socket creation could be the same user.  However, since
        // each socket is created in a directory created by mkdtemp(), and
        // mkdtemp() guarantees a unique directory name, there will be no
        // collision.
        if unsafe { mkdtemp(&mut buffer[0] as *mut c_char) } == null_mut() {
            return -1;
        }

        path_.assign(&buffer[0]);
        *file_ = path_ + "/socket";
        // #else
        //     LIBZMQ_UNUSED (path_);
        let fd = unsafe { mkstemp(&mut buffer[0] as *mut c_char) };
        if (fd == -1) {
            return -1;
        }
        // ::close (fd);

        file_.assign(&buffer[0]);
        // #endif
        // #endif
    }
    Ok(())
}
// #endif
