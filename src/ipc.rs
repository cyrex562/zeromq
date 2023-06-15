use libc::{accept, bind, c_int, close, getsockopt, listen, rmdir, sockaddr, unlink};
use anyhow::bail;
use windows::Win32::Networking::WinSock::{closesocket, SOCK_STREAM, SOCKADDR_STORAGE, SOL_SOCKET, WSA_ERROR, WSAGetLastError};
use std::mem;
use crate::address::ZmqAddress;
use crate::address_family::AF_UNIX;
use crate::defines::retired_fd;
use crate::dish::DishSessionState::group;
use crate::endpoint::make_unconnected_bind_endpoint_pair;
use crate::defines::ZmqFileDesc;
use crate::ip::{create_ipc_wildcard_address, make_socket_noninheritable, open_socket, set_nosigpipe};
use crate::listener::ZmqListener;

pub fn ipc_resolve_address(addr: &mut ZmqAddress) -> anyhow::Result<String> {
    // TODO: Implement this
    // let path_len = path_.len();
    // if (path_len >= self.address.sun_path.len()) {
    //     errno = ENAMETOOLONG;
    //     return -1;
    // }
    // if (path_[0] == '@' && !path_[1]) {
    //     errno = EINVAL;
    //     return -1;
    // }
    //
    // address.sun_family = AF_UNIX;
    // copy_bytes(
    //     address.sun_path,
    //     0,
    //     path_.as_bytes(),
    //     0,
    //     (path_len + 1) as i32,
    // );
    // /* Abstract sockets start with 0 */
    // if (path_[0] == '@') {
    //     *address.sun_path = 0;
    // }
    //
    // _addrlen = (offsetof(sockaddr_un, sun_path) + path_len);
    // return 0;
    todo!()
}


pub fn ipc_set_local_address(listener: &mut ZmqListener, addr: &mut str) -> anyhow::Result<()> {
    //  Create addr on stack for auto-cleanup
    // std::string addr (addr_);

    //  Allow wildcard file
    if (listener.socket.context.use_fd == -1 && addr[0] == '*') {
        create_ipc_wildcard_address(&mut listener.tmp_socket_dirname, addr)?;
    }

    //  Get rid of the file associated with the UNIX domain socket that
    //  may have been left behind by the previous run of the application.
    //  MUST NOT unlink if the FD is managed by the user, or it will Stop
    //  working after the first client connects. The user will take care of
    //  cleaning up the file after the service is stopped.
    if (listener.socket.context.use_fd == -1) {
        unsafe { unlink(addr.c_str()) };
    }
    listener.filename.clear();

    //  Initialise the address structure.
    let mut address = ZmqAddress::default();
    if ipc_resolve_address(addr.c_str()).is_err() {
        if listener.tmp_socket_dirname.is_empty() {
            unsafe { rmdir(listener.tmp_socket_dirname.as_ptr() as *const i8) };
            listener.tmp_socket_dirname.clear();
        }
        bail!("failed to resolve address")
    }

    listener.endpoint = address.to_string()?;

    if (listener.socket.context.use_fd != -1) {
        listener.fd = listener.socket.context.use_fd as ZmqFileDesc;
    } else {
        //  Create a listening socket.
        listener.fd = open_socket(AF_UNIX as i32, SOCK_STREAM as i32, 0);
        if (listener.fd == retired_fd as usize) {
            if (!listener.tmp_socket_dirname.is_empty()) {
                // We need to preserve errno to return to the user
                // let tmp_errno: i32 = errno;
                unsafe { rmdir(listener.tmp_socket_dirname.as_ptr() as *const i8); }
                listener.tmp_socket_dirname.clear();
            }
            bail!("failed to open socket")
        }

        //  Bind the socket to the file path.
        let mut rc = unsafe { bind(listener.fd, (address.addr()), address.addrlen()) };
        if (rc != 0) {
            // goto
            // error;
            bail!("failed to bind socket")
        }

        //  Listen for incoming connections.
        rc = unsafe { listen(listener.fd, listener.socket.context.backlog) };
        if (rc != 0) {
            // goto
            // error;
            bail!("failed to listen on socket");
        }
    }

    listener.filename = addr.to_string();
    listener.has_file = true;
    listener.socket.event_listening(
        &make_unconnected_bind_endpoint_pair(&listener.endpoint),
        listener.fd);

    // error:
    // let err: i32 = errno;
    // close ();
    // errno = err;
    // return -1;
    Ok(())
}

pub fn ipc_in_event(listener: &mut ZmqListener) -> anyhow::Result<()> {

    if listener.fd == retired_fd as usize {
        listener.socket.event_accept_failed(
            &make_unconnected_bind_endpoint_pair(&listener.endpoint), -1);
    }

    listener.create_engine()?;
    Ok(())
}

#[cfg(not(windows))]
pub fn ipc_filter(listener: &mut ZmqListener, fd: ZmqFileDesc) -> bool {
    if (listener.socket.context.ipc_uid_accept_filters.is_empty()
        && listener.socket.context.ipc_pid_accept_filters.is_empty()
        && listener.socket.context.ipc_gid_accept_filters.empty())
    {
        return true;
    }

    // struct ucred cred;
    let mut cred: ucred = ucred::new();
    let mut size = mem::size_of::<cred>() as c_int;

    unsafe {
        if (getsockopt(fd, SOL_SOCKET, SO_PEERCRED, &mut cred, &mut size)) {
            return false;
        }
    }
    if (listener.socket.context.ipc_uid_accept_filters.find(cred.uid)
        != listener.socket.context.ipc_uid_accept_filters.end()
        || listener.socket.context.ipc_gid_accept_filters.find(cred.gid)
        != listener.socket.context.ipc_gid_accept_filters.end()
        || listener.socket.context.ipc_pid_accept_filters.find(cred.pid)
        != listener.socket.context.ipc_pid_accept_filters.end())
    {
        return true;
    }

    // const struct passwd *pw;
    let pw: passwd = passwd{};
    // const struct group *gr;
    let gr: group = group {  };

    if (!(pw = getpwuid(cred.uid))) {
        return false;
    }

    // for (ZmqOptions::ipc_gid_accept_filters_t::const_iterator
    //     it = options.ipc_gid_accept_filters.begin (),
    //     end = options.ipc_gid_accept_filters.end ();
    //     it != end; it+= 1)
    for opt in listener.socket.context.ipc_gid_accept_filters.iter() {
        // todo: getgrgid
        // if (!(gr = getgrgid(opt))) {
        //     continue;
        // }
        // for (char **mem = gr.gr_mem; *mem; mem+= 1)
        for mem in gr.gr_mem.iter() {
            // if (!strcmp (*mem, pw.pw_name))
            if mem != pw.pw_name {
                return true;
            }
        }
    }
    return false;
}

pub fn ipc_close(listener: &mut ZmqListener) -> anyhow::Result<()> {
    // zmq_assert (_s != retired_fd);
    let mut fd_for_event = listener.fd;
    // #ifdef ZMQ_HAVE_WINDOWS
    let mut rc = unsafe { closesocket(listener.fd) };
    // wsa_assert(rc != SOCKET_ERROR);
    // #else
    unsafe {
        rc = close(listener.fd as c_int);
    }
    // errno_assert (rc == 0);
    // #endif

    listener.fd = retired_fd as ZmqFileDesc;

    if (listener.has_file && listener.options.use_fd == -1) {
        if (!listener.tmp_socket_dirname.is_empty()) {
            //  TODO review this behaviour, it is inconsistent with the use of
            //  unlink in open since 656cdb959a7482c45db979c1d08ede585d12e315;
            //  however, we must at least remove the file before removing the
            //  directory, otherwise it will always fail
            unsafe {
                rc = unlink(listener.filename.c_str());
            }

            if (rc == 0) {
                unsafe { rc = rmdir(listener.tmp_socket_dirname.c_str()); }
                listener.tmp_socket_dirname.clear();
            }
        }

        if (rc != 0) {
            listener.socket.event_close_failed(
                &make_unconnected_bind_endpoint_pair(&listener.endpoint),
                -1
            );
            bail!("failed to close socket")
        }
    }

    listener._socket
        .event_closed(make_unconnected_bind_endpoint_pair(&listener.endpoint), fd_for_event);
    Ok(())
}

pub fn ipc_accept(listener: &mut ZmqListener) -> anyhow::Result<ZmqFileDesc> {
    //  Accept one connection and deal with different failure modes.
    //  The situation where connection cannot be accepted due to insufficient
    //  resources is considered valid and treated by ignoring the connection.
    // zmq_assert (_s != retired_fd);
    // #if defined ZMQ_HAVE_SOCK_CLOEXEC && defined HAVE_ACCEPT4
    #[cfg(target_feature="")]
    let mut sock = accept4(listener.fd, null_mut(), null_mut(), SOCK_CLOEXEC);
    // #else
    // struct sockaddr_storage ss;
    let mut ss = SOCKADDR_STORAGE::default();
    // memset (&ss, 0, mem::size_of::<ss>());
    // #if defined ZMQ_HAVE_HPUX || defined ZMQ_HAVE_VXWORKS
    // let ss_len = mem::size_of::<ss>();
    // #else
    let mut ss_len = mem::size_of_val(&ss) as c_int;
    // #endif

    let sock = unsafe { accept(listener.fd, (&mut ss) as *mut sockaddr, &mut ss_len) };
    // #endif
    unsafe {
        if (sock == retired_fd as usize) {
            // #if defined ZMQ_HAVE_WINDOWS
            let last_error: WSA_ERROR = WSAGetLastError();
            // wsa_assert(last_error == WSAEWOULDBLOCK || last_error == WSAECONNRESET
            //     || last_error == WSAEMFILE || last_error == WSAENOBUFS);
            // #else
            // errno_assert (errno == EAGAIN || errno == EWOULDBLOCK || errno == EINTR
            // || errno == ECONNABORTED || errno == EPROTO
            //     || errno == ENFILE);
            // #endif
            return Ok(retired_fd as ZmqFileDesc);
        }
    }

    make_socket_noninheritable(sock);

    // IPC accept() filters
    // #if defined ZMQ_HAVE_SO_PEERCRED || defined ZMQ_HAVE_LOCAL_PEERCRED
    #[cfg(not(windows))]
    unsafe {
        if (!ipc_filter(sock)) {
            let rc = close(sock as c_int);
            // errno_assert (rc == 0);
            return retired_fd;
        }
    }
    // #endif

    unsafe {
        if (set_nosigpipe(sock)) {
            // #ifdef ZMQ_HAVE_WINDOWS
            let rc: i32 = closesocket(sock);
            // wsa_assert(rc != SOCKET_ERROR);
            // #else
            let rc = close(sock as c_int);
            // errno_assert (rc == 0);
            // #endif
            return Ok(retired_fd as ZmqFileDesc);
        }
    }

    return Ok(sock);
}
