#[cfg(target_os = "windows")]
use windows::Win32::Networking::WinSock::{ADDRESS_FAMILY, sa_family_t, SOCKADDR};

use crate::defines::{AF_INET, ZmqIpMreq, ZmqIpv6Mreq, ZmqSaFamily, ZmqSockAddr, ZmqSockAddrIn, ZmqSockAddrIn6, ZmqSockaddrStorage};

pub fn sockaddr_to_sockaddrin(sockaddr: &ZmqSockAddr) -> ZmqSockAddrIn {
    let mut out = ZmqSockAddrIn {
        sin_family: sockaddr.sa_family,
        sin_port: 0,
        sin_addr: 0,
        sin_zero: [0; 8],
    };
    out.sin_addr = u32::from_le_bytes(sockaddr.sa_data[2..6].try_into().unwrap());
    out
}

pub fn sockaddr_to_sockaddrin6(sockaddr: &ZmqSockAddr) -> ZmqSockAddrIn6 {
    let mut out = ZmqSockAddrIn6 {
        sin6_family: sockaddr.sa_family,
        sin6_port: 0,
        sin6_flowinfo: 0,
        sin6_addr: [0; 16],
        sin6_scope_id: 0,
    };
    // out.sin6_addr.s6_addr = sockaddr.sa_data[2..18].try_into().unwrap();
    out.sin6_addr.clone_from_slice(&sockaddr.sa_data[2..18]);
    out
}

pub fn zmq_sockaddr_to_sockaddr(sockaddr: &ZmqSockAddr) -> libc::sockaddr {
    let mut out = libc::sockaddr {
        sa_family: sockaddr.sa_family,
        sa_data: [0; 14],
    };
    for i in 0..out.sa_data.len() {
        out.sa_data[i] = sockaddr.sa_data[i] as libc::c_char;
    }
    out
}

pub fn zmq_sockaddr_storage_to_sockaddr(sockaddr_storage: &ZmqSockaddrStorage) -> libc::sockaddr {
    let mut out = libc::sockaddr {
        sa_family: sockaddr_storage.ss_family as libc::c_ushort,
        sa_data: [0; 14],
    };
    // out.sa_data[0..14].copy_from_slice(&sockaddr_storage.sa_data[0..14] as &[u8]);
    for i in 0..14 {
        out.sa_data[i] = sockaddr_storage.sa_data[i] as libc::c_char;
    }
    out
}

pub fn zmq_sockaddr_storage_to_zmq_sockaddr(ss: &ZmqSockaddrStorage) -> ZmqSockAddr {
    let mut out = ZmqSockAddr {
        sa_family: ss.ss_family as u16,
        sa_data: [0; 14],
    };
    out.sa_data[0..14].copy_from_slice(&ss.sa_data[0..14]);
    out
}

pub fn zmq_sockaddr_to_sockaddrin(sockaddr: &ZmqSockAddr) -> ZmqSockAddrIn {
    let mut out = ZmqSockAddrIn {
        sin_family: sockaddr.sa_family,
        sin_port: 0,
        sin_addr: 0,
        sin_zero: [0; 8],
    };
    out.sin_addr = u32::from_le_bytes(sockaddr.sa_data[2..6].try_into().unwrap());
    out
}

pub fn zmq_sockaddrin_to_sockaddr(sockaddrin: &ZmqSockAddrIn) -> ZmqSockAddr {
    let mut out = ZmqSockAddr {
        sa_family: sockaddrin.sin_family,
        sa_data: [0; 14],
    };
    out.sa_data[0..2].copy_from_slice(&sockaddrin.sin_family.to_le_bytes());
    out.sa_data[2..6].copy_from_slice(&sockaddrin.sin_addr.to_le_bytes());
    out
}

pub fn zmq_ip_mreq_to_bytes(ipmreq: &ZmqIpMreq) -> [u8; 8] {
    let mut out = [0; 8];
    out[0..4].copy_from_slice(&ipmreq.imr_multiaddr.s_addr.to_le_bytes());
    out[4..8].copy_from_slice(&ipmreq.imr_interface.s_addr.to_le_bytes());
    out
}

pub fn zmq_ipv6_mreq_to_bytes(ipv6mreq: &ZmqIpv6Mreq) -> [u8; 20] {
    let mut out = [0; 20];
    out[0..16].copy_from_slice(&ipv6mreq.ipv6mr_multiaddr.s6_addr);
    out[16..20].copy_from_slice(&ipv6mreq.ipv6mr_interface.to_le_bytes());
    out
}

pub fn sockaddr_to_zmq_sockaddr(sockaddr: &libc::sockaddr) -> ZmqSockAddr {
    let mut out = ZmqSockAddr {
        sa_family: sockaddr.sa_family,
        sa_data: [0; 14],
    };
    // out.sa_data[0..14].copy_from_slice(&sockaddr.sa_data[0..14]);
    for i in 0..out.sa_data.len() {
        out.sa_data[i] = sockaddr.sa_data[i] as u8;
    }
    out
}

#[cfg(target_os = "windows")]
pub fn wsa_sockaddr_to_zmq_sockaddr(sockaddr: &SOCKADDR) -> ZmqSockAddr {
    let mut out = ZmqSockAddr {
        sa_family: sockaddr.sa_family.0,
        sa_data: [0; 14],
    };
    out.sa_data[0..14].copy_from_slice(&sockaddr.sa_data[0..14]);
    out
}

pub fn zmq_sockaddr_to_zmq_sockaddrstorage(sockaddr: &ZmqSockAddr) -> ZmqSockaddrStorage {
    // let mut out = ZmqSockaddrStorage {
    //     ss_family: sockaddr.sa_family as ZmqSaFamily,
    //     sa_data: [0; 14],
    // };
    let mut out = ZmqSockaddrStorage::default();
    out.ss_family = sockaddr.sa_family as ZmqSaFamily;
    out.sa_data[0..14].copy_from_slice(&sockaddr.sa_data[0..14]);
    out
}

pub fn zmq_sockaddrstorage_to_zmq_sockaddr(sockaddr_storage: &ZmqSockaddrStorage) -> ZmqSockAddr {
    let mut out = ZmqSockAddr {
        sa_family: sockaddr_storage.ss_family as u16,
        sa_data: [0; 14],
    };
    out.sa_data[0..14].copy_from_slice(&sockaddr_storage.sa_data[0..14]);
    out
}

pub fn zmq_sockaddr_to_string(sockaddr: &ZmqSockAddr) -> String {
    let mut out = String::new();
    if sockaddr.sa_family == AF_INET as u16 {
        out = format!(
            "{}.{}.{}.{}",
            sockaddr.sa_data[2], sockaddr.sa_data[3], sockaddr.sa_data[4], sockaddr.sa_data[5]
        );
    } else {
        out = format!(
            "{:x}{:x}:{:x}{:x}:{:x}{:x}:{:x}{:x}:{:x}{:x}:{:x}{:x}:{:x}{:x}:{:x}{:x}",
            sockaddr.sa_data[2],
            sockaddr.sa_data[3],
            sockaddr.sa_data[4],
            sockaddr.sa_data[5],
            sockaddr.sa_data[6],
            sockaddr.sa_data[7],
            sockaddr.sa_data[8],
            sockaddr.sa_data[9],
            sockaddr.sa_data[10],
            sockaddr.sa_data[11],
            sockaddr.sa_data[12],
            sockaddr.sa_data[13],
            sockaddr.sa_data[14],
            sockaddr.sa_data[15],
            sockaddr.sa_data[16],
            sockaddr.sa_data[17],
        );
    }

    out
}

#[cfg(target_os = "windows")]
pub fn zmq_sockaddr_to_wsa_sockaddr(sockaddr: &ZmqSockAddr) -> SOCKADDR {
    let mut out = SOCKADDR {
        sa_family: ADDRESS_FAMILY { 0: sockaddr.sa_family },
        sa_data: [0; 14],
    };
    out.sa_data[0..14].copy_from_slice(&sockaddr.sa_data[0..14]);
    out
}

pub fn sockaddr_data_to_bytes(sa: &ZmqSockAddr) -> Vec<u8> {
    let mut out = Vec::new();
    for i in 0..sa.sa_data.len() {
        out.push(sa.sa_data[i]);
    }
    out
}
