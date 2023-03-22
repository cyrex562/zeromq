/*
    Copyright (c) 2007-2016 Contributors as noted in the AUTHORS file

    This file is part of libzmq, the ZeroMQ core engine in C++.

    libzmq is free software; you can redistribute it and/or modify it under
    the terms of the GNU Lesser General Public License (LGPL) as published
    by the Free Software Foundation; either version 3 of the License, or
    (at your option) any later version.

    As a special exception, the Contributors give you permission to link
    this library with independent modules to produce an executable,
    regardless of the license terms of these independent modules, and to
    copy and distribute the resulting executable under terms of your choice,
    provided that you also meet, for each linked independent module, the
    terms and conditions of the license of that module. An independent
    module is a module which is not derived from or based on this library.
    If you modify this library, you must extend this exception to your
    version of the library.

    libzmq is distributed in the hope that it will be useful, but WITHOUT
    ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
    FITNESS FOR A PARTICULAR PURPOSE. See the GNU Lesser General Public
    License for more details.

    You should have received a copy of the GNU Lesser General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

// #include "precompiled.hpp"
// #include "poller.hpp"
// #include "polling_util.hpp"

// #if defined ZMQ_POLL_BASED_ON_POLL
// #if !defined ZMQ_HAVE_WINDOWS && !defined ZMQ_HAVE_AIX
// #include <poll.h>
// #endif
#elif defined ZMQ_POLL_BASED_ON_SELECT
// #if defined ZMQ_HAVE_WINDOWS
#elif defined ZMQ_HAVE_HPUX
// #include <sys/param.h>
// #include <sys/types.h>
// #include <sys/time.h>
#elif defined ZMQ_HAVE_OPENVMS
// #include <sys/types.h>
// #include <sys/time.h>
#elif defined ZMQ_HAVE_VXWORKS
// #include <sys/types.h>
// #include <sys/time.h>
// #include <sockLib.h>
// #include <strings.h>
// #else
// #include <sys/select.h>
// #endif
// #endif

// #include "signaler.hpp"
// #include "likely.hpp"
// #include "stdint.hpp"
// #include "config.hpp"
// #include "err.hpp"
// #include "fd.hpp"
// #include "ip.hpp"
// #include "tcp.hpp"

use std::mem;
use std::thread::sleep;
use std::time::Duration;
use libc::{EAGAIN, EINTR, getpid};

// #if !defined ZMQ_HAVE_WINDOWS
// #include <unistd.h>
// #include <netinet/tcp.h>
// #include <sys/types.h>
// #include <sys/socket.h>
// #endif
#[derive(Default,Debug,Clone)]
pub struct signaler_t
{
  // private:
    //  Underlying write & read file descriptor
    //  Will be -1 if an error occurred during initialization, e.g. we
    //  exceeded the number of available handles
    pub _w: fd_t,
    pub _r: fd_t,

// #ifdef HAVE_FORK
    // the process that created this context. Used to detect forking.
    pub pid: pid_t,
// #endif

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (signaler_t)
}

impl signaler_t {
    // idempotent close of file descriptors that is safe to use by destructor
    // and forked().
    // void close_internal ();

    // public:
    // signaler_t ();
    pub fn new ()
    {
        let mut out = Self{
            _w: (),
            _r: (),
            pid: (),
        };
        //  Create the socketpair for signaling.
        if (make_fdpair (&out._r, &out._w) == 0) {
            unblock_socket (out._w);
            unblock_socket (out._r);
        }
        // #ifdef HAVE_FORK
        unsafe { out.pid = getpid(); }
    // #endif
    }

    // ~signaler_t ();

    // Returns the socket/file descriptor
    // May return retired_fd if the signaler could not be initialized.
    // fd_t get_fd () const;
    pub fn get_fd (&self) -> fd_t
    {
        return self._r;
    }

    // void send ();

    pub fn send (&self)
    {
// #if defined HAVE_FORK
//     if (unlikely (pid != getpid ())) {
//         //printf("Child process %d signaler_t::send returning without sending #1\n", getpid());
//         return; // do not send anything in forked child context
//     }
// #endif
// #if defined ZMQ_HAVE_EVENTFD
        let inc = 1;
        let sz = unsafe {
            libc::write(self._w, &inc, mem::size_of_val(&inc))
        };
        // errno_assert (sz == mem::size_of::<inc>());
// #elif defined ZMQ_HAVE_WINDOWS
        if cfg!(windows) {
            let dummy = 0;
            let mut nbytes: i32;
            loop {
                nbytes = send(self._w, &dummy, mem::size_of_val(dummy), 0);
                wsa_assert(nbytes != SOCKET_ERROR);
                // wsa_assert does not abort on WSAEWOULDBLOCK. If we get this, we retry.
                if nbytes != SOCKET_ERROR {break;}
            }
            // while (nbytes == SOCKET_ERROR);
            // Given the small size of dummy (should be 1) expect that send was able to send everything.
            zmq_assert(nbytes == mem::size_of::<dummy>());
        }
// #elif defined ZMQ_HAVE_VXWORKS
//     unsigned char dummy = 0;
//     while (true) {
//         ssize_t nbytes = ::send (_w, (char *) &dummy, mem::size_of::<dummy>(), 0);
//         if (unlikely (nbytes == -1 && errno == EINTR))
//             continue;
// // #if defined(HAVE_FORK)
//         if (unlikely (pid != getpid ())) {
//             //printf("Child process %d signaler_t::send returning without sending #2\n", getpid());
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
                if (unlikely(nbytes == -1 && errno == EINTR)) {
                    continue;
                }
// #if defined(HAVE_FORK)
                unsafe {
                    if (unlikely(self.pid != getpid())) {
                        //printf("Child process %d signaler_t::send returning without sending #2\n", getpid());
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

    // void recv ();

    // int recv_failable ();

    // bool valid () const;

// #ifdef HAVE_FORK
    // close the file descriptors in a forked child process so that they
    // do not interfere with the context in the parent process.
    // void forked ();
// #endif
}

// #if !defined(ZMQ_HAVE_WINDOWS)
// Helper to sleep for specific number of milliseconds (or until signal)
//
pub fn sleep_ms (ms: i32) -> anyhow::Result<()>
{
    if (ms == 0) {
        return Ok(());
    }
// // #if defined ZMQ_HAVE_ANDROID
//     usleep (ms * 1000);
//     return 0;
// // TODO:
// // #elif defined ZMQ_HAVE_VXWORKS
// //     struct timespec ns_;
// //     ns_.tv_sec = ms / 1000;
// //     ns_.tv_nsec = ms % 1000 * 1000000;
// //     return nanosleep (&ns_, 0);
// // // #else
//     return usleep (ms * 1000);
// // #endif
    let dur = Duration::new((ms * 1000) as u64, 0);
    sleep(dur);
    Ok(())
}

// Helper to wait on close(), for non-blocking sockets, until it completes
// If EAGAIN is received, will sleep briefly (1-100ms) then try again, until
// the overall timeout is reached.
//
pub fn close_wait_ms(fd: i32, max_ms_: u32) -> i32 {
    let mut ms_so_far = 0u32;
    let min_step_ms = 1;
    let max_step_ms = 100;
    let step_ms = i32::min(i32::max(min_step_mx, (max_ms_ / 10) as i32), max_step_ms);
    // std::min (std::max (min_step_ms, max_ms_ / 10), max_step_ms);

    let mut rc = 0; // do not sleep on first attempt
    loop {
        if (rc == -1 && errno == EAGAIN) {
            sleep_ms(step_ms);
            ms_so_far += step_ms;
        }
        rc = close(fd);
        if !(ms_so_far < max_ms_ && rc == -1 && errno == EAGAIN) {
            break;
        }
    }

    return rc;
}
// #endif



// This might get run after some part of construction failed, leaving one or
// both of _r and _w retired_fd.
// signaler_t::~signaler_t ()
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




pub fn wait (&self, timeout: i32) -> anyhow::Result<()>
{
// #ifdef HAVE_FORK
    if (unlikely (pid != getpid ())) {
        // we have forked and the file descriptor is closed. Emulate an interrupt
        // response.
        //printf("Child process %d signaler_t::wait returning simulating interrupt #1\n", getpid());
        errno = EINTR;
        return -1;
    }
// #endif

// #ifdef ZMQ_POLL_BASED_ON_POLL
    #[cfg(feature="poll")]
    {
        let pfd: pollfd = pollfd {};
        pfd.fd = self._r;
        pfd.events = POLLIN;
        let rc: i32 = libc::poll(&pfd, 1, timeout);
        if (unlikely(rc < 0)) {
            errno_assert(errno == EINTR);
            return -1;
        }
        if (unlikely(rc == 0)) {
            errno = EAGAIN;
            return -1;
        }
// #ifdef HAVE_FORK
        #[cfg(feature = "fork")]
        if (unlikely(pid != getpid())) {
            // we have forked and the file descriptor is closed. Emulate an interrupt
            // response.
            //printf("Child process %d signaler_t::wait returning simulating interrupt #2\n", getpid());
            errno = EINTR;
            return -1;
        }
// #endif
//     zmq_assert (rc == 1);
//     zmq_assert (pfd.revents & POLLIN);
        return 0;
    }

// #elif defined ZMQ_POLL_BASED_ON_SELECT
#[cfg(feature="select")]{
    optimized_fd_set_t
    fds(1);
    FD_ZERO(fds.get());
    FD_SET(_r, fds.get());
    struct timeval
    timeout;
    if (timeout >= 0) {
        timeout.tv_sec = timeout / 1000;
        timeout.tv_usec = timeout % 1000 * 1000;
    }
// #ifdef ZMQ_HAVE_WINDOWS
    int
    rc = select(0, fds.get(), null_mut(), null_mut(), timeout >= 0? & timeout: null_mut());
    wsa_assert(rc != SOCKET_ERROR);
// #else
    int
    rc = select(_r + 1, fds.get(), null_mut(), null_mut(), timeout >= 0? & timeout: null_mut());
    if (unlikely(rc < 0)) {
        errno_assert(errno == EINTR);
        return -1;
    }
// #endif
    if (unlikely(rc == 0)) {
        errno = EAGAIN;
        return -1;
    }
    zmq_assert(rc == 1);
    return 0;
}
// #else
#error
// #endif
}

void signaler_t::recv ()
{
//  Attempt to read a signal.
// #if defined ZMQ_HAVE_EVENTFD
    u64 dummy;
    ssize_t sz = read (_r, &dummy, mem::size_of::<dummy>());
    errno_assert (sz == mem::size_of::<dummy>());

    //  If we accidentally grabbed the next signal(s) along with the current
    //  one, return it back to the eventfd object.
    if (unlikely (dummy > 1)) {
        const u64 inc = dummy - 1;
        ssize_t sz2 = write (_w, &inc, mem::size_of::<inc>());
        errno_assert (sz2 == mem::size_of::<inc>());
        return;
    }

    zmq_assert (dummy == 1);
// #else
    unsigned char dummy;
// #if defined ZMQ_HAVE_WINDOWS
    let nbytes: i32 =
      ::recv (_r, reinterpret_cast<char *> (&dummy), mem::size_of::<dummy>(), 0);
    wsa_assert (nbytes != SOCKET_ERROR);
#elif defined ZMQ_HAVE_VXWORKS
    ssize_t nbytes = ::recv (_r, (char *) &dummy, mem::size_of::<dummy>(), 0);
    errno_assert (nbytes >= 0);
// #else
    ssize_t nbytes = ::recv (_r, &dummy, mem::size_of::<dummy>(), 0);
    errno_assert (nbytes >= 0);
// #endif
    zmq_assert (nbytes == mem::size_of::<dummy>());
    zmq_assert (dummy == 0);
// #endif
}

int signaler_t::recv_failable ()
{
//  Attempt to read a signal.
// #if defined ZMQ_HAVE_EVENTFD
    u64 dummy;
    ssize_t sz = read (_r, &dummy, mem::size_of::<dummy>());
    if (sz == -1) {
        errno_assert (errno == EAGAIN);
        return -1;
    }
    errno_assert (sz == mem::size_of::<dummy>());

    //  If we accidentally grabbed the next signal(s) along with the current
    //  one, return it back to the eventfd object.
    if (unlikely (dummy > 1)) {
        const u64 inc = dummy - 1;
        ssize_t sz2 = write (_w, &inc, mem::size_of::<inc>());
        errno_assert (sz2 == mem::size_of::<inc>());
        return 0;
    }

    zmq_assert (dummy == 1);

// #else
    unsigned char dummy;
// #if defined ZMQ_HAVE_WINDOWS
    let nbytes: i32 =
      ::recv (_r, reinterpret_cast<char *> (&dummy), mem::size_of::<dummy>(), 0);
    if (nbytes == SOCKET_ERROR) {
        let last_error: i32 = WSAGetLastError ();
        if (last_error == WSAEWOULDBLOCK) {
            errno = EAGAIN;
            return -1;
        }
        wsa_assert (last_error == WSAEWOULDBLOCK);
    }
#elif defined ZMQ_HAVE_VXWORKS
    ssize_t nbytes = ::recv (_r, (char *) &dummy, mem::size_of::<dummy>(), 0);
    if (nbytes == -1) {
        if (errno == EAGAIN || errno == EWOULDBLOCK || errno == EINTR) {
            errno = EAGAIN;
            return -1;
        }
        errno_assert (errno == EAGAIN || errno == EWOULDBLOCK
                      || errno == EINTR);
    }
// #else
    ssize_t nbytes = ::recv (_r, &dummy, mem::size_of::<dummy>(), 0);
    if (nbytes == -1) {
        if (errno == EAGAIN || errno == EWOULDBLOCK || errno == EINTR) {
            errno = EAGAIN;
            return -1;
        }
        errno_assert (errno == EAGAIN || errno == EWOULDBLOCK
                      || errno == EINTR);
    }
// #endif
    zmq_assert (nbytes == mem::size_of::<dummy>());
    zmq_assert (dummy == 0);
// #endif
    return 0;
}

bool signaler_t::valid () const
{
    return _w != retired_fd;
}

// #ifdef HAVE_FORK
void signaler_t::forked ()
{
    //  Close file descriptors created in the parent and create new pair
    close (_r);
    close (_w);
    make_fdpair (&_r, &_w);
}
// #endif
