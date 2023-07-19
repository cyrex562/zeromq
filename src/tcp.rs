use std::mem;
use std::ptr::null_mut;
use libc::{accept, bind, c_int, close, EAFNOSUPPORT, EAGAIN, EFAULT, EINTR, EISCONN, EMSGSIZE, ENOMEM, ENOTSOCK, EOPNOTSUPP, EWOULDBLOCK, listen, setsockopt, sockaddr, SOCKET, ssize_t};
use windows::Win32::Networking::WinSock::{closesocket, IPPROTO_TCP, recv, send, SEND_RECV_FLAGS, SIO_KEEPALIVE_VALS, SIO_LOOPBACK_FAST_PATH, SO_KEEPALIVE, SO_RCVBUF, SO_REUSEADDR, SO_SNDBUF, SOCK_STREAM, SOCKADDR_STORAGE, SOCKET_ERROR, SOL_SOCKET, tcp_keepalive, TCP_KEEPALIVE, TCP_KEEPCNT, TCP_KEEPIDLE, TCP_KEEPINTVL, TCP_MAXRT, TCP_NODELAY, WSA_ERROR, WSAECONNABORTED, WSAECONNREFUSED, WSAECONNRESET, WSAEHOSTUNREACH, WSAENETDOWN, WSAENETRESET, WSAENOBUFS, WSAENOTCONN, WSAEOPNOTSUPP, WSAETIMEDOUT, WSAEWOULDBLOCK, WSAGetLastError};
use std::mem::size_of_val;
use anyhow::bail;
use bincode::options;
use crate::address::{get_socket_name, ZmqAddress, ZmqSocketEnd};
use crate::address_family::{AF_INET, AF_INET6};
use crate::context::ZmqContext;
use crate::defines::RETIRED_FD;
use crate::endpoint::make_unconnected_bind_endpoint_pair;
use crate::err::wsa_error_to_errno;
use crate::defines::ZmqFileDesc;
use crate::ip::{assert_success_or_recoverable, bind_to_device, enable_ipv4_mapping, make_socket_noninheritable, ip_open_socket, set_ip_type_of_service, set_nosigpipe, set_socket_priority};
use crate::listener::ZmqListener;

use crate::tcp_address::TcpAddress;

pub fn tune_tcp_socket (s_: &mut ZmqFileDesc) -> i32
{
    //  Disable Nagle's algorithm. We are doing data batching on 0MQ level,
    //  so using Nagle wouldn't improve throughput in anyway, but it would
    //  hurt latency.
    let mut nodelay = 1;
    let mut rc: i32 = unsafe {
        setsockopt(s_ as SOCKET, IPPROTO_TCP as c_int, TCP_NODELAY,
                   (&nodelay), 4)
    };
    // assert_success_or_recoverable (s_, rc);
    if rc != 0 {
        return rc;
    }

// #ifdef ZMQ_HAVE_OPENVMS
    //  Disable delayed acknowledgements as they hurt latency significantly.
    // let nodelack = 1;
    // unsafe {
    //     rc = setsockopt(s_, IPPROTO_TCP as c_int, TCP_NODELACK, &nodelack,
    //                     4);
    // }
    // assert_success_or_recoverable (s_, rc);
// #endif
    return rc;
}

pub fn set_tcp_send_buffer (sockfd_: &mut ZmqFileDesc, bufsize_: i32) -> i32
{
    // TODO
    // let rc: i32 = unsafe {
    //     setsockopt(sockfd_, SOL_SOCKET, SO_SNDBUF,
    //                (bufsize_.to_le_bytes().as_mut_ptr()), 4)
    // };
    assert_success_or_recoverable (sockfd_.clone(), rc);
    return rc;
}

pub fn set_tcp_receive_buffer (sockfd_: &mut ZmqFileDesc, bufsize_: i32)
{
    // TODO
    // let rc: i32 = unsafe {
    //     setsockopt(sockfd_, SOL_SOCKET, SO_RCVBUF,
    //                (&bufsize_), sizeof bufsize_)
    // };
    assert_success_or_recoverable (sockfd_.clone(), rc);
    return rc;
}

pub fn tune_tcp_keepalives (s_: ZmqFileDesc,
                              keepalive_: i32,
                              keepalive_cnt_: i32,
                              keepalive_idle_: i32,
                              keepalive_intvl_: i32) -> i32
{
    // These options are used only under certain // #ifdefs below.
    // LIBZMQ_UNUSED (keepalive_);
    // LIBZMQ_UNUSED (keepalive_cnt_);
    // LIBZMQ_UNUSED (keepalive_idle_);
    // LIBZMQ_UNUSED (keepalive_intvl_);

    // If none of the // #ifdefs apply, then s_ is unused.
    // LIBZMQ_UNUSED (s_);

    //  Tuning TCP keep-alives if platform allows it
    //  All values = -1 means skip and leave it for OS
// #ifdef ZMQ_HAVE_WINDOWS
    if (keepalive_ != -1) {
        let mut keepalive_opts: tcp_keepalive = tcp_keepalive::default();
        keepalive_opts.onoff = keepalive_ as u32;
        keepalive_opts.keepalivetime = if keepalive_idle_ != -1 { keepalive_idle_ * 1000 } else { 7200000 } as u32;
        keepalive_opts.keepaliveinterval = if keepalive_intvl_ != -1 { keepalive_intvl_ * 1000 } else { 1000 } as u32;
        let mut num_bytes_returned =0u32;
        let rc: i32 = WSAIoctl (s_, SIO_KEEPALIVE_VALS, &keepalive_opts,
                                 mem::size_of::<keepalive_opts>(), null_mut(), 0,
                                 &num_bytes_returned, null_mut(), null_mut());
        assert_success_or_recoverable (s_, rc);
        if (rc == SOCKET_ERROR) {
            return rc;
        }
    }
// #else
// #ifdef ZMQ_HAVE_SO_KEEPALIVE
    if (keepalive_ != -1) {
        unsafe {
            // TODO
            // rc = setsockopt(s_, SOL_SOCKET, SO_KEEPALIVE,
            //                 (&keepalive_), 4);
        }
        assert_success_or_recoverable (s_, rc);
        if (rc != 0) {
            return rc;
        }

// #ifdef ZMQ_HAVE_TCP_KEEPCNT
        if (keepalive_cnt_ != -1) {
            unsafe {
                // TODO
                // rc = setsockopt(s_, IPPROTO_TCP, TCP_KEEPCNT, &keepalive_cnt_,
                //                 4);
            }
            assert_success_or_recoverable (s_, rc);
            if (rc != 0) {
                return rc;
            }
        }
// #endif // ZMQ_HAVE_TCP_KEEPCNT

// #ifdef ZMQ_HAVE_TCP_KEEPIDLE
//         if (keepalive_idle_ != -1) {
//             unsafe {
//                 rc = setsockopt(s_, IPPROTO_TCP, TCP_KEEPIDLE,
//                                 &keepalive_idle_, 4);
//             }
//             assert_success_or_recoverable (s_, rc);
//             if (rc != 0) {
//                 return rc;
//             }
//         }
// #else // ZMQ_HAVE_TCP_KEEPIDLE
// #ifdef ZMQ_HAVE_TCP_KEEPALIVE
//         if (keepalive_idle_ != -1) {
//             unsafe {
//                 rc = setsockopt(s_, IPPROTO_TCP, TCP_KEEPALIVE,
//                                 &keepalive_idle_, 4);
//             }
//             assert_success_or_recoverable (s_, rc);
//             if (rc != 0) {
//                 return rc;
//             }
//         }
// #endif // ZMQ_HAVE_TCP_KEEPALIVE
// #endif // ZMQ_HAVE_TCP_KEEPIDLE

// #ifdef ZMQ_HAVE_TCP_KEEPINTVL
//         if (keepalive_intvl_ != -1) {
//             unsafe {
//                 rc = setsockopt(s_, IPPROTO_TCP, TCP_KEEPINTVL,
//                                 &keepalive_intvl_, 4);
//             }
//             assert_success_or_recoverable (s_, rc);
//             if (rc != 0) {
//                 return rc;
//             }
//         }
// #endif // ZMQ_HAVE_TCP_KEEPINTVL
    }
// #endif // ZMQ_HAVE_SO_KEEPALIVE
// #endif // ZMQ_HAVE_WINDOWS

    return 0;
}

pub fn tune_tcp_maxrt (sockfd_: &mut ZmqFileDesc, mut timeout: i32) -> i32
{
    if (timeout <= 0) {
        return 0;
    }

    LIBZMQ_UNUSED (sockfd_);

// #if defined(ZMQ_HAVE_WINDOWS) && defined(TCP_MAXRT)
    // msdn says it's supported in >= Vista, >= Windows Server 2003
    timeout /= 1000; // in seconds
    // let rc: i32 = unsafe {
    //     setsockopt(sockfd_, IPPROTO_TCP, TCP_MAXRT,
    //                (&timeout), mem::size_of::<timeout>())
    // };
    // assert_success_or_recoverable (sockfd_, rc);
    return rc;
// FIXME: should be ZMQ_HAVE_TCP_USER_TIMEOUT
// #elif defined(TCP_USER_TIMEOUT)
//     int rc = setsockopt (sockfd_, IPPROTO_TCP, TCP_USER_TIMEOUT, &timeout,
//                          mem::size_of::<timeout>());
//     assert_success_or_recoverable (sockfd_, rc);
//     return rc;
// #else
//     return 0;
// #endif
}

pub fn tcp_write (s_: ZmqFileDesc, data: &mut [u8], size: usize) -> i32
{
// #ifdef ZMQ_HAVE_WINDOWS

    let nbytes: i32 = unsafe { send(s_, data, 0 as SEND_RECV_FLAGS) };

    //  If not a single byte can be written to the socket in non-blocking mode
    //  we'll get an error (this may happen during the speculative write).
    let last_error: WSA_ERROR = unsafe { WSAGetLastError() };
    if nbytes == SOCKET_ERROR && last_error == WSAEWOULDBLOCK {
        return 0;
    }

    //  Signalise peer failure.
    if nbytes == SOCKET_ERROR
        && (last_error == WSAENETDOWN || last_error == WSAENETRESET
            || last_error == WSAEHOSTUNREACH || last_error == WSAECONNABORTED
            || last_error == WSAETIMEDOUT || last_error == WSAECONNRESET) {
        return -1;
    }

    //  Circumvent a Windows bug:
    //  See https://support.microsoft.com/en-us/kb/201213
    //  See https://zeromq.jira.com/browse/LIBZMQ-195
    if nbytes == SOCKET_ERROR && last_error == WSAENOBUFS {
        return 0;
    }

    wsa_assert (nbytes != SOCKET_ERROR);
    return nbytes;

// #else
//     let nbytes = send (s_, (data), size, 0);
//
//     //  Several errors are OK. When speculative write is being Done we may not
//     //  be able to write a single byte from the socket. Also, SIGSTOP issued
//     //  by a debugging tool can result in EINTR error.
//     if (nbytes == -1
//         && (errno == EAGAIN || errno == EWOULDBLOCK || errno == EINTR)) {
//         return 0;
//     }
//
//     //  Signalise peer failure.
//     if (nbytes == -1) {
// // #if !defined(TARGET_OS_IPHONE) || !TARGET_OS_IPHONE
//         // errno_assert (errno != EACCES && errno != EBADF && errno != EDESTADDRREQ
//                       && errno != EFAULT && errno != EISCONN
//                       && errno != EMSGSIZE && errno != ENOMEM
//                       && errno != ENOTSOCK && errno != EOPNOTSUPP);
// // #else
//         // errno_assert (errno != EACCES && errno != EDESTADDRREQ
//                       && errno != EFAULT && errno != EISCONN
//                       && errno != EMSGSIZE && errno != ENOMEM
//                       && errno != ENOTSOCK && errno != EOPNOTSUPP);
// // #endif
//         return -1;
//     }
//
//     return  (nbytes);

// #endif
}

pub fn tcp_read (s_: ZmqFileDesc, data: &mut [u8], size: usize) -> i32
{
// #ifdef ZMQ_HAVE_WINDOWS

    let rc: i32 = unsafe { recv(s_, (data),  0 as SEND_RECV_FLAGS) };

    //  If not a single byte can be read from the socket in non-blocking mode
    //  we'll get an error (this may happen during the speculative read).
    unsafe {
        if rc == SOCKET_ERROR {
            let last_error: WSA_ERROR = WSAGetLastError();
            if last_error == WSAEWOULDBLOCK {
              // errno = EAGAIN;
            } else {
                wsa_assert(
                    last_error == WSAENETDOWN || last_error == WSAENETRESET || last_error == WSAECONNABORTED || last_error == WSAETIMEDOUT || last_error == WSAECONNRESET || last_error == WSAECONNREFUSED || last_error == WSAENOTCONN || last_error == WSAENOBUFS);
              // errno = wsa_error_to_errno(last_error);
            }
        }
    }

    return if rc == SOCKET_ERROR { -1 }else { rc };

// #else

//     const ssize_t rc = recv (s_,  (data), size, 0);
//
//     //  Several errors are OK. When speculative read is being Done we may not
//     //  be able to read a single byte from the socket. Also, SIGSTOP issued
//     //  by a debugging tool can result in EINTR error.
//     if (rc == -1) {
// // #if !defined(TARGET_OS_IPHONE) || !TARGET_OS_IPHONE
//         // errno_assert (errno != EBADF && errno != EFAULT && errno != ENOMEM
//                       && errno != ENOTSOCK);
// // #else
//         // errno_assert (errno != EFAULT && errno != ENOMEM && errno != ENOTSOCK);
// // #endif
//         if (errno == EWOULDBLOCK || errno == EINTR)
//           // errno = EAGAIN;
//     }
//
//     return  (rc);

// #endif
}

pub fn tcp_tune_loopback_fast_path (socket: ZmqFileDesc)
{
// #if defined ZMQ_HAVE_WINDOWS && defined SIO_LOOPBACK_FAST_PATH
    let sio_loopback_fastpath = 1;
    let mut number_of_bytes_returned = 0;

    // let rc: i32 = WSAIoctl (
    //   socket, SIO_LOOPBACK_FAST_PATH, &sio_loopback_fastpath,
    //   sizeof sio_loopback_fastpath, null_mut(), 0, &number_of_bytes_returned, 0, 0);

    unsafe {
        if (SOCKET_ERROR == rc) {
            let last_error = WSAGetLastError();

            if (WSAEOPNOTSUPP == last_error) {
                // This system is not Windows 8 or Server 2012, and the call is not supported.
            } else {
                wsa_assert(false);
            }
        }
    }
// #else
//     LIBZMQ_UNUSED (socket);
// #endif
}

pub fn tune_tcp_busy_poll (socket: &mut ZmqFileDesc, busy_poll_: i32)
{
// #if defined(ZMQ_HAVE_BUSY_POLL)
    unsafe {
        if (busy_poll_ > 0) {
            // TODO
            // let rc: i32 = setsockopt(socket, SOL_SOCKET, SO_BUSY_POLL,
            //                          (&busy_poll_), 4);
            assert_success_or_recoverable(socket.clone(), rc);
        }
    }
// #else
//     LIBZMQ_UNUSED (socket);
//     LIBZMQ_UNUSED (busy_poll_);
// #endif
}

pub fn tcp_open_socket (address: &mut str,
                        ctx: &ZmqContext,
                        local: bool,
                        fallback_to_ipv4: bool,
                        out_addr: &mut ZmqAddress) -> anyhow::Result<ZmqFileDesc>
{
    //  Convert the textual address into address structure.
    // resolve (address_, local, ctx.ipv6)
    out_addr.resolve (address)?;
    let mut out: ZmqFileDesc = RETIRED_FD as ZmqFileDesc;

    //  Create the socket.
    match ip_open_socket(out_addr.family (), SOCK_STREAM as i32, IPPROTO_TCP as i32) {
        Ok(s) => out = s,
        Err(e) => {
            
        }
    }

    //  IPv6 address family not supported, try automatic downgrade to IPv4.
    if (s == RETIRED_FD && fallback_to_ipv4
        && out_addr.family () == AF_INET6 && errno == EAFNOSUPPORT
        && options_.ipv6) {
        rc = out_addr.resolve (address, local, false);
        if (rc != 0) {
            return RETIRED_FD;
        }
        s = ip_open_socket(AF_INET as i32, SOCK_STREAM as i32, IPPROTO_TCP as i32);
    }

    if (s == RETIRED_FD) {
        return RETIRED_FD;
    }

    //  On some systems, IPv4 mapping in IPv6 sockets is disabled by default.
    //  Switch it on in such cases.
    if (out_addr.family () == AF_INET6) {
        enable_ipv4_mapping(s);
    }

    // Set the IP Type-Of-Service priority for this socket
    if (options_.tos != 0) {
        set_ip_type_of_service(s, options_.tos);
    }

    // Set the protocol-defined priority for this socket
    if (options_.priority != 0) {
        set_socket_priority(s, options_.priority);
    }

    // Set the socket to loopback fastpath if configured.
    if (options_.loopback_fastpath) {
        tcp_tune_loopback_fast_path(s);
    }

    // Bind the socket to a device if applicable
    if (!options_.bound_device.empty ()) {
        if (bind_to_device(s, options_.bound_device) == -1) {
            // goto
            // setsockopt_error;
        }
    }

    //  Set the socket buffer limits for the underlying socket.
    if (options_.sndbuf >= 0) {
        set_tcp_send_buffer(&mut s, options_.sndbuf);
    }
    if (options_.rcvbuf >= 0) {
        set_tcp_receive_buffer(&mut s, options_.rcvbuf);
    }

    //  This option removes several delays caused by scheduling, interrupts and context switching.
    if (options_.busy_poll) {
        tune_tcp_busy_poll(&mut s, options_.busy_poll);
    }
    return s;

    // TODO
// setsockopt_error:
// // #ifdef ZMQ_HAVE_WINDOWS
//     rc = closesocket (s);
//     wsa_assert (rc != SOCKET_ERROR);
// // #else
//     rc = ::close (s);
//     // errno_assert (rc == 0);
// // #endif
//     return retired_fd;
}


pub fn tcp_in_event(listener: &mut ZmqListener) -> anyhow::Result<()> {
    let mut tcp_sockaddr = sockaddr { sa_family: AF_INET, sa_data: [0; 14] };
    tcp_sockaddr.sa_data = listener.address.addr_sockaddr.sa_data;
    let mut tcp_sockaddr_len = size_of_val(&tcp_sockaddr) as c_int;
    let mut fd = unsafe {
        accept(listener.fd,
               &mut listener.address.addr_sockaddr as *mut sockaddr,
               &mut tcp_sockaddr_len)
    };

    //  If connection was reset by the peer in the meantime, just ignore it.
    //  TODO: Handle specific errors like ENFILE/EMFILE etc.
    if (fd == RETIRED_FD as usize) {
        listener.socket
            .event_accept_failed(&make_unconnected_bind_endpoint_pair(&listener.endpoint), zmq_errno());
        bail!("tcp_in_event failed");
    }

    let mut rc = tune_tcp_socket(&mut fd);
    rc = rc
        | tune_tcp_keepalives(
        fd,
        options.tcp_keepalive,
        options.tcp_keepalive_cnt,
        options.tcp_keepalive_idle,
        options.tcp_keepalive_intvl,
    );
    rc = rc | tune_tcp_maxrt(&mut fd, options.tcp_maxrt);
    if (rc != 0) {
        listener.socket
            .event_accept_failed(&make_unconnected_bind_endpoint_pair(&listener.endpoint), zmq_errno());
        bail!("tcp_in_event failed");
    }

    //  Create the engine object for this connection.
    listener.create_engine();
    Ok(())
}

pub fn tcp_create_socket(listener: &mut ZmqListener, addr_: &mut str) -> anyhow::Result<()> {
    listener.fd = tcp_open_socket(addr_, listener.socket.context, true, true, &mut listener.address);
    if (listener.fd == RETIRED_FD as usize) {
        bail!("tcp_create_socket failed");
    }

    //  TODO why is this only Done for the listener?
    make_socket_noninheritable(listener.fd);

    //  Allow reusing of the address.
    let mut flag = 1;
    let mut rc = 0i32;
    // #ifdef ZMQ_HAVE_WINDOWS
    //  TODO this was changed for Windows from SO_REUSEADDRE to
    //  SE_EXCLUSIVEADDRUSE by 0ab65324195ad70205514d465b03d851a6de051c,
    //  so the comment above is no longer correct; also, now the settings are
    //  different between listener and connecter with a src address.
    //  is this intentional?

    // unsafe {
    //     rc = setsockopt(listener.fd, SOL_SOCKET, SO_EXCLUSIVEADDRUSE, (&flag), 4);
    // }
    // wsa_assert(rc != SOCKET_ERROR);
    // #elif defined ZMQ_HAVE_VXWORKS
    //     rc =
    //       setsockopt (_s, SOL_SOCKET, SO_REUSEADDR,  &flag, mem::size_of::<int>());
    //     // errno_assert (rc == 0);
    // // #else
    unsafe { rc = setsockopt(listener.fd, SOL_SOCKET, SO_REUSEADDR, &flag, 4); }
    //     // errno_assert (rc == 0);
    // #endif

    //  Bind the socket to the network interface and port.
    // #if defined ZMQ_HAVE_VXWORKS
    //     rc = Bind (_s, (sockaddr *) address.addr (), address.addrlen ());
    // #else
    unsafe {
        rc = bind(listener.fd, listener.address.addr(), listener.address.addrlen());
    }
    // #endif
    // #ifdef ZMQ_HAVE_WINDOWS
    if (rc == SOCKET_ERROR) {
        // unsafe {
        //   // errno = wsa_error_to_errno(WSAGetLastError());
        // }
        // goto error;
    }
    // #else
    if (rc != 0) {
        // goto
        // error;
    }
    // #endif

    //  Listen for incoming connections.
    unsafe {
        rc = listen(listen.fd, options.backlog);
    }
    // #ifdef ZMQ_HAVE_WINDOWS
    if (rc == SOCKET_ERROR) {
        // unsafe {
        //   // errno = wsa_error_to_errno(WSAGetLastError());
        // }
        // goto error;
    }
    // #else
    if (rc != 0) {
        // goto
        // error;
    }
    // #endif

    Ok(())

    // TODO
    // error:
    //     let err: i32 = errno;
    //     close ();
    //   // errno = err;
    //     return -1;
}

pub fn tcp_set_local_address(listener: &mut ZmqListener, addr: &mut str) -> anyhow::Result<()> {
    if options.use_fd != -1 {
        //  in this case, the addr_ passed is not used and ignored, since the
        //  socket was already created by the application
        listener.fd = options.use_fd;
    } else {
        tcp_create_socket(listener, addr)?;
    }

    listener.endpoint = get_socket_name(listener.fd, ZmqSocketEnd::SocketEndLocal)?;

    listener.socket
        .event_listening(&make_unconnected_bind_endpoint_pair(&listener.endpoint), listener.fd);
    Ok(())
}

pub fn tcp_accept(listener: &mut ZmqListener) -> anyhow::Result<ZmqFileDesc> {
    //  The situation where connection cannot be accepted due to insufficient
    //  resources is considered valid and treated by ignoring the connection.
    //  Accept one connection and deal with different failure modes.
    // zmq_assert (_s != retired_fd);

    // struct sockaddr_storage ss;
    let mut ss = SOCKADDR_STORAGE::default();
    // memset (&ss, 0, mem::size_of::<ss>());
    // #if defined ZMQ_HAVE_HPUX || defined ZMQ_HAVE_VXWORKS
    //     let mut ss_len = mem::size_of::<ss>();
    // #else
    let mut ss_len: c_int = mem::size_of_val(&ss) as c_int;
    // #endif
    // #if defined ZMQ_HAVE_SOCK_CLOEXEC && defined HAVE_ACCEPT4
    // let mut sock: ZmqFileDesc = ::accept4(listener.fd, (&ss), &ss_len, SOCK_CLOEXEC);
    // #else
    let sock = unsafe { accept(listener.fd, (&mut ss) as *mut sockaddr, &mut ss_len) };
    // #endif

    unsafe {
        if (sock == RETIRED_FD as usize) {
            // #if defined ZMQ_HAVE_WINDOWS
            let last_error: i32 = WSAGetLastError() as i32;
            // wsa_assert(last_error == WSAEWOULDBLOCK || last_error == WSAECONNRESET || last_error == WSAEMFILE || last_error == WSAENOBUFS); # elif
            // defined
            // ZMQ_HAVE_ANDROID
            // errno_assert (errno == EAGAIN || errno == EWOULDBLOCK || errno == EINTR || errno == ECONNABORTED || errno == EPROTO || errno == ENOBUFS || errno == ENOMEM || errno == EMFILE || errno == ENFILE || errno == EINVAL);
            // #else
            // errno_assert (errno == EAGAIN || errno == EWOULDBLOCK || errno == EINTR
            // || errno == ECONNABORTED || errno == EPROTO || errno == ENOBUFS || errno == ENOMEM || errno == EMFILE || errno == ENFILE);
            // #endif
            return Ok(RETIRED_FD as ZmqFileDesc);
        }
    }

    make_socket_noninheritable(sock);

    if (!options.tcp_accept_filters.empty()) {
        let mut matched = false;
        // for (ZmqOptions::tcp_accept_filters_t::size_type
        //        i = 0,
        //        size = options.tcp_accept_filters.size ();
        //      i != size; += 1i)
        for i in 0..options.tcp_accept_filters.len() {
            if (options.tcp_accept_filters[i].match_address((&ss), ss_len)) {
                matched = true;
                break;
            }
        }
        unsafe {
            if (!matched) {
                // #ifdef ZMQ_HAVE_WINDOWS
                let rc: i32 = closesocket(sock);
                // wsa_assert (rc != SOCKET_ERROR);
                // #else
                //             int rc = ::close (sock);
                // errno_assert (rc == 0);
                // #endif
                return Ok(RETIRED_FD as ZmqFileDesc);
            }
        }
    }

    unsafe {
        if (set_nosigpipe(sock)) {
            // #ifdef ZMQ_HAVE_WINDOWS
            let rc: i32 = closesocket(sock);
            // wsa_assert(rc != SOCKET_ERROR);
            // #else
            // let rc = ::close(sock);
            // errno_assert (rc == 0);
            // #endif
            return Ok(RETIRED_FD as ZmqFileDesc);
        }
    }

    // Set the IP Type-Of-Service priority for this client socket
    if (options.tos != 0) {
        set_ip_type_of_service(sock, options.tos);
    }

    // Set the protocol-defined priority for this client socket
    if (options.priority != 0) {
        set_socket_priority(sock, options.priority);
    }

    return Ok(sock as ZmqFileDesc);
}
