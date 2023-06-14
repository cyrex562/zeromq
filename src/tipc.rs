use std::mem;
use anyhow::bail;
use bincode::options;
use libc::windows::{accept, bind, c_int, listen};
use windows::Windows::Win32::Networking::WinSock::{SOCK_STREAM, SOCKADDR_STORAGE};
use zeromq::address_family::AF_TIPC;
use zeromq::defines::{retired_fd, TIPC_ADDR_ID};
use zeromq::endpoint::make_unconnected_bind_endpoint_pair;
use zeromq::fd::ZmqFileDesc;
use zeromq::ip::open_socket;
use zeromq::listener::ZmqListener;

pub fn tipc_accept(listener: &mut ZmqListener) -> anyhow::Result<ZmqFileDesc> {
    //  Accept one connection and deal with different failure modes.
    //  The situation where connection cannot be accepted due to insufficient
    //  resources is considered valid and treated by ignoring the connection.
    let mut ss = SOCKADDR_STORAGE::default();
    // socklen_t ss_len = mem::size_of::<ss>();
    let mut ss_len = mem::size_of_val(&ss);

    // zmq_assert (_s != retired_fd);
    // #ifdef ZMQ_HAVE_VXWORKS
    //      let mut sock: ZmqFileDesc = accept (_s, &ss, &ss_len);
    // #else
    let mut sock: ZmqFileDesc = unsafe { accept(listener.fd, &mut ss as *mut windows::sockaddr, &mut ss_len as *mut c_int) };
    // #endif
    if (sock == -1) {
        // errno_assert (errno == EAGAIN || errno == EWOULDBLOCK
        //               || errno == ENOBUFS || errno == EINTR
        //               || errno == ECONNABORTED || errno == EPROTO
        //               || errno == EMFILE || errno == ENFILE);
        return Ok(retired_fd as ZmqFileDesc);
    }
    /*FIXME Accept filters?*/
    return Ok(sock);
}

pub fn tipc_in_event(listener: &mut ZmqListener) -> anyhow::Result<()>
{
    let mut fd: ZmqFileDesc = tipc_accept(listener)?;

    //  If connection was reset by the peer in the meantime, just ignore it.
    //  TODO: Handle specific errors like ENFILE/EMFILE etc.
    if (fd == retired_fd as usize) {
        listener.socket
            .event_accept_failed(&make_unconnected_bind_endpoint_pair(&listener.endpoint), -1);
        bail!("failed to bind")
    }

    //  Create the engine object for this connection.
    listener.create_engine();

    Ok(())
}

pub fn tipc_set_local_address(listener: &mut ZmqListener, addr: &mut String) -> anyhow::Result<()> {
    // Convert str to address struct
    *addr = listener.address.resolve()?;

    // Cannot Bind non-random Port Identity
    let a = (listener.address.addr());
    if !listener.address.is_random() && a.addrtype == TIPC_ADDR_ID {
        bail!("Cannot bind to non-random Port Identity")
    }

    //  Create a listening socket.
    listener.fd = open_socket(AF_TIPC as i32, SOCK_STREAM as i32, 0);
    if (listener.fd == retired_fd as usize) {
        bail!("failed to open socket")
    }

    // If random Port Identity, update address object to reflect the assigned address
    if (listener.address.is_random()) {
        // TODO
        // let mut ss = SOCKADDR_STORAGE::default();
        // let sl = get_socket_address(listener.fd, ZmqSocketEnd::SocketEndLocal, &mut ss).unwrap();
        // if (sl == 0) {
        //     // goto
        //     // error;
        // }
        //
        // self.address = ZmqTipcAddress::new2((&ss), sl as socklen_t);
    }

    listener.endpoint = listener.address.to_string()?;

    //  Bind the socket to tipc name
    if (listener.address.is_service()) {
        // #ifdef ZMQ_HAVE_VXWORKS
        //         rc = Bind (_s,  address.addr (), address.addrlen ());
        // #else
        let mut bind_result = 0;
        unsafe {
            bind_result = bind(listener.fd, listener.address.addr(), listener.address.addrlen());
        }
        // #endif
        if (bind_result != 0) {
            // goto
            // error;
            bail!("failed to bind")
        }
    }

    //  Listen for incoming connections.
    let mut listen_result = 0;
    unsafe {
        listen_result = listen(listener.fd, options.backlog);
    }
    if (listen_result != 0) {
        // goto
        // error;
    }

    listener._socket
        .event_listening(make_unconnected_bind_endpoint_pair(&listener.endpoint), listener.fd);

    // error:
    //     int err = errno;
    //     close ();
    //     errno = err;
    //     return -1;
    Ok(())
}
