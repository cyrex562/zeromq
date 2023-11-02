use crate::address::SocketEnd::SocketEndRemote;
use crate::defines::{ZmqFd, RETIRED_FD};
use libc::setsockopt;
use std::ffi::{c_char, c_int};
use std::mem;
use windows::Win32::Networking::WinSock::{
    getnameinfo, IPPROTO_IP, IPPROTO_IPV6, IPV6_V6ONLY, IP_TOS, NI_MAXHOST, NI_NUMERICHOST,
    SOCKADDR, SOCK_STREAM, SOL_SOCKET,
};

pub mod ip_resolver;
pub mod ip_resolver_options;

pub fn open_socket(domain_: i32, type_: i32, protocol_: i32) -> ZmqFd {
    let mut rc = 0;

    #[cfg(target_os = "windows")]
    let s: fd_t = unsafe { WSASocket(domain_, type_, protocol_, null_mut(), 0, 0) };
    #[cfg(not(target_os = "windows"))]
    let s: fd_t = unsafe { socket(domain_, type_, protocol_) };

    if s == RETIRED_FD {
        return RETIRED_FD;
    }

    make_socket_noninheritable(s);
    rc = set_nosigpipe(s);
    s
}

pub unsafe fn unblock_socket(s_: fd_t) {
    #[cfg(target_os = "windows")]
    {
        let mut nonblock: c_ulong = 1;
        let rc = ioctlsocket(s_, FIONBIO, &nonblock);
    }
    #[cfg(not(target_os = "windows"))]
    {
        let mut flags: i32 = fcntl(s_, F_GETFL, 0);
        if (flags == -1) {
            flags = 0;
        }
        let rc = fcntl(s_, F_SETFL, flags | O_NONBLOCK);
    }
}

pub unsafe fn enable_ipv4_mapping(s_: fd_t) {
    let mut on: c_int = 1;
    let rc = setsockopt(
        s_ as SOCKET,
        IPPROTO_IPV6 as c_int,
        IPV6_V6ONLY,
        &on as *const c_char,
        mem::size_of_val(&on) as c_int,
    );
}

pub fn get_peer_ip_address(sockfd_: fd_t, ip_addr: &str) -> i32 {
    // XXX: This probably needs re-writing
    let mut ss = SOCKADDR::default();
    let addrlen = get_socket_address(sockfd_, SocketEndRemote, &mut ss);
    let mut host: [u8; NI_MAXHOST as usize] = [0; NI_MAXHOST];
    let mut rc = getnameinfo(
        &ss as *const SOCKADDR,
        addrlen,
        Some(&mut host),
        None,
        NI_NUMERICHOST as i32,
    );
    let ip_addr = std::str::from_utf8(&host).unwrap();
    return ss.sa_family as i32;
}

pub unsafe fn set_ip_type_of_service(s_: fd_t, iptos_: i32) {
    let mut tos: c_int = iptos_;
    let rc = setsockopt(
        s_ as SOCKET,
        IPPROTO_IP as c_int,
        IP_TOS,
        &tos as *const c_char,
        mem::size_of_val(&tos) as c_int,
    );
    // TODO: on non-windows platforms with IP v6 enabled, we should also set the IPV6_TCLASS option
    // rc = setsockopt (s_, IPPROTO_IPV6, IPV6_TCLASS,
    //                      reinterpret_cast<char *> (&iptos_), sizeof (iptos_));
}

pub unsafe fn set_socket_priority(s_: fd_t, priority: i32) {
    let rc = setsockopt(
        s_,
        SOL_SOCKET,
        SO_PRIORITY,
        &priority as *const c_char,
        mem::size_of_val(&priority) as c_int,
    );
}

pub unsafe fn set_nosigpipe(s_: fd_t) -> i32 {
    // todo: only do when SO_NOSIGPIPE is defined
    let mut on: c_int = 1;
    let rc = setsockopt(
        s_ as SOCKET,
        SOL_SOCKET,
        SO_NOSIGPIPE,
        &on as *const c_char,
        mem::size_of_val(&on) as c_int,
    );
    return rc;
}

pub unsafe fn bind_to_device(s_: fd_t, bound_device_: &str) -> i32 {
    let rc = setsockopt(
        s_,
        SOL_SOCKET,
        SO_BINDTODEVICE,
        bound_device_.as_ptr() as *const c_char,
        bound_device_.len() as c_int,
    );
    return rc;
}

pub fn initialize_network() -> Result<(), ZmqError> {
    // define pgm section omitted
    #[cfg(target_os = "windows")]
    {
        let mut wsaData: WSADATA = WSADATA::default();
        let rc = WSAStartup(0x0202u16, &mut wsaData);
        if rc != 0 {
            return false;
        }
    }

    return Ok(());
}

pub fn shutdown_network() {
    #[cfg(target_os = "windows")]
    {
        let rc = WSACleanup();
    }
    // define pgm section omitted
}

#[cfg(target_os = "windows")]
pub unsafe fn tune_socket(socket_: SOCKET) {
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
        let mut sync =
            CreateEventA(Some(&sa), FALSE, TRUE, "Global\\zmq-signaler-port-sync").unwrap();

        if sync == INVALID_HANDLE_VALUE && GetLastError() == ERROR_ACCESS_DENIED {
            let desired_access =
                (SYNCHRONIZE as SYNCHRONIZATION_ACCESS_RIGHTS | EVENT_MODIFY_STATE);
            sync = OpenEventA(desired_access, FALSE, "Global\\zmq-signaler-port-sync").unwrap();
        }
    } else if SIGNALER_PORT != 0 {
        // let mutex_name: [u8; MAX_PATH] = [0; MAX_PATH];
        // let rc = snprintf(mutex_name, MAX_PATH, "Global\\zmq-signaler-port-sync-%u", SIGNALER_PORT);
        let mux_name = format!("Global\\zmq-signaler-port-sync-{}", SIGNALER_PORT);
        let mutex_name = mux_name.as_ptr() as *const c_char;
        let mut sync = CreateMutexA(Some(&sa), FALSE, mutex_name).unwrap();
        if sync == INVALID_HANDLE_VALUE && GetLastError() == ERROR_ACCESS_DENIED {
            let desired_access =
                (SYNCHRONIZE as SYNCHRONIZATION_ACCESS_RIGHTS | EVENT_MODIFY_STATE);
            sync = OpenMutexA(desired_access as u32, FALSE, mutex_name).unwrap();
        }
        *w_ = INVALID_SOCKET as fd_t;
        *r_ = INVALID_SOCKET as fd_t;
        let mut listener = SOCKET::default();
        listener = open_socket(AF_INET as i32, SOCK_STREAM as i32, 0) as SOCKET;
        let mut so_reuseaddr: BOOL = 1;
        let mut rc = setsockopt(
            listener,
            SOL_SOCKET,
            SO_REUSEADDR,
            &so_reuseaddr as *const c_char,
            mem::size_of_val(&so_reuseaddr) as c_int,
        );
        tune_socket(listener);

        let mut addr = SOCKADDR_IN::default();
        addr.sin_family = AF_INET as u16;
        addr.sin_addr.s_addr = INADDR_LOOPBACK;
        addr.sin_port = SIGNALER_PORT as u16;

        *w_ = open_socket(AF_INET as i32, SOCK_STREAM as i32, 0);

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
            tune_socket(*w_);
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
            make_socket_noninheritable(*r_);
            return 0;
        }

        if *w_ != INVALID_SOCKET {
            let result = closesocket(*w_);
            *w_ = INVALID_SOCKET;
        }

        return -1;
    }
}

pub unsafe fn make_fdpair(r_: *mut fd_t, w_: *mut fd_t) -> i32 {
    let mut rc = 0;
    let mut pipefd: [fd_t; 2] = [0; 2];
    #[cfg(target_os = "windows")]
    {
        return make_fdpair_tcpip(r_, w_);
    }
    #[cfg(not(target_os = "windows"))]
    {
        let sv: [i32; 2] = [0; 2];
        let rc = socketpair(AF_UNIX, SOCK_STREAM, 0, sv);
        if rc == -1 {
            *w_ = -1;
            *r_ = -1;
            return -1;
        } else {
            make_socket_noninheritable(sv[0]);
            make_socket_noninheritable(sv[1]);
            *w_ = sv[0];
            *r_ = sv[1];
            return 0;
        }
    }
    return -1;
}

pub unsafe fn make_socket_noninheritable(sock_: fd_t) {
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
