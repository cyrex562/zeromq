use std::intrinsics::
// zmq_assert;
use std::mem;
use std::ptr::null_mut;
use std::thread::{sleep, sleep_ms};
use std::time::Duration;

use anyhow::{anyhow, bail};
use libc::{c_int, c_uint, c_void, close, getpid, ssize_t, timeval, EAGAIN, EINTR, EWOULDBLOCK};

use crate::fd::ZmqFileDesc;
use crate::mechanism::ZmqMechanismStatus::error;
use crate::polling_util::optimized_fd_set_t;

#[derive(Default, Debug, Clone)]
pub struct ZmqSignaler {
    //
    //  Underlying write & read file descriptor
    //  Will be -1 if an error occurred during initialization, e.g. we
    //  exceeded the number of available handles
    pub _w: ZmqFileDesc,
    pub _r: ZmqFileDesc,

    // #ifdef HAVE_FORK
    // the process that created this context. Used to detect forking.
    pub pid: pid_t,
    // #endif

    // // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqSignaler)
}

impl ZmqSignaler {
    // idempotent close of file descriptors that is safe to use by destructor
    // and forked().
    // void close_internal ();

    //
    // ZmqSignaler ();
    pub fn new() {
        let mut out = Self {
            _w: 0,
            _r: 0,
            pid: (),
        };
        //  Create the socketpair for signaling.
        if (make_fdpair(&out._r, &out._w) == 0) {
            unblock_socket(out._w);
            unblock_socket(out._r);
        }
        // #ifdef HAVE_FORK
        unsafe {
            out.pid = getpid();
        }
        // #endif
    }

    // ~ZmqSignaler ();

    // Returns the socket/file descriptor
    // May return retired_fd if the signaler could not be initialized.
    // ZmqFileDesc get_fd () const;
    pub fn get_fd(&self) -> ZmqFileDesc {
        return self._r;
    }

    // void send ();

    pub fn send(&self) {
        // #if defined HAVE_FORK
        //     if (unlikely (pid != getpid ())) {
        //         //printf("Child process %d ZmqSignaler::send returning without sending #1\n", getpid());
        //         return; // do not send anything in forked child context
        //     }
        // #endif
        // #if defined ZMQ_HAVE_EVENTFD
        let inc = 1;
        let sz = unsafe {
            libc::write(
                self._w as c_int,
                &inc as *const c_void,
                mem::size_of_val(&inc) as c_uint,
            )
        };
        // errno_assert (sz == mem::size_of::<inc>());
        // #elif defined ZMQ_HAVE_WINDOWS
        if cfg!(windows) {
            let dummy = 0;
            let mut nbytes: i32;
            loop {
                nbytes = send(self._w, &dummy, mem::size_of_val(&dummy), 0);
                wsa_assert(nbytes != SOCKET_ERROR);
                // wsa_assert does not abort on WSAEWOULDBLOCK. If we get this, we retry.
                if nbytes != SOCKET_ERROR {
                    break;
                }
            }
            // while (nbytes == SOCKET_ERROR);
            // Given the small size of dummy (should be 1) expect that send was able to send everything.
            // zmq_assert(nbytes == mem::size_of::<dummy>());
        }
        // #elif defined ZMQ_HAVE_VXWORKS
        //     unsigned char dummy = 0;
        //     while (true) {
        //         ssize_t nbytes = ::send (_w, (char *) &dummy, mem::size_of::<dummy>(), 0);
        //         if (unlikely (nbytes == -1 && errno == EINTR))
        //             continue;
        // // #if defined(HAVE_FORK)
        //         if (unlikely (pid != getpid ())) {
        //             //printf("Child process %d ZmqSignaler::send returning without sending #2\n", getpid());
        //             errno = EINTR;
        //             break;
        //         }
        // // #endif
        //         zmq_assert (nbytes == sizeof dummy);
        //         break;
        //     }
        // #else
        else {
            let dummy = 0;
            loop {
                let nbytes = send(self._w, &dummy, mem::size_of::<dummy>(), 0);
                if ((nbytes == -1 && errno == EINTR)) {
                    continue;
            }
            // #if defined(HAVE_FORK)
            unsafe {
                if ((self.pid != getpid())) {
                    //printf("Child process %d ZmqSignaler::send returning without sending #2\n", getpid());
                    errno = EINTR;
                break;
            }
        }
        // #endif
        //             zmq_assert(nbytes == sizeof dummy);
        break;
    }
}
// #endif
}

// int wait (timeout: i32) const;

pub fn wait(&self, timeout: i32) -> anyhow::Result<()> {
    // #ifdef HAVE_FORK
    #[cfg(feature = "fork")]
    if ((pid != std::process::id())) {
        // we have forked and the file descriptor is closed. Emulate an interrupt
        // response.
        //printf("Child process %d ZmqSignaler::wait returning simulating interrupt #1\n", getpid());
        // errno = EINTR;
        // return -1;
        bail!("error: EINTR");
}
// #endif

// #ifdef ZMQ_POLL_BASED_ON_POLL
# [cfg(feature = "poll")]
{
let pfd: pollfd = pollfd {};
pfd.fd = self._r;
pfd.events = POLLIN;
let rc: i32 = libc::poll( & pfd, 1, timeout);
if ((rc < 0)) {
// errno_assert(errno == EINTR);
// return -1;
bail ! ("error: EINTR");
}
if ((rc == 0)) {
// errno = EAGAIN;
// return -1;
bail!("error: EAGAIN")
}
// #ifdef HAVE_FORK
unsafe {
# [cfg(feature = "fork")]
if ((pid != std::process::id())) {
// we have forked and the file descriptor is closed. Emulate an interrupt
// response.
//printf("Child process %d ZmqSignaler::wait returning simulating interrupt #2\n", getpid());
// errno = EINTR;
// return -1;
bail!("error: EINTR")
}
}
// #endif
//     zmq_assert (rc == 1);
//     zmq_assert (pfd.revents & POLLIN);
//             return 0;
return Ok(());
}

// #elif defined ZMQ_POLL_BASED_ON_SELECT
# [cfg(feature = "select")]
{
fds: optimized_fd_set_t = optimized_fd_set_t {}; //(1);
FD_ZERO(fds.get());
FD_SET(_r, fds.get());
let mut timeout: libc::timeval = libc::timeval {
tv_sec: 0,
tv_usec: 0,
};
timeout;
if (timeout > = 0) {
timeout.tv_sec = timeout / 1000;
timeout.tv_usec = timeout % 1000 * 1000;
}
// #ifdef ZMQ_HAVE_WINDOWS
let rc = select(
0,
fds.get(),
null_mut(),
null_mut(),
timeout > = 0 ? & timeout: null_mut(),
);
wsa_assert(rc != SOCKET_ERROR);
// #else
let rc = select(
_r + 1,
fds.get(),
null_mut(),
null_mut(),
timeout > = 0 ? & timeout: null_mut(),
);
if ((rc < 0)) {
// errno_assert(errno == EINTR);
// return -1;
bail ! ("EINTR")
}
// #endif
if ((rc == 0)) {
// errno = EAGAIN;
// return -1;
bail!("EAGAIN");
}
// zmq_assert(rc == 1);
// return 0;
return Ok(());
}
// #else
// #error
// #endif
}

// void recv ();
pub fn recv(&mut self) {
    //  Attempt to read a signal.
    // #if defined ZMQ_HAVE_EVENTFD
    #[cfg(feature = "eventfd")]
    {
        let mut dummy = 0u64;
        let sz = unsafe {
            libc::read(
                self._r as c_int,
                &dummy as *mut c_void,
                mem::size_of::<dummy>() as c_uint,
            )
        };
        // errno_assert (sz == mem::size_of::<dummy>());

        //  If we accidentally grabbed the next signal(s) along with the current
        //  one, return it back to the eventfd object.
        if ((dummy > 1)) {
            let mut inc = dummy - 1;
        let sz2 = unsafe {
            libc::write(
                self._w as c_int,
                &inc as *mut c_void,
                mem::size_of::<inc>() as c_uint,
            )
        };
        // errno_assert (sz2 == mem::size_of::<inc>());
        return;
    }
}

// zmq_assert (dummy == 1);
// #else
# [cfg(not(feature = "eventfd"))]
{
let mut dummy = 0u8;
// #if defined ZMQ_HAVE_WINDOWS
# [cfg(target_os = "windows")]
{
let nbytes: i32 = recv( self._r, & dummy, mem::size_of::< dummy> (), 0);
// libc::wsa_assert(nbytes != SOCKET_ERROR);
}
// # elif defined ZMQ_HAVE_VXWORKS
// ssize_t
// nbytes = ::recv(_r, (char *) & dummy, mem::size_of::<dummy>(), 0);
// errno_assert(nbytes >= 0);

// #else
# [cfg(not(target_os = "windows"))]
let nbytes = recv(_r, &dummy, mem::size_of::< dummy > (), 0);
// errno_assert(nbytes >= 0);
// #endif
//             zmq_assert(nbytes == mem::size_of::<dummy>());
//             zmq_assert(dummy == 0);
// #endif
}
}

// int recv_failable ();

pub fn recv_failable(&mut self) -> anyhow::Result<()> {
    //  Attempt to read a signal.
    // #if defined ZMQ_HAVE_EVENTFD
    unsafe {
        #[cfg(feature = "eventfd")]
        {
            let mut dummy = 0u64;
            let mut sz = libc::read(
                self._r as c_int,
                &dummy as *mut c_void,
                mem::size_of_val(&dummy) as c_uint,
            );
            if (sz == -1) {
                // errno_assert (errno == EAGAIN);
                // return -1;
                bail!("error EGAIN")
            }
            // errno_assert (sz == mem::size_of::<dummy>());

            //  If we accidentally grabbed the next signal(s) along with the current
            //  one, return it back to the eventfd object.
            if ((dummy > 1)) {
                let inc = dummy - 1;
            let sz2 = libc::write(
                self._w as c_int,
                &inc as *mut c_void,
                mem::size_of_val(&inc) as c_uint,
            );
            // errno_assert(sz2 == mem::size_of::<inc>());
            // return 0;
            return Ok(());
        }

        // zmq_assert(dummy == 1);
    }
}
// #else
# [cfg(not(feature = "eventfd"))]
{
let mut dummy = 0u8;
// #if defined ZMQ_HAVE_WINDOWS
# [cfg(target_os = "windows")]
{
let nbytes: i32 = recv( self._r, ( & dummy), mem::size_of::< dummy > (), 0);
if (nbytes == SOCKET_ERROR) {
let last_error: i32 = WSAGetLastError();
if (last_error == WSAEWOULDBLOCK) {
// errno = EAGAIN;
// return -1;
bail ! ("error: EAGAIN")
}
wsa_assert(last_error == WSAEWOULDBLOCK);
}
}
// #elif defined ZMQ_HAVE_VXWORKS
//     ssize_t nbytes = ::recv (_r, (char *) &dummy, mem::size_of::<dummy>(), 0);
//     if (nbytes == -1) {
//         if (errno == EAGAIN || errno == EWOULDBLOCK || errno == EINTR) {
//             errno = EAGAIN;
//             return -1;
//         }
//         errno_assert (errno == EAGAIN || errno == EWOULDBLOCK
//                       || errno == EINTR);
//     }
// #else
# [cfg(not(target_os = "windows"))]
{
let nbytes = libc::recv( self._r, & dummy, mem::size_of::< dummy> (), 0);
if (nbytes == - 1) {
if (errno == EAGAIN | | errno == EWOULDBLOCK | | errno == EINTR) {
// errno = EAGAIN;
// return -1;
bail ! ("error EAGAIN")
}
// errno_assert(errno == EAGAIN || errno == EWOULDBLOCK
//     || errno == EINTR);
}
}
// #endif
// zmq_assert(nbytes == mem::size_of::<dummy>());
// zmq_assert(dummy == 0);
}
// #endif
//     return 0;
return Ok(());
}

// bool valid () const;
pub fn valid(&mut self) -> bool {
    return self._w != retired_fd;
}

// #ifdef HAVE_FORK
// close the file descriptors in a forked child process so that they
// do not interfere with the context in the parent process.
// void forked ();
// #endif
// #ifdef HAVE_FORK
#[cfg(feature = "fork")]
pub fn forked(&mut self) {
    //  Close file descriptors created in the parent and create new pair
    unsafe {
        libc::close(self._r as c_int);
    }
    unsafe {
        libc::close(self._w as c_int);
    }
    make_fdpair(&self._r, &self._w);
}
// #endif
}

// #if !defined(ZMQ_HAVE_WINDOWS)
// Helper to sleep for specific number of milliseconds (or until signal)
//
// pub fn sleep_ms (ms: i32) -> anyhow::Result<()>
// {
//     if (ms == 0) {
//         return Ok(());
//     }
// // // #if defined ZMQ_HAVE_ANDROID
// //     usleep (ms * 1000);
// //     return 0;
// // // TODO:
// // // #elif defined ZMQ_HAVE_VXWORKS
// // //     struct timespec ns_;
// // //     ns_.tv_sec = ms / 1000;
// // //     ns_.tv_nsec = ms % 1000 * 1000000;
// // //     return nanosleep (&ns_, 0);
// // // // #else
// //     return usleep (ms * 1000);
// // // #endif
//     let dur = Duration::new((ms * 1000) as u64, 0);
//     sleep(dur);
//     Ok(())
// }

// Helper to wait on close(), for non-blocking sockets, until it completes
// If EAGAIN is received, will sleep briefly (1-100ms) then try again, until
// the overall timeout is reached.
//
pub fn close_wait_ms(fd: i32, max_ms_: u32) -> i32 {
    let mut ms_so_far = 0u32;
    let min_step_ms = 1u32;
    let max_step_ms = 100u32;
    let step_ms = u32::min(u32::max(min_step_mx, (max_ms_ / 10) as u32), max_step_ms);
    // std::min (std::max (min_step_ms, max_ms_ / 10), max_step_ms);

    let mut rc = 0; // do not sleep on first attempt
    loop {
        if (rc == -1 && errno == EAGAIN) {
            sleep_ms(step_ms);
            ms_so_far += step_ms;
        }
        unsafe {
            rc = libc::close(fd);
        }
        if !(ms_so_far < max_ms_ && rc == -1 && errno == EAGAIN) {
            break;
        }
    }

    return rc;
}
// #endif

// This might get run after some part of construction failed, leaving one or
// both of _r and _w retired_fd.
// ZmqSignaler::~ZmqSignaler ()
// {
// // #if defined ZMQ_HAVE_EVENTFD
//     if (_r == retired_fd)
//         return;
//     int rc = close_wait_ms (_r);
//     errno_assert (rc == 0);
// #elif defined ZMQ_HAVE_WINDOWS
//     if (_w != retired_fd) {
//         const struct linger so_linger = {1, 0};
//         int rc = setsockopt (_w, SOL_SOCKET, SO_LINGER,
//                              reinterpret_cast<const char *> (&so_linger),
//                              sizeof so_linger);
//         //  Only check shutdown if WSASTARTUP was previously done
//         if (rc == 0 || WSAGetLastError () != WSANOTINITIALISED) {
//             wsa_assert (rc != SOCKET_ERROR);
//             rc = closesocket (_w);
//             wsa_assert (rc != SOCKET_ERROR);
//             if (_r == retired_fd)
//                 return;
//             rc = closesocket (_r);
//             wsa_assert (rc != SOCKET_ERROR);
//         }
//     }
// // #else
//     if (_w != retired_fd) {
//         int rc = close_wait_ms (_w);
//         errno_assert (rc == 0);
//     }
//     if (_r != retired_fd) {
//         int rc = close_wait_ms (_r);
//         errno_assert (rc == 0);
//     }
// // #endif
// }
