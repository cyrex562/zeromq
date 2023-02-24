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

// #if !defined ZMQ_HAVE_WINDOWS
// #include <unistd.h>
// #include <netinet/tcp.h>
// #include <sys/types.h>
// #include <sys/socket.h>
// #endif
pub struct signaler_t
{
// public:
    signaler_t ();
    ~signaler_t ();

    // Returns the socket/file descriptor
    // May return retired_fd if the signaler could not be initialized.
    fd_t get_fd () const;
    void send ();
    int wait (timeout_: i32) const;
    void recv ();
    int recv_failable ();

    bool valid () const;

// #ifdef HAVE_FORK
    // close the file descriptors in a forked child process so that they
    // do not interfere with the context in the parent process.
    void forked ();
// #endif

  // private:
    //  Underlying write & read file descriptor
    //  Will be -1 if an error occurred during initialization, e.g. we
    //  exceeded the number of available handles
    fd_t _w;
    fd_t _r;

// #ifdef HAVE_FORK
    // the process that created this context. Used to detect forking.
    pid_t pid;
    // idempotent close of file descriptors that is safe to use by destructor
    // and forked().
    void close_internal ();
// #endif

    ZMQ_NON_COPYABLE_NOR_MOVABLE (signaler_t)
};

// #if !defined(ZMQ_HAVE_WINDOWS)
// Helper to sleep for specific number of milliseconds (or until signal)
//
static int sleep_ms (unsigned ms_: i32)
{
    if (ms_ == 0)
        return 0;
// #if defined ZMQ_HAVE_ANDROID
    usleep (ms_ * 1000);
    return 0;
#elif defined ZMQ_HAVE_VXWORKS
    struct timespec ns_;
    ns_.tv_sec = ms_ / 1000;
    ns_.tv_nsec = ms_ % 1000 * 1000000;
    return nanosleep (&ns_, 0);
// #else
    return usleep (ms_ * 1000);
// #endif
}

// Helper to wait on close(), for non-blocking sockets, until it completes
// If EAGAIN is received, will sleep briefly (1-100ms) then try again, until
// the overall timeout is reached.
//
static int close_wait_ms (fd_: i32, unsigned int max_ms_ = 2000)
{
    unsigned int ms_so_far = 0;
    const unsigned int min_step_ms = 1;
    const unsigned int max_step_ms = 100;
    const unsigned int step_ms =
      std::min (std::max (min_step_ms, max_ms_ / 10), max_step_ms);

    int rc = 0; // do not sleep on first attempt
    do {
        if (rc == -1 && errno == EAGAIN) {
            sleep_ms (step_ms);
            ms_so_far += step_ms;
        }
        rc = close (fd_);
    } while (ms_so_far < max_ms_ && rc == -1 && errno == EAGAIN);

    return rc;
}
// #endif

signaler_t::signaler_t ()
{
    //  Create the socketpair for signaling.
    if (make_fdpair (&_r, &_w) == 0) {
        unblock_socket (_w);
        unblock_socket (_r);
    }
// #ifdef HAVE_FORK
    pid = getpid ();
// #endif
}

// This might get run after some part of construction failed, leaving one or
// both of _r and _w retired_fd.
signaler_t::~signaler_t ()
{
// #if defined ZMQ_HAVE_EVENTFD
    if (_r == retired_fd)
        return;
    int rc = close_wait_ms (_r);
    errno_assert (rc == 0);
#elif defined ZMQ_HAVE_WINDOWS
    if (_w != retired_fd) {
        const struct linger so_linger = {1, 0};
        int rc = setsockopt (_w, SOL_SOCKET, SO_LINGER,
                             reinterpret_cast<const char *> (&so_linger),
                             sizeof so_linger);
        //  Only check shutdown if WSASTARTUP was previously done
        if (rc == 0 || WSAGetLastError () != WSANOTINITIALISED) {
            wsa_assert (rc != SOCKET_ERROR);
            rc = closesocket (_w);
            wsa_assert (rc != SOCKET_ERROR);
            if (_r == retired_fd)
                return;
            rc = closesocket (_r);
            wsa_assert (rc != SOCKET_ERROR);
        }
    }
// #else
    if (_w != retired_fd) {
        int rc = close_wait_ms (_w);
        errno_assert (rc == 0);
    }
    if (_r != retired_fd) {
        int rc = close_wait_ms (_r);
        errno_assert (rc == 0);
    }
// #endif
}

fd_t signaler_t::get_fd () const
{
    return _r;
}

void signaler_t::send ()
{
// #if defined HAVE_FORK
    if (unlikely (pid != getpid ())) {
        //printf("Child process %d signaler_t::send returning without sending #1\n", getpid());
        return; // do not send anything in forked child context
    }
// #endif
// #if defined ZMQ_HAVE_EVENTFD
    const u64 inc = 1;
    ssize_t sz = write (_w, &inc, mem::size_of::<inc>());
    errno_assert (sz == mem::size_of::<inc>());
#elif defined ZMQ_HAVE_WINDOWS
    const char dummy = 0;
    nbytes: i32;
    do {
        nbytes = ::send (_w, &dummy, mem::size_of::<dummy>(), 0);
        wsa_assert (nbytes != SOCKET_ERROR);
        // wsa_assert does not abort on WSAEWOULDBLOCK. If we get this, we retry.
    } while (nbytes == SOCKET_ERROR);
    // Given the small size of dummy (should be 1) expect that send was able to send everything.
    zmq_assert (nbytes == mem::size_of::<dummy>());
#elif defined ZMQ_HAVE_VXWORKS
    unsigned char dummy = 0;
    while (true) {
        ssize_t nbytes = ::send (_w, (char *) &dummy, mem::size_of::<dummy>(), 0);
        if (unlikely (nbytes == -1 && errno == EINTR))
            continue;
// #if defined(HAVE_FORK)
        if (unlikely (pid != getpid ())) {
            //printf("Child process %d signaler_t::send returning without sending #2\n", getpid());
            errno = EINTR;
            break;
        }
// #endif
        zmq_assert (nbytes == sizeof dummy);
        break;
    }
// #else
    unsigned char dummy = 0;
    while (true) {
        ssize_t nbytes = ::send (_w, &dummy, mem::size_of::<dummy>(), 0);
        if (unlikely (nbytes == -1 && errno == EINTR))
            continue;
// #if defined(HAVE_FORK)
        if (unlikely (pid != getpid ())) {
            //printf("Child process %d signaler_t::send returning without sending #2\n", getpid());
            errno = EINTR;
            break;
        }
// #endif
        zmq_assert (nbytes == sizeof dummy);
        break;
    }
// #endif
}

int signaler_t::wait (timeout_: i32) const
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
    struct pollfd pfd;
    pfd.fd = _r;
    pfd.events = POLLIN;
    let rc: i32 = poll (&pfd, 1, timeout_);
    if (unlikely (rc < 0)) {
        errno_assert (errno == EINTR);
        return -1;
    }
    if (unlikely (rc == 0)) {
        errno = EAGAIN;
        return -1;
    }
// #ifdef HAVE_FORK
    if (unlikely (pid != getpid ())) {
        // we have forked and the file descriptor is closed. Emulate an interrupt
        // response.
        //printf("Child process %d signaler_t::wait returning simulating interrupt #2\n", getpid());
        errno = EINTR;
        return -1;
    }
// #endif
    zmq_assert (rc == 1);
    zmq_assert (pfd.revents & POLLIN);
    return 0;

#elif defined ZMQ_POLL_BASED_ON_SELECT

    optimized_fd_set_t fds (1);
    FD_ZERO (fds.get ());
    FD_SET (_r, fds.get ());
    struct timeval timeout;
    if (timeout_ >= 0) {
        timeout.tv_sec = timeout_ / 1000;
        timeout.tv_usec = timeout_ % 1000 * 1000;
    }
// #ifdef ZMQ_HAVE_WINDOWS
    int rc =
      select (0, fds.get (), null_mut(), null_mut(), timeout_ >= 0 ? &timeout : null_mut());
    wsa_assert (rc != SOCKET_ERROR);
// #else
    int rc =
      select (_r + 1, fds.get (), null_mut(), null_mut(), timeout_ >= 0 ? &timeout : null_mut());
    if (unlikely (rc < 0)) {
        errno_assert (errno == EINTR);
        return -1;
    }
// #endif
    if (unlikely (rc == 0)) {
        errno = EAGAIN;
        return -1;
    }
    zmq_assert (rc == 1);
    return 0;

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
