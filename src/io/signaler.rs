use std::ffi::c_void;
use std::mem::size_of_val;

use libc::{c_int, close, EAGAIN, EINTR, getpid, read, write};
#[cfg(target_os = "windows")]
use windows::Win32::Networking::WinSock::{recv, select, send, SEND_RECV_FLAGS, SOCKET_ERROR, TIMEVAL};
#[cfg(target_os = "windows")]
use windows::Win32::Networking::WinSock::{WSAEWOULDBLOCK, WSAGetLastError};
#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::Sleep;

use crate::defines::{ZmqFd, ZmqPid};
use crate::defines::err::ZmqError;
use crate::defines::err::ZmqError::PollerError;
use crate::defines::time::ZmqTimeval;
use crate::ip::{make_fdpair, unblock_socket};
use crate::platform::platform_select;
use crate::poll::select::{fd_set, FD_SET, FD_ZERO};
use crate::utils::get_errno;

#[derive(Default, Debug, Clone)]
pub struct ZmqSignaler {
    pub _w: ZmqFd,
    pub _r: ZmqFd,
    #[cfg(feature = "fork")]
    pub pid: ZmqPid,
}

impl ZmqSignaler {
    pub fn new() -> Result<Self, ZmqError> {
        let mut out = Self {
            _r: 0,
            _w: 0,
            pid: 0,
        };
        let mut rc = unsafe { make_fdpair(&mut out._r, &mut out._w) };
        if rc.is_ok() {
            unsafe { unblock_socket(out._r)?; }
            unsafe { unblock_socket(out._w)?; }
        }

        Ok(out)
    }

    pub fn get_fd(&mut self) -> ZmqFd {
        self._r
    }

    pub fn send(&mut self) -> Result<(), ZmqError> {
        #[cfg(feature = "fork")]
        unsafe {
            if self.pid != getpid() {
                return Ok(());
            }
        }

        #[cfg(feature = "eventfd")]
        {
            let mut inc = 1u64;
            let inc_bytes = inc.to_le_bytes();
            let mut sz = unsafe {
                write(
                    self._w as c_int,
                    inc_bytes.as_ptr() as *const c_void,
                    size_of_val(&inc),
                )
            };
        }
        #[cfg(target_os = "windows")]
        {
            let mut dummy = 0u8;
            let mut nbytes = 0i32;
            let mut flags = SEND_RECV_FLAGS::default();
            flags.0 = 0;
            loop {
                // nbytes = send(self._w, &[dummy], flags);
                // if nbytes != SOCKET_ERROR {
                //     break;
                // }
                platform_send(self._w, &[dummy], flags.0)?;
            }
        }

        // #else
        // unsigned char dummy = 0;
        #[cfg(not(target_os = "windows"))]{
            let mut dummy = 0u8;
            let dummy_bytes = dummy.to_le_bytes();
            loop {
                let mut nbytes = unsafe {
                    libc::send(
                        self._w,
                        dummy_bytes.as_mut_ptr() as *const c_void,
                        0,
                        0,
                    )
                };
                if nbytes == -1 && get_errno() == EINTR {
                    continue;
                }
// #if defined(HAVE_FORK)
                #[cfg(feature = "fork")]{
                    if self.pid != unsafe { getpid() } {
                        //printf("Child process %d signaler_t::send returning without sending #2\n", getpid());
                        // get_errno() = EINTR;
                        break;
                    }
                }
// #endif
//                 zmq_assert(nbytes == sizeof dummy);
                break;
            }
        }
// #endif
        Ok(())
    }

    pub fn wait(&mut self, timeout_: i32) -> Result<(), ZmqError> {
        // #ifdef HAVE_FORK
        #[cfg(feature = "fork")]
        unsafe {
            if self.pid != getpid() {
                // we have forked and the file descriptor is closed. Emulate an interrupt
                // response.
                //printf("Child process %d signaler_t::wait returning simulating interrupt #1\n", getpid());
                // errno = EINTR;
                return Err(PollerError("wait failed"));
            }
        }
        // #endif

        // #ifdef ZMQ_POLL_BASED_ON_POLL
        #[cfg(feature = "poll")]
        {
            // struct pollfd pfd;
            let mut pfd = ZmqPollFd {
                fd: 0,
                events: 0,
                revents: 0,
            };
            pfd.fd = self._r;
            pfd.events = POLLIN;
            let mut rc = 0;
            // TODO
            //let rc = poll(&pfd, 1, timeout_);
            if rc < 0 {
                // errno_assert (errno == EINTR);
                return -1;
            }
            if rc == 0 {
                // errno = EAGAIN;
                return -1;
            }
            // #ifdef HAVE_FORK
            #[cfg(feature = "fork")]
            {
                if self.pid != getpid() {
                    // we have forked and the file descriptor is closed. Emulate an interrupt
                    // response.
                    //printf("Child process %d signaler_t::wait returning simulating interrupt #2\n", getpid());
                    // errno = EINTR;
                    return -1;
                }
            }
            // #endif
            //     zmq_assert (rc == 1);
            //     zmq_assert (pfd.revents & POLLIN);
            return 0;
        }
        // #elif defined ZMQ_POLL_BASED_ON_SELECT
        #[cfg(feature = "select")] {
            // optimized_fd_set_t fds (1);
            let mut fds = fd_set { fd_count: 0, fd_array: [0 as ZmqFd; 64] };
            FD_ZERO(&mut fds);
            FD_SET(self._r, &mut fds);
            // struct timeval timeout;
            let mut timeout = ZmqTimeval::default();
            if timeout_ >= 0 {
                timeout.tv_sec = timeout_ / 1000;
                timeout.tv_usec = timeout_ % 1000 * 1000;
            }
            // #ifdef ZMQ_HAVE_WINDOWS
            let mut rc = 0;

            platform_select(0, Some(&mut fds), None, None, if timeout_ >= 0 { Some(&mut timeout) } else { None })?;

            // #[cfg(target_os = "windows")]
            // {
            //     rc = select(0, fds.get(), None, None, if timeout_ >= 0 { Some(&timeout as *const TIMEVAL) } else { None });
            //     // wsa_assert (rc != SOCKET_ERROR);
            // }
            // // #else
            // #[cfg(not(target_os = "windows"))]
            // {
            //     unsafe { rc = select(_r + 1, fds.get(), NULL, NULL, timeout_ >= 0? & timeout: NULL); }
            //     if unlikely(rc < 0) {
            //         errno_assert(errno == EINTR);
            //         return Err(PollerError("select failed"));
            //     }
            // }
            // // #endif
            // if rc == 0 {
            //     // errno = EAGAIN;
            //     return Err(PollerError("EAGAIN"));
            // }
            // // zmq_assert (rc == 1);
            return Ok(());
        }
        // #else
        // #Error
        // #endif
    }

    pub fn recv(&mut self) {
        //  Attempt to read a signal.
        // #if defined ZMQ_HAVE_EVENTFD
        let mut dummy = 0u64;
        #[cfg(feature = "eventfd")]
        {
            // uint64_t dummy;

            let sz = unsafe { read(self._r as c_int, dummy.to_le_bytes().as_mut_ptr() as *mut c_void, 8) };
            // errno_assert (sz == sizeof (dummy));

            //  If we accidentally grabbed the next signal(s) along with the current
            //  one, return it back to the eventfd object.
            if (dummy > 1) {
                let mut inc = dummy - 1;
                let sz2 = unsafe { write(self._w as c_int, inc.to_le_bytes().as_ptr() as *const c_void, 8) };
                // errno_assert (sz2 == sizeof (inc));
                return;
            }

            // zmq_assert (dummy == 1);
        }
        // #else
        #[cfg(not(feature = "eventfd"))]
        {
            let mut dummy = 0u8;
        }
        // #if defined ZMQ_HAVE_WINDOWS
        #[cfg(target_os = "windows")]
        {
            let nbytes = recv(self._r, dummy.to_le_bytes().as_mut_slice(), SEND_RECV_FLAGS::default());
            // wsa_assert (nbytes != SOCKET_ERROR);
        }
        // #elif defined ZMQ_HAVE_VXWORKS
        //     ssize_t nbytes = ::recv (_r, (char *) &dummy, sizeof (dummy), 0);
        //     errno_assert (nbytes >= 0);
        // #else
        #[cfg(not(target_os = "windows"))]
        {
            let mut dummy_ptr = dummy.to_le_bytes().as_mut_ptr();
            let nbytes = unsafe { libc::recv(self._r, dummy_ptr as *mut c_void, 8, 0) };
            // errno_assert (nbytes >= 0);
        }
        // #endif
        // zmq_assert (nbytes == sizeof (dummy));
        // zmq_assert (dummy == 0);
        // #endif
    }

    pub fn recv_failable(&mut self) -> Result<(), ZmqError> {
        //  Attempt to read a signal.
        // #if defined ZMQ_HAVE_EVENTFD
        let mut dummy = 0u64;
        #[cfg(feature = "eventfd")]
        {
            let sz = unsafe { read(self._r as c_int, dummy.to_le_bytes().as_mut_ptr() as *mut c_void, 8) };
            if sz == -1 {
                // errno_assert (errno == EAGAIN);
                return Err(PollerError("EAGAIN"));
            }
            // errno_assert (sz == sizeof (dummy));

            //  If we accidentally grabbed the next signal(s) along with the current
            //  one, return it back to the eventfd object.
            if dummy > 1 {
                let inc = dummy - 1;
                let sz2 = unsafe {
                    write(
                        self._w as c_int,
                        inc.to_le_bytes().as_ptr() as *const c_void,
                        8,
                    )
                };
                // errno_assert (sz2 == sizeof (inc));
                return Ok(());
            }

            // zmq_assert (dummy == 1);
        }
        #[cfg(not(feature = "eventfd"))]
        {
            let mut dummy = 0u8;
        }
        // #else
        //     unsigned char dummy;
        // #if defined ZMQ_HAVE_WINDOWS
        #[cfg(target_os = "windows")]
        {
            let nbytes = recv(self._r, dummy.to_le_bytes().as_mut_slice(), SEND_RECV_FLAGS::default());
            if nbytes == SOCKET_ERROR {
                let last_error = unsafe { WSAGetLastError() };
                if last_error == WSAEWOULDBLOCK {
                    // errno = EAGAIN;
                    return -1;
                }
                // wsa_assert (last_error == WSAEWOULDBLOCK);
            }
        }
        // #elif defined ZMQ_HAVE_VXWORKS
        //     ssize_t nbytes = ::recv (_r, (char *) &dummy, sizeof (dummy), 0);
        //     if (nbytes == -1) {
        //         if (errno == EAGAIN || errno == EWOULDBLOCK || errno == EINTR) {
        //             errno = EAGAIN;
        //             return -1;
        //         }
        //         errno_assert (errno == EAGAIN || errno == EWOULDBLOCK
        //                       || errno == EINTR);
        //     }
        // #else
        #[cfg(not(target_os = "windows"))]
        {
            let mut dummy_ptr = dummy.to_le_bytes().as_mut_ptr();
            let nbytes = unsafe {
                libc::recv(
                    self._r,
                    dummy_ptr as *mut c_void,
                    size_of_val(&dummy),
                    0,
                )
            };
            if nbytes == -1 {
                // if (errno == EAGAIN || errno == EWOULDBLOCK || errno == EINTR) {
                //     errno = EAGAIN;
                //     return -1;
                // }
                // errno_assert (errno == EAGAIN || errno == EWOULDBLOCK
                //               || errno == EINTR);
            }
        }
        // #endif
        // zmq_assert (nbytes == sizeof (dummy));
        // zmq_assert (dummy == 0);
        // #endif
        return Ok(());
    }

    pub fn valid(&mut self) -> bool {
        self._w != -1
    }

    pub fn forked(&mut self) {
        #[cfg(feature = "fork")]
        {
            // self.pid = getpid();
            unsafe { close(self._r as c_int) };
            unsafe { close(self._w as c_int) };
            make_fdpair(&mut self._r, &mut self._w);
        }
    }
}

pub fn sleep_ms(millis: u32) -> Result<(), ZmqError> {
    let mut rc = 0;
    if millis == 0 {
        return Err(PollerError("sleep_ms failed"));
    }

    #[cfg(target_os = "windows")]
    {
        unsafe { Sleep(millis as u32) };
    }
    #[cfg(not(target_os = "windows"))]
    {
        rc = unsafe { libc::usleep(millis * 1000) };
    }
    return Ok(());
}

pub fn close_wait_ms(fd_: i32, max_ms_: u32) -> Result<(), ZmqError> {
    let mut ms_so_far = 0i32;
    let min_step_ms = 1u32;
    let max_step_ms = 100u32;
    let step_ms = u32::min(u32::max(min_step_ms, max_ms_ / 10), max_step_ms);

    let mut rc = 0i32;
    while ms_so_far < max_ms_ as i32 && rc == -1 && get_errno() == EAGAIN {
        if rc == -1 && get_errno() == EAGAIN {
            std::thread::sleep(std::time::Duration::from_millis(step_ms.into()));
            ms_so_far += step_ms as i32;
        }
        rc = unsafe { libc::close(fd_) };
    }

    Ok(())
}
