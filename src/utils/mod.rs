use std::ffi::c_char;

use anyhow::bail;
use libc::c_void;
#[cfg(not(target_os = "windows"))]
use libc::sa_family_t;
#[cfg(target_os = "windows")]
use windows::Win32::Networking::WinSock::sa_family_t;

use crate::defines::{ZmqFd, ZmqIpMreq, ZmqSockAddr, ZmqSockAddrIn, ZmqSockAddrIn6, ZmqSockaddrStorage};
use crate::select::fd_set;

mod decoder_allocators;
mod random;
mod zmq_utils;

pub fn copy_bytes(
    src: &[u8],
    src_offset: usize,
    src_count: usize,
    dst: &mut [u8],
    dst_offset: usize,
) -> anyhow::Result<()> {
    if dst.len() - dst_offset < src_count {
        bail!("insufficient length in source to copy destination")
    }

    for i in 0..src_count {
        dst[dst_offset + i] = src[src_offset + i];
    }

    Ok(())
}

pub unsafe fn copy_void(
    src: *const c_void,
    src_offset: usize,
    src_count: usize,
    dst: *mut c_void,
    dst_offset: usize,
    dst_len: usize,
) -> anyhow::Result<()> {
    if dst_len - dst_offset < src_count {
        bail!("insufficient length in source to copy destination")
    }

    for i in 0..src_count {
        *dst.add(dst_offset + i) = *src.add(src_offset + i);
    }

    Ok(())
}

pub fn get_errno() -> i32 {
    std::io::Error::last_os_error().raw_os_error().unwrap()
}

pub const decoder: [u8; 96] = [
    0xFF, 0x44, 0xFF, 0x54, 0x53, 0x52, 0x48, 0xFF, 0x4B, 0x4C, 0x46, 0x41, 0xFF, 0x3F, 0x3E, 0x45,
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x40, 0xFF, 0x49, 0x42, 0x4A, 0x47,
    0x51, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0x30, 0x31, 0x32,
    0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x4D, 0xFF, 0x4E, 0x43, 0xFF,
    0xFF, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
    0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22, 0x23, 0x4F, 0xFF, 0x50, 0xFF, 0xFF,
];

pub const encoder: &'static str = "0123456789 \
abcdefghij \
klmnopqrst \
uvwxyzABCD \
EFGHIJKLMN \
OPQRSTUVWX \
YZ.-:+=^!/ \
*?&<>()[]{ \
}@%$#";

pub unsafe fn zmq_z85_decode(dest_: *mut u8, string_: *const c_char) -> *mut u8 {
    let mut byte_nbr = 0u8;
    let mut char_nbr = 0u8;
    let mut value = 0u32;
    let mut src_len = libc::strlen(string_);

    if src_len < 5 || src_len % 5 != 0 {
        return std::ptr::null_mut();
    }

    while string_[char_nbr] != 0 {
        if u32::MAX / 85 < value {
            return std::ptr::null_mut();
        }

        value *= 85;

        let index = string_[char_nbr] - 32;
        char_nbr += 1;
        if index >= decoder.len() {
            return std::ptr::null_mut();
        }
        let summand = decoder[index];
        if summand == 0xff || summand > u32::MAX - value {
            return std::ptr::null_mut();
        }
        value += summand;
        if char_nbr % 5 == 0 {
            let mut divisor = 256 * 256 * 256;
            while (divisor) {
                dest_[byte_nbr] = value / divisor % 256;
                byte_nbr += 1;
                divisor /= 256;
            }
            value = 0;
        }
    }

    if char_nbr % 5 != 0 {
        return std::ptr::null_mut();
    }
    return dest_;
}

pub fn put_u32(ptr: &mut [u8], value: u32) {
    unsafe {
        *ptr = (value >> 24) as u8;
        *ptr.add(1) = (value >> 16) as u8;
        *ptr.add(2) = (value >> 8) as u8;
        *ptr.add(3) = value as u8;
    }
}

pub unsafe fn get_u32(ptr: &[u8]) -> u32 {
    let u32_bytes: [u8; 4] = [*ptr, *ptr.add(1), *ptr.add(2), *ptr.add(3)];
    u32::from_le_bytes(u32_bytes)
}

pub fn put_u64(ptr: &mut [u8], value: u64) {
    unsafe {
        *ptr = (value >> 56) as u8;
        *ptr.add(1) = (value >> 48) as u8;
        *ptr.add(2) = (value >> 40) as u8;
        *ptr.add(3) = (value >> 32) as u8;
        *ptr.add(4) = (value >> 24) as u8;
        *ptr.add(5) = (value >> 16) as u8;
        *ptr.add(6) = (value >> 8) as u8;
        *ptr.add(7) = value as u8;
    }
}

pub fn get_u64(ptr: &[u8]) -> u64 {
    let u64_bytes: [u8; 8] = [
        ptr[0], ptr[1], ptr[2], ptr[3], ptr[4], ptr[5], ptr[6], ptr[7],
    ];
    u64::from_le_bytes(u64_bytes)
}

pub fn is_retired_fd(x: ZmqFd) -> bool {
    x == -1
}

pub fn FD_ISSET(fd: ZmqFd, fds: &fd_set) -> bool {
    let mut i = 0;
    while i < fds.len() {
        if fds[i] == fd {
            return true;
        }
        i += 1;
    }
    false
}

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
        sin6_addr: in6_addr { s6_addr: [0; 16] },
        sin6_scope_id: 0,
    };
    out.sin6_addr.s6_addr = sockaddr.sa_data[2..18].try_into().unwrap();
    out
}

pub fn zmq_sockaddr_to_sockaddr(sockaddr: &ZmqSockAddr) -> libc::sockaddr {
    let mut out = libc::sockaddr {
        sa_family: sockaddr.sa_family,
        sa_data: [0; 14],
    };
    out.sa_data[0..14].copy_from_slice(&sockaddr.sa_data[0..14]);
    out
}

pub fn zmq_sockaddr_storage_to_sockaddr(sockaddr_storage: ZmqSockaddrStorage) -> libc::sockaddr {
    let mut out = libc::sockaddr {
        sa_family: sockaddr_storage.ss_family as sa_family_t,
        sa_data: [0; 14],
    };
    out.sa_data[0..14].copy_from_slice(&sockaddr_storage.sa_data[0..14]);
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

pub fn zmq_ip_mreq_to_bytes(ipmreq: &ZmqIpMreq) -> [u8;8] {
    let mut out = [0; 8];
    out[0..4].copy_from_slice(&ipmreq.imr_multiaddr.to_le_bytes());
    out[4..8].copy_from_slice(&ipmreq.imr_interface.to_le_bytes());
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
    out.sa_data[0..14].copy_from_slice(&sockaddr.sa_data[0..14]);
    out
}
