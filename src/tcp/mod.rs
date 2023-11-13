use std::ffi::c_void;

use libc::{EAFNOSUPPORT, setsockopt};
#[cfg(target_os = "windows")]
use windows::Win32::Networking::WinSock::{recv, send, SIO_KEEPALIVE_VALS, SIO_LOOPBACK_FAST_PATH, SOCKET_ERROR, tcp_keepalive, WSAECONNABORTED, WSAECONNRESET, WSAEHOSTUNREACH, WSAENETDOWN, WSAENETRESET, WSAENOBUFS, WSAEOPNOTSUPP, WSAETIMEDOUT, WSAEWOULDBLOCK, WSAGetLastError, WSAIoctl};

use crate::address::tcp_address::ZmqTcpAddress;
use crate::defines::{AF_INET, AF_INET6, IPPROTO_TCP, RETIRED_FD, SO_RCVBUF, SO_SNDBUF, SOCK_STREAM, SOL_SOCKET, ZmqFd};
use crate::defines::err::ZmqError;
use crate::defines::tcp::TCP_NODELAY;
use crate::ip::{bind_to_device, enable_ipv4_mapping, open_socket, set_ip_type_of_service, set_socket_priority};
use crate::net::platform_socket::platform_setsockopt;
use crate::options::ZmqOptions;
use crate::utils::get_errno;

pub fn tune_tcp_socket(s_: ZmqFd) -> Result<(),ZmqError> {
    //  Disable Nagle's algorithm. We are doing data batching on 0MQ level,
    //  so using Nagle wouldn't improve throughput in anyway, but it would
    //  hurt latency.
    let mut nodelay = 1;
    // let rc = unsafe {
    //     setsockopt(s_, IPPROTO_TCP, TCP_NODELAY,
    //                nodelay.to_le_bytes().as_ptr() as *const c_void, 4)
    // };
    // // assert_success_or_recoverable (s_, rc);
    // if rc != 0 {
    //     return rc;
    // }
    platform_setsockopt(s_, IPPROTO_TCP, TCP_NODELAY, &nodelay.to_le_bytes(), 4)?;

// #ifdef ZMQ_HAVE_OPENVMS
//     //  Disable delayed acknowledgements as they hurt latency significantly.
//     int nodelack = 1;
//     rc = setsockopt (s_, IPPROTO_TCP, TCP_NODELACK, (char *) &nodelack,
//                      sizeof (int));
//     assert_success_or_recoverable (s_, rc);
// #endif
    return Ok(());
}

pub fn set_tcp_send_buffer(sockfd_: ZmqFd, bufsize_: i32) -> Result<(), ZmqError> {
    // let rc =
    //   setsockopt (sockfd_, SOL_SOCKET as libc::c_int, SO_SNDBUF,
    //               bufsize_.to_le_bytes().as_ptr() as *const c_void, 4);

    platform_setsockopt(sockfd_, SOL_SOCKET as libc::c_int, SO_SNDBUF,
                        &bufsize_.to_le_bytes(), 4)?;

    // assert_success_or_recoverable (sockfd_, rc);
    // return rc;
    Ok(())
}

pub fn set_tcp_receive_buffer(sockfd_: ZmqFd, bufsize_: i32) -> Result<(),ZmqError> {
    platform_setsockopt(sockfd_, SOL_SOCKET as libc::c_int, SO_RCVBUF,
                        &bufsize_.to_le_bytes(), 4)?;
    // assert_success_or_recoverable (sockfd_, rc);
    Ok(())
}

pub fn tune_tcp_keepalives(s_: ZmqFd,
                           keepalive_: i32,
                           keepalive_cnt_: i32,
                           keepalive_idle_: i32,
                           keepalive_intvl_: i32) -> i32 {
    // These options are used only under certain #ifdefs below.
    // LIBZMQ_UNUSED (keepalive_);
    // LIBZMQ_UNUSED (keepalive_cnt_);
    // LIBZMQ_UNUSED (keepalive_idle_);
    // LIBZMQ_UNUSED (keepalive_intvl_);

    // If none of the #ifdefs apply, then s_ is unused.
    // LIBZMQ_UNUSED (s_);

    //  Tuning TCP keep-alives if platform allows it
    //  All values = -1 means skip and leave it for OS
// #ifdef ZMQ_HAVE_WINDOWS
    #[cfg(target_os = "windows")]
    {
        if keepalive_ != -1 {
            let mut keepalive_opts: tcp_keepalive = tcp_keepalive::default();
            keepalive_opts.onoff = keepalive_ as u32;
            keepalive_opts.keepalivetime = if keepalive_idle_ != -1 {
                keepalive_idle_ * 1000
            } else { 7200000 } as u32;
            keepalive_opts.keepaliveinterval = if keepalive_intvl_ != -1 {
                keepalive_intvl_ * 1000
            } else { 1000 } as u32;
            let mut num_bytes_returned = 0u32;
            let mut rc = 0i32;
            unsafe { rc = WSAIoctl(s_, SIO_KEEPALIVE_VALS as i32, &keepalive_opts, None, 0, &mut num_bytes_returned, None, 0) };
            // assert_success_or_recoverable(s_, rc);
            if rc == SOCKET_ERROR {
                return rc;
            }
        }
    }
// #else
    #[cfg(not(target_os = "windows"))]
    {
// #ifdef ZMQ_HAVE_SO_KEEPALIVE
        #[cfg(feature = "so_keepalive")]
        {
            if (keepalive_ != -1) {
                let setsockopt(s_, SOL_SOCKET, SO_KEEPALIVE,
                               (&keepalive_), 4);
                // assert_success_or_recoverable (s_, rc);
                if (rc != 0) {
                    return rc;
                }

// #ifdef ZMQ_HAVE_TCP_KEEPCNT
                #[cfg(feature = "tcp_keepcnt")]
                {
                    if (keepalive_cnt_ != -1) {
                        let rc = setsockopt(s_, IPPROTO_TCP, TCP_KEEPCNT, &keepalive_cnt_,
                                            4);
                        // assert_success_or_recoverable (s_, rc);
                        if (rc != 0) {
                            return rc;
                        }
                    }
                }
// #endif // ZMQ_HAVE_TCP_KEEPCNT

// #ifdef ZMQ_HAVE_TCP_KEEPIDLE
                #[cfg(feature = "tcp_keepidle")]
                {
                    if (keepalive_idle_ != -1) {
                        let rc = setsockopt(s_, IPPROTO_TCP, TCP_KEEPIDLE,
                                            &keepalive_idle_, 4);
                        // assert_success_or_recoverable (s_, rc);
                        if (rc != 0) {
                            return rc;
                        }
                    }
                }
// #else // ZMQ_HAVE_TCP_KEEPIDLE
                #[cfg(not(feature = "tcp_keepidle"))]
                {
// #ifdef ZMQ_HAVE_TCP_KEEPALIVE
                    #[cfg(feature = "tcp_keepalive")]
                    {
                        if (keepalive_idle_ != -1) {
                            let rc = setsockopt(s_, IPPROTO_TCP, TCP_KEEPALIVE,
                                                &keepalive_idle_, 4);
                            // assert_success_or_recoverable (s_, rc);
                            if (rc != 0) {
                                return rc;
                            }
                        }
                    }
// #endif // ZMQ_HAVE_TCP_KEEPALIVE
// #endif // ZMQ_HAVE_TCP_KEEPIDLE
                }
// #ifdef ZMQ_HAVE_TCP_KEEPINTVL
                #[cfg(feature = "tcp_keepintvl")]
                {
                    if (keepalive_intvl_ != -1) {
                        let rc = setsockopt(s_, IPPROTO_TCP, TCP_KEEPINTVL,
                                            &keepalive_intvl_, 4);
                        // assert_success_or_recoverable (s_, rc);
                        if (rc != 0) {
                            return rc;
                        }
                    }
                }
// #endif // ZMQ_HAVE_TCP_KEEPINTVL
            }
// #endif // ZMQ_HAVE_SO_KEEPALIVE
        }
// #endif // ZMQ_HAVE_WINDOWS
    }
    return 0;
}

pub fn tune_tcp_maxrt(sockfd_: ZmqFd, timeout_: i32) -> i32 {
    if timeout_ <= 0 {
        return 0;
    }

    // LIBZMQ_UNUSED (sockfd_);

// #if defined(ZMQ_HAVE_WINDOWS) && defined(TCP_MAXRT)
    #[cfg(target_os = "windows")]
    {
        #[cfg(feature = "tcp_maxrt")]
        {
            // msdn says it's supported in >= Vista, >= Windows Server 2003
            timeout_ /= 1000; // in seconds
            let rc = setsockopt(sockfd_, IPPROTO_TCP, TCP_MAXRT,
                                (&timeout_), 4);
            // assert_success_or_recoverable (sockfd_, rc);
            return rc;
        }
    }
// FIXME: should be ZMQ_HAVE_TCP_USER_TIMEOUT
// #elif defined(TCP_USER_TIMEOUT)
    unsafe {
        #[cfg(not(target_os = "windows"))]
        {
            let rc = setsockopt(sockfd_, IPPROTO_TCP, TCP_USER_TIMEOUT, timeout_.to_le_bytes().as_ptr() as *const c_void,
                                4);
            // assert_success_or_recoverable (sockfd_, rc);
            return rc;
        }
    }
    // #else
    return 0;
// #endif
}

pub fn tcp_write(s_: ZmqFd, data_: &[u8], size_: usize) -> i32 {
// #ifdef ZMQ_HAVE_WINDOWS
    #[cfg(target_os = "windows")]
    {
        // let nbytes = send(s_, data_, (size_), 0);
        let nbytes = platform_send(s_, data_, 0)?;

        //  If not a single byte can be written to the socket in non-blocking mode
        //  we'll get an Error (this may happen during the speculative write).
        let last_error = WSAGetLastError();
        if nbytes == SOCKET_ERROR && last_error == WSAEWOULDBLOCK {
            return 0;
        }

        //  Signalise peer failure.
        if nbytes == SOCKET_ERROR && (last_error == WSAENETDOWN || last_error == WSAENETRESET || last_error == WSAEHOSTUNREACH || last_error == WSAECONNABORTED || last_error == WSAETIMEDOUT || last_error == WSAECONNRESET) {
            return -1;
        }

        //  Circumvent a Windows bug:
        //  See https://support.microsoft.com/en-us/kb/201213
        //  See https://zeromq.jira.com/browse/LIBZMQ-195
        if nbytes == SOCKET_ERROR && last_error == WSAENOBUFS {
            return 0;
        }

        // wsa_assert (nbytes != SOCKET_ERROR);
        return nbytes;
    }
// #else
    unsafe {
        let mut nbytes = 0isize;
        #[cfg(not(target_os = "windows"))]
        {
            nbytes = libc::send(s_, data_.as_ptr() as *const c_void, size_, 0);

            //  Several errors are OK. When speculative write is being Done we may not
            //  be able to write a single byte from the socket. Also, SIGSTOP issued
            //  by a debugging tool can result in EINTR Error.
            if nbytes == -1 && (get_errno() == EAGAIN || get_errno() == EWOULDBLOCK || get_errno() == EINTR) {
                return 0;
            }

            //  Signalise peer failure.
            if nbytes == -1 {
// #if !defined(TARGET_OS_IPHONE) || !TARGET_OS_IPHONE
//         errno_assert (errno != EACCES && errno != EBADF && errno != EDESTADDRREQ
//                       && errno != EFAULT && errno != EISCONN
//                       && errno != EMSGSIZE && errno != ENOMEM
//                       && errno != ENOTSOCK && errno != EOPNOTSUPP);
// #else
//         errno_assert (errno != EACCES && errno != EDESTADDRREQ
//                       && errno != EFAULT && errno != EISCONN
//                       && errno != EMSGSIZE && errno != ENOMEM
//                       && errno != ENOTSOCK && errno != EOPNOTSUPP);
// #endif
                return -1;
            }
        }

        return nbytes as i32;
    }
// #endif
}

pub fn tcp_read(s_: ZmqFd, data_: &mut [u8], size_: usize) -> i32 {
// #ifdef ZMQ_HAVE_WINDOWS
    #[cfg(target_os = "windows")]
    {
        let mut rc = 0;
        unsafe { rc = recv(s_, (data_), 0); };

        //  If not a single byte can be read from the socket in non-blocking mode
        //  we'll get an Error (this may happen during the speculative read).
        if (rc == SOCKET_ERROR) {
            let last_error = WSAGetLastError();
            if (last_error == WSAEWOULDBLOCK) {
                // errno = EAGAIN;
            } else {
                // wsa_assert (
                //   last_error == WSAENETDOWN || last_error == WSAENETRESET
                //   || last_error == WSAECONNABORTED || last_error == WSAETIMEDOUT
                //   || last_error == WSAECONNRESET || last_error == WSAECONNREFUSED
                //   || last_error == WSAENOTCONN || last_error == WSAENOBUFS);
                // errno = wsa_error_to_errno (last_error);
            }
        }

        return rc = if SOCKET_ERROR { -1 } else { rc };
    }
// #else
    unsafe {
        #[cfg(not(target_os = "windows"))]
        {
            let rc = libc::recv(s_, data_.as_mut_ptr() as *mut c_void, size_, 0);

            //  Several errors are OK. When speculative read is being Done we may not
            //  be able to read a single byte from the socket. Also, SIGSTOP issued
            //  by a debugging tool can result in EINTR Error.
            if rc == -1 {
// #if !defined(TARGET_OS_IPHONE) || !TARGET_OS_IPHONE
//         errno_assert (errno != EBADF && errno != EFAULT && errno != ENOMEM
//                       && errno != ENOTSOCK);
// #else
//         errno_assert (errno != EFAULT && errno != ENOMEM && errno != ENOTSOCK);
// #endif
//         if (errno == EWOULDBLOCK || errno == EINTR) {
//             errno = EAGAIN;
//         }
            }

            return rc as i32;
        }
    }
// #endif
}

pub fn tcp_tune_loopback_fast_path(socket_: ZmqFd) {
// #if defined ZMQ_HAVE_WINDOWS && defined SIO_LOOPBACK_FAST_PATH
    #[cfg(target_os = "windows")]
    {
        let mut sio_loopback_fastpath = 1;
        let mut number_of_bytes_returned = 0;

        let mut rc = WSAIoctl(
            socket_, SIO_LOOPBACK_FAST_PATH, &sio_loopback_fastpath,
            size_of::<sio_loopback_fastpath>, null_mut(), 0, &number_of_bytes_returned, 0, 0);

        if SOCKET_ERROR == rc {
            let last_error = WSAGetLastError();

            if WSAEOPNOTSUPP == last_error {
                // This system is not Windows 8 or Server 2012, and the call is not supported.
            } else {
                // wsa_assert (false);
            }
        }
    }
// #else
    // LIBZMQ_UNUSED (socket_);
// #endif
}

pub fn tune_tcp_busy_poll(socket: ZmqFd, busy_poll: i32) -> Result<(),ZmqError> {
// #if defined(ZMQ_HAVE_BUSY_POLL)
    if busy_poll > 0 {
        // let rc = setsockopt(socket_, SOL_SOCKET as libc::c_int, SO_BUSY_POLL,
        //                     busy_poll_.to_le_bytes().as_ptr() as *const c_void, 4);
        platform_setsockopt(socket, SOL_SOCKET as i32, SO_BUSY_POLL, &busy_poll.to_le_bytes(), 4)?;
        // assert_success_or_recoverable (socket_, rc);
    }
// #else
//     LIBZMQ_UNUSED (socket_);
//     LIBZMQ_UNUSED (busy_poll_);
// #endif
    Ok(())
}

pub fn tcp_open_socket(addr_str: &mut String,
                       options: &ZmqOptions,
                       local_: bool,
                       fallback_to_ipv4_: bool,
                       out_tcp_addr_: &mut ZmqTcpAddress) -> ZmqFd {
    //  Convert the textual address into address structure.
    out_tcp_addr_.resolve(options, addr_str, local_, options.ipv6)?;
    // if rc != 0 {
    //     return RETIRED_FD;
    // }

    //  Create the socket.
    let mut s = open_socket(out_tcp_addr_.family() as i32, SOCK_STREAM, IPPROTO_TCP)?;

    //  IPv6 address family not supported, try automatic downgrade to IPv4.
    if s == RETIRED_FD && fallback_to_ipv4_ && out_tcp_addr_.family() == AF_INET6 as u16 && get_errno() == EAFNOSUPPORT && options.ipv6 {
        out_tcp_addr_.resolve(options, addr_str, local_, false)?;
        // if rc != 0 {
        //     return RETIRED_FD;
        // }
        s = open_socket(AF_INET, SOCK_STREAM, IPPROTO_TCP)?;
    }

    if s == RETIRED_FD {
        return RETIRED_FD;
    }

    //  On some systems, IPv4 mapping in IPv6 sockets is disabled by default.
    //  Switch it on in such cases.
    if out_tcp_addr_.family() == AF_INET6 as u16 {
        enable_ipv4_mapping(s)?;
    }

    // Set the IP Type-Of-Service priority for this socket
    if options.tos != 0 {
        set_ip_type_of_service(s, options.tos)?;
    }

    // Set the protocol-defined priority for this socket
    if options.priority != 0 {
        set_socket_priority(s, options.priority)?;
    }

    // Set the socket to loopback fastpath if configured.
    if options.loopback_fastpath {
        tcp_tune_loopback_fast_path(s)?;
    }

    // Bind the socket to a device if applicable
    if !options.bound_device.empty() {
        if bind_to_device(s, &options.bound_device)? {
            // goto setsockopt_error;
        }
    }

    //  Set the socket buffer limits for the underlying socket.
    if options.sndbuf >= 0 {
        set_tcp_send_buffer(s, options.sndbuf);
    }
    if options.rcvbuf >= 0 {
        set_tcp_receive_buffer(s, options.rcvbuf);
    }

    //  This option removes several delays caused by scheduling, interrupts and context switching.
    if options.busy_poll {
        tune_tcp_busy_poll(s, options.busy_poll);
    }
    return s;

    // TODO
// // setsockopt_error:
// // #ifdef ZMQ_HAVE_WINDOWS
// #[cfg(target_os="windows")]{
//     rc = closesocket(s);
//     // wsa_assert(rc != SOCKET_ERROR);
// }
// // #else
//     #[cfg(not(target_os="windows"))]{
//         rc = ::close(s);
//         errno_assert(rc == 0);
//     }
// // #endif
//     return RETIRED_FD;
}
