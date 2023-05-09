#[cfg(target_os = "linux")]
use crate::libc::{sockaddr, sockaddr_storage};
#[cfg(target_os = "windows")]
use crate::windows::Win32::Networking::WinSock::{SOCKADDR, SOCKADDR_STORAGE};
use windows::Win32::Networking::WinSock::{SOCKADDR, SOCKADDR_STORAGE};

#[cfg(target_os = "linux")]
pub type ZmqSockaddrStorage = sockaddr_storage;
#[cfg(target_os = "windows")]
pub type ZmqSockaddrStorage = SOCKADDR_STORAGE;

#[cfg(target_os = "linux")]
pub type ZmqSockaddr = sockaddr;
#[cfg(target_os = "windows")]
pub type ZmqSockaddr = SOCKADDR;
