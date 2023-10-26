

use std::ptr::null_mut;

use libc::c_void;
use windows::Win32::{Networking::WinSock::{socklen_t, SOCKADDR_STORAGE}, };
use crate::ctx::ZmqContext;
use crate::fd::fd_t;
use crate::tcp_address::ZmqTcpAddress;
use crate::udp_address::UdpAddress;


pub type zmq_socklen_t = socklen_t;

pub enum socket_end_t
{
    socket_end_local,
    socket_end_remote,
}

pub union AddressUnion
{
    //pub dummy: *mut c_void,
    pub tcp_addr: ZmqTcpAddress,
    pub udp_addr: UdpAddress,
    // pub ws_addr: *mut ws_address_t,
    // pub wss_addr: *mut wss_address_t,
    // pub ipc_addr: *mut ipc_address_t,
    // pub tipc_addr: *mut tipc_address_t,
    // pub vmci_addr: *mut vmci_address_t,
}

impl std::fmt::Debug for AddressUnion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

pub struct ZmqAddress<'a>
{
    pub protocol: String,
    pub address: String,
    pub parent: &'a mut ZmqContext<'a>,
    pub resolved: AddressUnion,

}

impl Clone for ZmqAddress {
    fn clone(&self) -> Self {
        Self { protocol: self.protocol.clone(), address: self.address.clone(), parent: self.parent.clone(), resolved: self.resolved.clone() }
    }
}

impl std::fmt::Debug for ZmqAddress
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("address_t").field("protocol", &self.protocol).field("address", &self.address).field("parent", &self.parent).field("resolved", &self.resolved).finish()
    }
}



impl Default for ZmqAddress
{
    fn default() -> Self {
        Self { resolved: AddressUnion{dummy: null_mut()}, ..Default::default() }
    }
}

impl ZmqAddress
{
    pub fn new(protocol_: &mut String, address_: &mut String, parent_: &mut ZmqContext) -> Self
    {
        Self {
            protocol: (*protocol_).clone(),
            address: (*address_).clone(),
            parent: parent_,
            resolved: AddressUnion{dummy: null_mut()},
        }
    }

    pub fn to_string(&mut self) -> String {
        todo!()
    }
}

pub fn get_socket_name<T>(fd_: fd_t, socket_end_: socket_end_t) -> String
{
    let mut ss = SOCKADDR_STORAGE::default();
    let mut sl = get_socket_address(fd_,socket_end_, &mut ss);
    //  const T addr (reinterpret_cast<struct sockaddr *> (&ss), sl);
    // TODO figure out how to create instance of T from sl and ss

    let mut address_string = String::new();

    // TODO
    // addr.to_string (address_string);

    return address_string
    todo!()
}
