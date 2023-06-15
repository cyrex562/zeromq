use anyhow::bail;
use libc::{bind, listen, setsockopt};
use windows::Win32::Networking::WinSock::{SO_REUSEADDR, SOCKET_ERROR, SOL_SOCKET};
use crate::address::{get_socket_name, ZmqAddress, ZmqSocketEnd};
use crate::defines::retired_fd;
use crate::endpoint::{EndpointUriPair, make_unconnected_bind_endpoint_pair};
use crate::endpoint::EndpointType::endpoint_type_bind;
use crate::engine_interface::ZmqEngineInterface;
use crate::defines::ZmqFileDesc;
use crate::ip::make_socket_noninheritable;
use crate::listener::ZmqListener;
use crate::session_base::ZmqSessionBase;
use crate::tcp::{tcp_open_socket, tune_tcp_maxrt, tune_tcp_socket};
use crate::ws_engine::ZmqWsEngine;
use crate::wss_engine::WssEngine;

pub fn ws_in_event(listener: &mut ZmqListener) -> anyhow::Result<()>{
    let mut fd = listener.accept()?;

    //  If connection was reset by the peer in the meantime, just ignore it.
    //  TODO: Handle specific errors like ENFILE/EMFILE etc.
    if (fd == retired_fd as usize) {
        listener.socket
            .event_accept_failed(&make_unconnected_bind_endpoint_pair(&listener.endpoint), -1);
        bail!("accept failed")
    }

    let mut rc = tune_tcp_socket(&mut fd);
    rc = rc | tune_tcp_maxrt(&mut fd, listener.socket.context.tcp_maxrt);
    if (rc != 0) {
        listener.socket
            .event_accept_failed(&make_unconnected_bind_endpoint_pair(&listener.endpoint), -1);
        bail!("accept failed")
    }

    //  Create the engine object for this connection.
    ws_create_engine(listener, fd);
}

pub fn ws_create_socket(listener: &mut ZmqListener, addr_: &mut str) -> anyhow::Result<()> {
    // TcpAddress address;
    let mut address: ZmqAddress = ZmqAddress::default();
    listener.fd = tcp_open_socket(addr_, listener.socket.context, true, true, &mut address);
    if (listener.fd == retired_fd as usize) {
        bail!("failed to open socket")
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
    // wsa_assert (rc != SOCKET_ERROR);
    // #elif defined ZMQ_HAVE_VXWORKS
    //     rc =
    //       setsockopt (listener.fd, SOL_SOCKET, SO_REUSEADDR,  &flag, mem::size_of::<int>());
    // errno_assert (rc == 0);
    // #else
    unsafe {
        rc = setsockopt(listener.fd, SOL_SOCKET, SO_REUSEADDR, &flag, 4);
    }
    // errno_assert (rc == 0);
    // #endif

    //  Bind the socket to the network interface and port.
    // #if defined ZMQ_HAVE_VXWORKS
    // unsafe {
    //     rc = bind(listener.fd, address.addr(), address.addrlen());
    // }
    // #else
    let mut bind_result = 0;
    unsafe {
        bind_result = bind(listener.fd, address.addr(), address.addrlen());
    }
    // #endif
    // #ifdef ZMQ_HAVE_WINDOWS
    if (bind_result == SOCKET_ERROR) {
        unsafe {
            // errno = wsa_error_to_errno(WSAGetLastError());
        }
        // goto error;
        bail!("failed to bind socket");
    }
    // #else
    // if (rc != 0) {
    //     // goto
    //     // error;
    // }
    // #endif

    //  Listen for incoming connections.
    let mut listen_result = 0;
    unsafe {
        listen_result = listen(listener.fd, listener.socket.context.backlog);
    }
    // #ifdef ZMQ_HAVE_WINDOWS
    if (listen_result == SOCKET_ERROR) {
        unsafe {
            // errno = wsa_error_to_errno(WSAGetLastError());
        }
        // goto error;
        bail!("failed to listen on socket");
    }
    // #else
    // if (rc != 0) {
    //     // goto
    //     // error;
    // }
    // #endif

    // return 0;
    Ok(())

    // error:
    //     let err: i32 = errno;
    //     close ();
    //     errno = err;
    //     return -1;
}

pub fn ws_set_local_address(listener: &mut ZmqListener, addr: &mut String) -> i32 {
    if (listener.socket.context.use_fd != -1) {
        //  in this case, the addr_ passed is not used and ignored, since the
        //  socket was already created by the application
        listener.fd = listener.socket.context.use_fd as ZmqFileDesc;
    } else {
        *addr = listener.address.resolve()?;

        //  remove the path, otherwise resolving the port will fail with wildcard
        // TODO:
        // const char *delim = strrchr (addr_, '/');
        // host_address: String;
        // if (delim) {
        //     host_address = std::string (addr_, delim - addr_);
        // } else {
        //     host_address = addr_;
        // }

        if (ws_create_socket(listener, listener.host_address) == -1) {
            return -1;
        }
    }

    listener.endpoint = get_socket_name(listener.fd, ZmqSocketEnd::SocketEndLocal)?;

    listener.socket
        .event_listening(&make_unconnected_bind_endpoint_pair(&listener.endpoint), listener.fd);
    return 0;
}

pub fn ws_accept(listener: &mut ZmqListener) -> anyhow::Result<ZmqFileDesc> {
    //  The situation where connection cannot be accepted due to insufficient
    //  resources is considered valid and treated by ignoring the connection.
    //  Accept one connection and deal with different failure modes.
    // zmq_assert (listener.fd != retired_fd);

    //     struct sockaddr_storage ss;
    //     memset (&ss, 0, mem::size_of::<ss>());
    // // #if defined ZMQ_HAVE_HPUX || defined ZMQ_HAVE_VXWORKS
    //     int ss_len = mem::size_of::<ss>();
    // // #else
    //     socklen_t ss_len = mem::size_of::<ss>();
    // // #endif
    // // #if defined ZMQ_HAVE_SOCK_CLOEXEC && defined HAVE_ACCEPT4
    //      let mut sock: ZmqFileDesc = ::accept4 (listener.fd, (&ss),
    //                            &ss_len, SOCK_CLOEXEC);
    // // #else
    //     const ZmqFileDesc sock =
    //       ::accept (listener.fd, (&ss), &ss_len);
    // // #endif
    //
    //     if (sock == retired_fd) {
    // // #if defined ZMQ_HAVE_WINDOWS
    //         let last_error: i32 = WSAGetLastError ();
    //         wsa_assert (last_error == WSAEWOULDBLOCK || last_error == WSAECONNRESET
    //                     || last_error == WSAEMFILE || last_error == WSAENOBUFS);
    // #elif defined ZMQ_HAVE_ANDROID
    //         // errno_assert (errno == EAGAIN || errno == EWOULDBLOCK || errno == EINTR
    //                       || errno == ECONNABORTED || errno == EPROTO
    //                       || errno == ENOBUFS || errno == ENOMEM || errno == EMFILE
    //                       || errno == ENFILE || errno == EINVAL);
    // // #else
    //         // errno_assert (errno == EAGAIN || errno == EWOULDBLOCK || errno == EINTR
    //                       || errno == ECONNABORTED || errno == EPROTO
    //                       || errno == ENOBUFS || errno == ENOMEM || errno == EMFILE
    //                       || errno == ENFILE);
    // // #endif
    //         return retired_fd;
    //     }
    //
    //     make_socket_noninheritable (sock);
    //
    //     if (set_nosigpipe (sock)) {
    // // #ifdef ZMQ_HAVE_WINDOWS
    //         let rc: i32 = closesocket (sock);
    //         wsa_assert (rc != SOCKET_ERROR);
    // // #else
    //         int rc = ::close (sock);
    //         // errno_assert (rc == 0);
    // // #endif
    //         return retired_fd;
    //     }
    //
    //     // Set the IP Type-Of-Service priority for this client socket
    //     if (options.tos != 0)
    //         set_ip_type_of_service (sock, options.tos);
    //
    //     // Set the protocol-defined priority for this client socket
    //     if (options.priority != 0)
    //         set_socket_priority (sock, options.priority);
    //
    //     return sock;
    todo!()
}

pub fn ws_create_engine(listener: &mut ZmqListener, fd: ZmqFileDesc) -> anyhow::Result<()> {
    let mut endpoint_pair = EndpointUriPair::new(
        &get_socket_name(fd, ZmqSocketEnd::SocketEndLocal).unwrap(),
        &get_socket_name(fd, ZmqSocketEnd::SocketEndRemote).unwrap(),
        endpoint_type_bind,
    );

    // ZmqEngineInterface *engine = null_mut();
    let mut engine: ZmqEngineInterface;
    if (listener.wss) {
        // #ifdef ZMQ_HAVE_WSS
        engine = WssEngine::new(
            fd,
            listener.socket.context,
            &mut endpoint_pair,
            &mut listener.address,
            false,
            Some(listener.tls_cred.as_mut_slice()),
            "",
        );
        // #else
        // zmq_assert (false);
        // #endif
    } else {
        engine = ZmqWsEngine::new(fd, listener.socket.context, &mut endpoint_pair, &mut listener.address, false);
    }

    // alloc_assert (engine);

    //  Choose I/O thread to run connecter in. Given that we are already
    //  running in an I/O thread, there must be at least one available.
    let io_thread = listener.choose_io_thread(listener.socket.context.affinity).unwrap();
    // zmq_assert (io_thread);

    //  Create and launch a session object.
    let mut session =
        ZmqSessionBase::create(io_thread, false, listener.socket,  None);
    // errno_assert (session);
    session.inc_seqnum();
    // TODO
    // launch_child(&session);
    // TODO
    // send_attach(session, engine, false);

    listener.socket.event_accepted(&endpoint_pair, fd);
    Ok(())
}
