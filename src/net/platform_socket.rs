use std::ptr::null_mut;
use crate::defines::{ZmqSockAddr, NI_MAXHOST, ZmqFd, ZmqPollFd, RETIRED_FD, SOCK_STREAM};
use crate::err::ZmqError;
use crate::err::ZmqError::PlatformError;
use crate::ip::set_nosigpipe;
#[cfg(not(target_os = "windows"))]
use libc::{getnameinfo, setsockopt, F_GETFL, F_SETFL, O_NONBLOCK};

use libc::{timeval};
#[cfg(target_os = "windows")]
use windows::Win32::Networking::WinSock::{setsockopt, getsockname, SOCKADDR, getpeername, SOCKET, ioctlsocket, FIONBIO, WSASocketA, WSADATA, WSAStartup};
#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::OpenEventA;

use crate::poll::select::fd_set;
use crate::utils::sock_utils::wsa_sockaddr_to_zmq_sockaddr;

pub fn platform_setsockopt(
    fd: ZmqFd,
    level: i32,
    optname: i32,
    optval: &[u8],
    optlen: i32,
) -> Result<(), ZmqError> {
    let mut rc = 0;
    #[cfg(target_os = "windows")]
    {
        unsafe {
            rc = setsockopt(fd, level, optname, Some(optval));
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        unsafe {
            rc = setsockopt(
                fd,
                level,
                optname,
                optval.as_ptr() as *const libc::c_void,
                optlen as socklen_t,
            );
        }
    }

    return if rc == 0 {
        Ok(())
    } else {
        Err(PlatformError(&format!("setsockopt failed {}", rc)))
    };
}

pub struct GetNameInfoResult {
    pub nodebuffer: Option<String>,
    pub servicebuffer: Option<String>,
}

pub fn platform_getnameinfo(
    sockaddr: &ZmqSockAddr,
    sockaddr_len: usize,
    get_nodename: bool,
    get_servicename: bool,
    flags: i32,
) -> Result<GetNameInfoResult, ZmqError> {
    let mut nodebuffer_cc: *mut libc::c_char = null_mut();
    let mut servicebuffer_cc: *mut libc::c_char = null_mut();
    let mut nodebuffer_cc_len = 0;
    let mut servicebuffer_cc_len = 0;
    let mut rc = 0;

    if get_nodename {
        unsafe {
            nodebuffer_cc = libc::malloc(NI_MAXHOST as libc::size_t) as *mut libc::c_char;
            nodebuffer_cc_len = NI_MAXHOST;
        }
    }
    if get_servicename {
        unsafe {
            servicebuffer_cc = libc::malloc(NI_MAXHOST as libc::size_t) as *mut libc::c_char;
            servicebuffer_cc_len = NI_MAXHOST;
        }
    }

    #[cfg(target_os = "windows")]
    {}

    #[cfg(not(target_os = "windows"))]
    unsafe {
        rc = getnameinfo(
            &zmq_sockaddr_to_sockaddr(sockaddr),
            sockaddr_len as socklen_t,
            nodebuffer_cc,
            nodebuffer_cc_len as socklen_t,
            servicebuffer_cc,
            servicebuffer_cc_len as socklen_t,
            flags,
        );
    }


    return if rc == 0 {
        let mut result = GetNameInfoResult {
            nodebuffer: None,
            servicebuffer: None,
        };

        if get_nodename {
            unsafe {
                let nodebuffer_result = String::from_raw_parts(
                    nodebuffer_cc as *mut u8,
                    nodebuffer_cc_len as usize,
                    nodebuffer_cc_len as usize,
                );
                result.nodebuffer = Some(nodebuffer_result);
            }
        }
        if get_servicename {
            unsafe {
                let servicebuffer_result = String::from_raw_parts(
                    servicebuffer_cc as *mut u8,
                    servicebuffer_cc_len as usize,
                    servicebuffer_cc_len as usize,
                );
                result.servicebuffer = Some(servicebuffer_result);
            }
        }

        Ok(result)
    } else {
        Err(PlatformError(&format!("getnameinfo failed {}", rc)))
    };
}

pub fn platform_getsockname(fd: ZmqFd) -> Result<ZmqSockAddr, ZmqError> {
    let mut zsa = ZmqSockAddr::default();
    let mut zsl = std::mem::size_of::<ZmqSockAddr>() as u32;
    let mut rc = 0;
    #[cfg(target_os = "windows")]
    {
        let mut sa = SOCKADDR::default();
        let mut sl = std::mem::size_of::<SOCKADDR>() as i32;
        rc = unsafe { getsockname(fd, &mut sa as *mut SOCKADDR, &mut sl) };
        if rc == 0 {
            zsa = wsa_sockaddr_to_zmq_sockaddr(&sa);
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let mut sa = libc::sockaddr {
            sa_family: 0,
            sa_data: [0; 14],
        };
        let mut sl = std::mem::size_of::<libc::sockaddr>() as u32;
        rc = unsafe { libc::getsockname(fd, &mut sa as *mut libc::sockaddr, &mut sl) };
        if rc == 0 {
            zsa = sockaddr_to_zmq_sockaddr(&sa);
        }
    }

    return if rc == 0 {
        Ok(zsa)
    } else {
        Err(PlatformError(&format!("getsockname failed {}", rc)))
    };
}

pub fn platform_getpeername(fd: ZmqFd) -> Result<ZmqSockAddr, ZmqError> {
    let mut zsa = ZmqSockAddr::default();
    let mut zsl = std::mem::size_of::<ZmqSockAddr>() as u32;
    let mut rc = 0;
    #[cfg(target_os = "windows")]
    {
        let mut sa = SOCKADDR::default();
        let mut sl = std::mem::size_of::<SOCKADDR>() as i32;
        rc = unsafe { getpeername(SOCKET::try_from(fd), &mut sa as *mut SOCKADDR, &mut sl) };
        if rc == 0 {
            zsa = wsa_sockaddr_to_zmq_sockaddr(&sa);
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let mut sa = libc::sockaddr {
            sa_family: 0,
            sa_data: [0; 14],
        };
        let mut sl = std::mem::size_of::<libc::sockaddr>() as u32;
        rc = unsafe { libc::getpeername(fd, &mut sa as *mut libc::sockaddr, &mut sl) };
        if rc == 0 {
            zsa = sockaddr_to_zmq_sockaddr(&sa);
        }
    }

    return if rc == 0 {
        Ok(zsa)
    } else {
        Err(PlatformError(&format!("getsockname failed {}", rc)))
    };
}

pub fn platform_unblock_socket(fd: ZmqFd) -> Result<(), ZmqError> {
    let mut rc = 0;
    #[cfg(target_os = "windows")]
    {
        let mut nonblock: libc::c_ulong = 1;
        unsafe { rc = ioctlsocket(fd, FIONBIO, &mut nonblock); }
    }
    unsafe {
        #[cfg(not(target_os = "windows"))]
        {
            let mut flags: i32 = libc::fcntl(fd, F_GETFL, 0);
            if flags == -1 {
                flags = 0;
            }
            rc = libc::fcntl(fd, F_SETFL, flags | O_NONBLOCK);
        }
    }

    return if rc == 0 {
        Ok(())
    } else {
        Err(PlatformError(&format!("unblock_socket failed {}", rc)))
    };
}

pub fn platform_open_socket(domain: i32, type_: i32, protocol: i32) -> Result<ZmqFd, ZmqError> {
    #[cfg(target_os = "windows")] let s: ZmqFd = unsafe { WSASocketA(domain, type_, protocol, None, 0, 0).0 as ZmqFd};
    #[cfg(not(target_os = "windows"))] let s: ZmqFd = unsafe { libc::socket(domain, type_, protocol) };

    if s == RETIRED_FD {
        return Err(PlatformError("socket failed"));
    }

    unsafe {
        platform_make_socket_noninheritable(s)?;
    }
    unsafe {
        set_nosigpipe(s)?;
    }

    Ok(s)
}

pub fn platform_init_network() -> Result<(), ZmqError> {
    #[cfg(target_os = "windows")]
    {
        let mut wsaData: WSADATA = WSADATA::default();
        let rc = WSAStartup(0x0202u16, &mut wsaData);
        if rc != 0 {
            return Err(PlatformError(&format!("WSAStartup failed {}", rc)));
        }
    }

    return Ok(());
}

pub fn platform_shutdown_network() -> Result<(), ZmqError> {
    let mut rc = 0;
    #[cfg(target_os = "windows")]
    {
        rc = WSACleanup();
    }
    return if rc == 0 {
        Ok(())
    } else {
        Err(PlatformError("WSACleanup failed"))
    };
}

#[cfg(target_os = "windows")]
pub unsafe fn make_fdpair_tcpip(r_: *mut fd_t, w_: *mut fd_t) -> i32 {
    let mut sd = SECURITY_DESCRIPTOR::default();
    let mut sa = SECURITY_ATTRIBUTES::default();
    InitializeSecurityDescriptor(&mut sd, SECURITY_DESCRIPTOR_REVISION);
    SetSecurityDescriptorDacl(&mut sd, TRUE, None, FALSE);

    sa.nLength = size_of_val(&sa) as u32;
    sa.lpSecurityDescriptor = (&mut sd) as *mut c_void;

    let sync: HANDLE = 0;
    let event_signaler_port = 5905;
    if SIGNALER_PORT == event_signaler_port {
        let mut sync = CreateEventA(Some(&sa), FALSE, TRUE, "Global\\zmq-signaler-port-sync").unwrap();

        if sync == INVALID_HANDLE_VALUE && GetLastError() == ERROR_ACCESS_DENIED {
            let desired_access = (SYNCHRONIZE as SYNCHRONIZATION_ACCESS_RIGHTS | EVENT_MODIFY_STATE);
            sync = OpenEventA(desired_access, FALSE, "Global\\zmq-signaler-port-sync").unwrap();
        }
    } else if SIGNALER_PORT != 0 {
        // let mutex_name: [u8; MAX_PATH] = [0; MAX_PATH];
        // let rc = snprintf(mutex_name, MAX_PATH, "Global\\zmq-signaler-port-sync-%u", SIGNALER_PORT);
        let mux_name = format!("Global\\zmq-signaler-port-sync-{}", SIGNALER_PORT);
        let mutex_name = mux_name.as_ptr() as *const c_char;
        let mut sync = CreateMutexA(Some(&sa), FALSE, mutex_name).unwrap();
        if sync == INVALID_HANDLE_VALUE && GetLastError() == ERROR_ACCESS_DENIED {
            let desired_access = (SYNCHRONIZE as SYNCHRONIZATION_ACCESS_RIGHTS | EVENT_MODIFY_STATE);
            sync = OpenMutexA(desired_access as u32, FALSE, mutex_name).unwrap();
        }
        *w_ = INVALID_SOCKET as fd_t;
        *r_ = INVALID_SOCKET as fd_t;
        let mut listener = SOCKET::default();
        listener = ip::open_socket(AF_INET as i32, SOCK_STREAM as i32, 0) as SOCKET;
        let mut so_reuseaddr: BOOL = 1;
        let mut rc = setsockopt(
            listener,
            SOL_SOCKET,
            SO_REUSEADDR,
            &so_reuseaddr as *const c_char,
            mem::size_of_val(&so_reuseaddr) as c_int,
        );
        ip::tune_socket(listener);

        let mut addr = SOCKADDR_IN::default();
        addr.sin_family = AF_INET as u16;
        addr.sin_addr.s_addr = INADDR_LOOPBACK;
        addr.sin_port = SIGNALER_PORT as u16;

        *w_ = ip::open_socket(AF_INET as i32, SOCK_STREAM as i32, 0);

        if sync != INVALID_HANDLE_VALUE {
            let dwrc = WaitForSingleObject(sync, INFINITE);
        }

        rc = bind(listener, &addr, mem::size_of_val(&addr) as i32);

        if rc != SOCKET_ERROR && SIGNALER_PORT == 0 {
            let addrlen = mem::size_of_val(&addr) as i32;
            rc = getsockname(
                listener,
                &mut addr as *mut SOCKADDR_IN as *mut SOCKADDR,
                &mut addrlen,
            );
        }

        if rc != SOCKET_ERROR {
            rc = listen(listener, 1);
        }

        if rc != SOCKET_ERROR {
            rc = connect(
                *w_,
                &addr as *const SOCKADDR_IN as *const SOCKADDR,
                mem::size_of_val(&addr) as i32,
            );
        }

        if rc != SOCKET_ERROR {
            ip::tune_socket(*w_);
            *r_ = accept(listener, null_mut(), null_mut()) as fd_t;
        }

        if *r_ != INVALID_SOCKET {
            let dummy_size = 1024 * 1024;
            let mut dummy = vec![0u8; dummy_size];
            let mut still_to_send = dummy_size;
            let mut still_to_recv = dummy_size;
            while still_to_send || still_to_recv {
                let mut nbytes = 0i32;
                if still_to_send > 0 {
                    nbytes = send(*w_, &dummy[dummy_size - still_to_send..], 0);
                    if nbytes > 0 {
                        still_to_send -= nbytes;
                    }
                }
                nbytes = recv(*r_, &dummy[dummy_size - still_to_recv], 0);
                still_to_recv -= nbytes;
            }
        }

        rc = closesocket(listener);

        if sync != INVALID_HANDLE_VALUE {
            if SIGNALER_PORT == event_signaler_port {
                let result = SetEvent(sync);
            } else {
                let result = ReleaseMutex(sync);
            }

            let result = CloseHandle(sync);
        }

        if *r_ != INVALID_SOCKET {
            platform_make_socket_noninheritable(*r_);
            return 0;
        }

        if *w_ != INVALID_SOCKET {
            let result = closesocket(*w_);
            *w_ = INVALID_SOCKET;
        }

        return -1;
    }
}

#[cfg(target_os = "windows")]
pub unsafe fn platform_tune_socket(socket_: SOCKET) {
    let tcp_nodelay = 1;
    let rc = setsockopt(
        socket_,
        IPPROTO_TCP,
        TCP_NODELAY,
        &tcp_nodelay as *const c_char,
        mem::size_of_val(&tcp_nodelay) as c_int,
    );
    tcp_tune_loopback_fast_path(socket_);
}

pub fn platform_make_fdpair(r_: &mut ZmqFd, w_: &mut ZmqFd) -> Result<(), ZmqError> {
    let mut rc = 0;
    let mut pipefd: [ZmqFd; 2] = [0; 2];
    #[cfg(target_os = "windows")]
    {
        rc = make_fdpair_tcpip(r_, w_);
    }
    #[cfg(not(target_os = "windows"))]
    {
        let mut sv: [i32; 2] = [0; 2];
        unsafe { rc = libc::socketpair(AF_UNIX, SOCK_STREAM, 0, sv.as_mut_ptr()); }
        if rc != 0 {
            *w_ = -1;
            *r_ = -1;
        } else {
            platform_make_socket_noninheritable(sv[0]);
            platform_make_socket_noninheritable(sv[1]);
            *w_ = sv[0];
            *r_ = sv[1];
        }
    }
    return if rc == 0 {
        Ok(())
    } else {
        Err(PlatformError("failed to make fdpair"))
    };
}

pub fn platform_make_socket_noninheritable(sock_: ZmqFd) {
    #[cfg(target_os = "windows")]
    {
        let brc = SetHandleInformation(
            sock_ as HANDLE,
            HANDLE_FLAG_INHERIT as u32,
            0 as HANDLE_FLAGS,
        );
    }
    #[cfg(not(target_os = "windows"))]
    {
        // FD_CLOEXEC code ommitted
    }
}

pub fn platform_poll(poll_fd: &mut [ZmqPollFd], nitems: u32, timeout: u32) -> Result<(), ZmqError> {
    let mut result = 0i32;
    #[cfg(target_os = "windows")]
    {
        // pub unsafe fn WSAPoll(fdarray: *mut WSAPOLLFD, fds: u32, timeout: i32) -> i32
        let fdarray = [poll_fd];
        unsafe {
            result = WSAPoll(fdarray.as_mut_ptr(), nitems, timeout as i32);
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        unsafe {
            result = libc::poll(
                poll_fd.as_mut_ptr(),
                nitems as libc::nfds_t,
                timeout as i32,
            );
        }
    }

    Ok(())
}

pub fn platform_select(
    nfds: i32,
    readfds: Option<&mut fd_set>,
    writefds: Option<&mut fd_set>,
    exceptfds: Option<&mut fd_set>,
    timeout: Option<&mut timeval>,
) -> Result<i32, ZmqError> {
    let mut result = 0i32;
    #[cfg(target_os = "windows")]
    {
        unsafe {
            result = select(
                nfds,
                readfds as *mut fd_set,
                writefds as *mut fd_set,
                exceptfds as *mut fd_set,
                timeout as *mut timeval,
            );
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        unsafe {
            result = libc::select(
                nfds,
                readfds.unwrap() as *mut fd_set,
                writefds.unwrap() as *mut fd_set,
                exceptfds.unwrap() as *mut fd_set,
                timeout.unwrap() as *mut timeval,
            );
        }
    }

    return if result >= 0 {
        Ok(result)
    } else {
        Err(PlatformError(&format!("select failed {}", result)))
    };
}
