use std::ffi::c_void;
use std::mem::size_of_val;
use libc::{c_int, close, EAGAIN, getpid, read, timeval, write};
use windows::Win32::Networking::WinSock::{POLLIN, recv, select, send, SEND_RECV_FLAGS, SOCKET_ERROR, TIMEVAL, WSAEWOULDBLOCK, WSAGetLastError};
use crate::ctx::pid_t;
use crate::fd::fd_t;
use crate::ip::{make_fdpair, unblock_socket};
use crate::utils::get_errno;

pub struct ZmqSignaler {
    pub _w: fd_t,
    pub _r: fd_t,
    #[cfg(feature = "fork")]
    pub pid: pid_t,
}

impl ZmqSignaler {
    pub fn new() -> Self {
        let mut out = Self {
            _r: 0,
            _w: 0,
        };
        let mut rc = make_fdpair(&mut out._r, &mut out._w);
        if rc == 0 {
            unblock_socket(out._r);
            unblock_socket(out._w);
        }
    }

    pub fn get_fd(&mut self) -> fd_t {
        self._r
    }

    pub unsafe fn send(&mut self) {
        #[cfg(feature = "fork")]
        {
            if self.pid != getpid() {
                return;
            }
        }

        #[cfg(feature = "eventfd")]
        {
            let mut inc = 1u64;
            let mut sz = self.write(self._w, &inc, size_of_val(&inc));
        }
        #[cfg(target_os = "windows")]
        {
            let mut dummy = 0u8;
            let mut nbytes = 0i32;
            loop {
                nbytes = send(self._w, &dummy as &[u8], 0 as SEND_RECV_FLAGS);
                if nbytes != SOCKET_ERROR {
                    break;
                }
            }
        }

        // #else
        // unsigned char dummy = 0;
        #[cfg(not(target_os = "windows"))]{
            let mut dummy = 0u8;
            loop {
                let mut nbytes = send(self._w, &dummy, 0);
                if ((nbytes == -1 && errno == EINTR)) {
                    continue;
                }
// #if defined(HAVE_FORK)
                #[cfg(feature = "fork")]{
                    if ((self.pid != getpid())) {
                        //printf("Child process %d signaler_t::send returning without sending #2\n", getpid());
                        errno = EINTR;
                        break;
                    }
                }
// #endif
//                 zmq_assert(nbytes == sizeof dummy);
                break;
            }
        }
// #endif
    }

    pub unsafe fn wait(&mut self, timeout_: i32) -> i32 {
        // #ifdef HAVE_FORK
        #[cfg(feature = "fork")]
        {
            if ((self.pid != getpid())) {
                // we have forked and the file descriptor is closed. Emulate an interrupt
                // response.
                //printf("Child process %d signaler_t::wait returning simulating interrupt #1\n", getpid());
                // errno = EINTR;
                return -1;
            }
        }
        // #endif

        // #ifdef ZMQ_POLL_BASED_ON_POLL
        #[cfg(feature = "poll")]
        {
            // struct pollfd pfd;
            let mut pfd = pollfd {
                fd: 0,
                events: 0,
                revents: 0,
            };
            pfd.fd = self._r;
            pfd.events = POLLIN;
            let rc = poll(&pfd, 1, timeout_);
            if ((rc < 0)) {
                // errno_assert (errno == EINTR);
                return -1;
            }
            if ((rc == 0)) {
                // errno = EAGAIN;
                return -1;
            }
            // #ifdef HAVE_FORK
            #[cfg(feature = "fork")]
            {
                if ((self.pid != getpid())) {
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
            let fds = optimized_fd_set_t::new(1);
            FD_ZERO(fds.get());
            FD_SET(self._r, fds.get());
            // struct timeval timeout;
            let mut timeout = timeval {
                tv_sec: 0,
                tv_usec: 0,
            };
            if (timeout_ >= 0) {
                timeout.tv_sec = timeout_ / 1000;
                timeout.tv_usec = timeout_ % 1000 * 1000;
            }
            // #ifdef ZMQ_HAVE_WINDOWS
            #[cfg(target_os = "windows")]
            {
                let rc = select(0, fds.get(), None, None, if timeout_ >= 0 { Some(&timeout as *const TIMEVAL) } else { None });
                // wsa_assert (rc != SOCKET_ERROR);
            }
            // #else
            #[cfg(not(target_os = "windows"))]
            {
                int
                rc = select(_r + 1, fds.get(), NULL, NULL, timeout_ >= 0? & timeout: NULL);
                if (unlikely(rc < 0)) {
                    errno_assert(errno == EINTR);
                    return -1;
                }
            }
            // #endif
            if ((rc == 0)) {
                // errno = EAGAIN;
                return -1;
            }
            // zmq_assert (rc == 1);
            return 0;
        }
        // #else
        // #Error
        // #endif
    }

    pub unsafe fn recv(&mut self) {
        //  Attempt to read a signal.
        // #if defined ZMQ_HAVE_EVENTFD
        let mut dummy = 0u64;
        #[cfg(feature = "eventfd")]
        {
            // uint64_t dummy;

            let sz = read(self._r as c_int, &dummy as *mut c_void, 8);
            // errno_assert (sz == sizeof (dummy));

            //  If we accidentally grabbed the next signal(s) along with the current
            //  one, return it back to the eventfd object.
            if ((dummy > 1)) {
                let mut inc = dummy - 1;
                let sz2 = write(self._w as c_int, &inc as *mut c_void, 8);
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
            let nbytes = recv(self._r, &dummy as &mut [u8], 0 as SEND_RECV_FLAGS);
            // wsa_assert (nbytes != SOCKET_ERROR);
        }
        // #elif defined ZMQ_HAVE_VXWORKS
        //     ssize_t nbytes = ::recv (_r, (char *) &dummy, sizeof (dummy), 0);
        //     errno_assert (nbytes >= 0);
        // #else
        #[cfg(not(target_os = "windows"))]
        {
            let nbytes = recv(_r, &dummy, 8, 0);
            // errno_assert (nbytes >= 0);
        }
        // #endif
        // zmq_assert (nbytes == sizeof (dummy));
        // zmq_assert (dummy == 0);
        // #endif
    }

    pub unsafe fn recv_failable(&mut self) -> i32 {
        //  Attempt to read a signal.
        // #if defined ZMQ_HAVE_EVENTFD
        let mut dummy = 0u64;
        #[cfg(feature = "eventfd")]
        {
            let sz = read(self._r, &dummy, 8);
            if (sz == -1) {
                // errno_assert (errno == EAGAIN);
                return -1;
            }
            // errno_assert (sz == sizeof (dummy));

            //  If we accidentally grabbed the next signal(s) along with the current
            //  one, return it back to the eventfd object.
            if ((dummy > 1)) {
                let inc = dummy - 1;
                let sz2 = write(self._w, &inc, 8);
                // errno_assert (sz2 == sizeof (inc));
                return 0;
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
            let nbytes = recv(self._r, (&dummy) as &mut [u8], 0 as SEND_RECV_FLAGS);
            if nbytes == SOCKET_ERROR {
                let last_error = WSAGetLastError();
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
            let nbytes = recv(self._r, &dummy, sizeof(dummy), 0);
            if (nbytes == -1) {
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
        return 0;
    }

    pub fn valid(&mut self) -> bool {
        self._w != -1
    }

    pub unsafe fn forked(&mut self) {
        #[cfg(feature = "fork")]
        {
            // self.pid = getpid();
            close(self._r as c_int);
            close(self._w as c_int);
            make_fdpair(&mut self._r, &mut self._w);
        }
    }
}

pub unsafe fn sleep_ms(ms_: u32) -> i32 {
    if ms_ == 0 {
        return 0;
    }
    return usleep(ms_ * 1000);
}

pub unsafe fn close_wait_ms(fd_: i32, max_ms_: u32) -> i32 {
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
        rc = libc::close(fd_);
    }

    rc
}
