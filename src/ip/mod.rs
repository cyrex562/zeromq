use std::mem;

use libc::c_int;
use windows::Win32::Foundation::BOOL;
use windows::Win32::Networking::WinSock::SOCKET;

use crate::address::get_socket_address;
use crate::address::SocketEnd::SocketEndRemote;
use crate::defines::{IP_TOS, IPPROTO_IP, IPPROTO_IPV6, IPPROTO_TCP, IPV6_V6ONLY, MSG_NOSIGNAL, NI_NUMERICHOST, SO_BINDTODEVICE, SO_PRIORITY, SOL_SOCKET, ZmqFd, ZmqSockAddr};
use crate::defines::err::ZmqError;
use crate::defines::tcp::TCP_NODELAY;
use crate::net::platform_socket::{
    platform_getnameinfo, platform_init_network, platform_make_fdpair, platform_open_socket,
    platform_setsockopt, platform_shutdown_network, platform_unblock_socket,
};
use crate::tcp::tcp_tune_loopback_fast_path;
use crate::utils::sock_utils::zmq_sockaddrstorage_to_zmq_sockaddr;

pub mod ip_resolver;
pub mod ip_resolver_options;

pub fn open_socket(domain_: i32, type_: i32, protocol_: i32) -> Result<ZmqFd, ZmqError> {
    platform_open_socket(domain_, type_, protocol_)
}

pub fn unblock_socket(fd: ZmqFd) -> Result<(), ZmqError> {
    platform_unblock_socket(fd)
}

pub fn enable_ipv4_mapping(fd: ZmqFd) -> Result<(), ZmqError> {
    let mut on: c_int = 1;
    let on_bytes = on.to_le_bytes();
    platform_setsockopt(
        fd,
        IPPROTO_IPV6,
        IPV6_V6ONLY,
        &on_bytes,
        on_bytes.len() as i32,
    )
}

pub fn get_peer_ip_address(fd: ZmqFd) -> Result<String, ZmqError> {
    // let sa = ZmqSockAddr::default();
    let ss = get_socket_address(fd, SocketEndRemote)?;
    let sa = zmq_sockaddrstorage_to_zmq_sockaddr(&ss);
    let sl = mem::size_of::<ZmqSockAddr>() as u32;
    let result = platform_getnameinfo(&sa, sl as usize, true, false, NI_NUMERICHOST)?;

    Ok(result.nodebuffer.unwrap())
}

pub fn set_ip_type_of_service(fd: ZmqFd, iptos_: i32) -> Result<(), ZmqError> {
    let mut tos: c_int = iptos_;

    platform_setsockopt(
        fd,
        IPPROTO_IP,
        IP_TOS,
        &tos.to_le_bytes(),
        mem::size_of_val(&tos) as i32,
    )?;

    // TODO: on non-windows platforms with IP v6 enabled, we should also set the IPV6_TCLASS option
    // rc = setsockopt (s_, IPPROTO_IPV6, IPV6_TCLASS,
    //                      reinterpret_cast<char *> (&iptos_), sizeof (iptos_));
    Ok(())
}

pub fn set_socket_priority(fd: ZmqFd, priority: i32) -> Result<(), ZmqError> {
    platform_setsockopt(
        fd,
        SOL_SOCKET as i32,
        SO_PRIORITY,
        &priority.to_le_bytes(),
        mem::size_of_val(&priority) as i32,
    )?;
    Ok(())
}

pub fn set_nosigpipe(fd: ZmqFd) -> Result<(), ZmqError> {
    // todo: only do when SO_NOSIGPIPE is defined
    let mut on: c_int = 1;

    platform_setsockopt(
        fd,
        SOL_SOCKET as i32,
        MSG_NOSIGNAL,
        &on.to_le_bytes(),
        mem::size_of_val(&on) as i32,
    )?;
    Ok(())
}

pub fn bind_to_device(fd: ZmqFd, bound_device_: &str) -> Result<(), ZmqError> {
    platform_setsockopt(
        fd,
        SOL_SOCKET as i32,
        SO_BINDTODEVICE,
        bound_device_.as_bytes(),
        bound_device_.len() as c_int,
    )
}

pub fn initialize_network() -> Result<(), ZmqError> {
    platform_init_network()
}

pub fn shutdown_network() -> Result<(), ZmqError> {
    platform_shutdown_network()
    // define pgm section omitted
}

pub unsafe fn make_fdpair(r_: &mut ZmqFd, w_: &mut ZmqFd) -> Result<(), ZmqError> {
    platform_make_fdpair(r_, w_)
}

// static void tune_socket (const SOCKET socket_)
// {
//     BOOL tcp_nodelay = 1;
//     const int rc =
//       setsockopt (socket_, IPPROTO_TCP, TCP_NODELAY,
//                   reinterpret_cast<char *> (&tcp_nodelay), sizeof tcp_nodelay);
//     wsa_assert (rc != SOCKET_ERROR);
//
//     zmq::tcp_tune_loopback_fast_path (socket_);
// }
#[cfg(target_os = "windows")]
pub fn tune_socket(socket: SOCKET) -> Result<(), ZmqError> {
    let tcp_nodelay = BOOL { 0: 1 };
    platform_setsockopt(socket.0, IPPROTO_TCP, TCP_NODELAY, &tcp_nodelay.0.to_le_bytes(), 4)?;

    tcp_tune_loopback_fast_path(socket.0)?;

    Ok(())
}
