use std::ffi::c_void;
use std::os::windows::raw::HANDLE;
use std::ptr::null_mut;
use windows::Win32::Security::{PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES, SECURITY_DESCRIPTOR};
use windows::Win32::System::Threading::CreateEventW;
use crate::fd::fd_t;

pub unsafe fn make_fdpair(r_: *mut fd_t, w_: *mut fd_t) -> i32 {
    #[cfg(target_os = "windows")]
    {
        make_fdpair_tcpip(r_, w_)
    }
    #[cfg (not(target_os="windows"))]{
        let mut sv: [fd_t; 2] = [0; 2];
        let rc = unsafe { libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM, 0, sv.as_mut_ptr()) };
        if rc == -1 {
            return -1;
        }
        *r_ = sv[0];
        *w_ = sv[1];
        return 0;
    }
}

pub unsafe fn make_fdpair_tcpip(r_: *mut fd_t, w_: *mut fd_t) -> i32 {
    let mut sd = SECURITY_DESCRIPTOR::default();
    let mut sa = SECURITY_ATTRIBUTES::default();

    let mut psd: PSECURITY_DESCRIPTOR = PSECURITY_DESCRIPTOR::default();
    psd.0 = (&mut sd as *mut SECURITY_DESCRIPTOR) as *mut c_void;

    let rc = windows::Win32::Security::InitializeSecurityDescriptor(psd, 1);
    windows::Win32::Security::SetSecurityDescriptorDacl(
        psd,
        true,
        None,
        false);
    sa.lpSecurityDescriptor = psd.0;
    sa.nLength = std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32;

    let mut sync: HANDLE = null_mut();
    let event_signaler_port: i32 = 5905;
    let lpname = PCWSTR::
    if signaler_port == event_signaler_port {
        sync = CreateEventW(&mut sa, FALSE, TRUE, "Global\\zmq-signaler-port-sync").unwrap();
    }

    return -1;
}