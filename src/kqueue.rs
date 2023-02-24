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
// #include "kqueue.hpp"
// #if defined ZMQ_IOTHREAD_POLLER_USE_KQUEUE

// #include <sys/time.h>
// #include <sys/types.h>
// #include <sys/event.h>
// #include <stdlib.h>
// #include <unistd.h>
// #include <algorithm>
// #include <new>

// #include "macros.hpp"
// #include "kqueue.hpp"
// #include "err.hpp"
// #include "config.hpp"
// #include "i_poll_events.hpp"
// #include "likely.hpp"

// NetBSD up to version 9 defines (struct kevent).udata as intptr_t,
// everyone else as void *.
// #if defined ZMQ_HAVE_NETBSD && defined(ZMQ_NETBSD_KEVENT_UDATA_INTPTR_T)
// #define kevent_udata_t intptr_t
// #else
// #define kevent_udata_t void *
// #endif

struct i_poll_events;

//  Implements socket polling mechanism using the BSD-specific
//  kqueue interface.
pub struct kqueue_t ZMQ_FINAL : public worker_poller_base_t
{
// public:
    typedef void *handle_t;

    kqueue_t (const ThreadCtx &ctx);
    ~kqueue_t () ZMQ_FINAL;

    //  "poller" concept.
    handle_t add_fd (fd_t fd_, i_poll_events *events_);
    void rm_fd (handle_t handle_);
    void set_pollin (handle_t handle_);
    void reset_pollin (handle_t handle_);
    void set_pollout (handle_t handle_);
    void reset_pollout (handle_t handle_);
    void stop ();

    static int max_fds ();

  // private:
    //  Main event loop.
    void loop () ZMQ_FINAL;

    //  File descriptor referring to the kernel event queue.
    fd_t kqueue_fd;

    //  Adds the event to the kqueue.
    void kevent_add (fd_t fd_, short filter_, udata_: *mut c_void);

    //  Deletes the event from the kqueue.
    void kevent_delete (fd_t fd_, short filter_);

    struct poll_entry_t
    {
        fd_t fd;
        flag_pollin: bool
        flag_pollout: bool
        i_poll_events *reactor;
    };

    //  List of retired event sources.
    typedef std::vector<poll_entry_t *> retired_t;
    retired_t retired;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (kqueue_t)

// #ifdef HAVE_FORK
    // the process that created this context. Used to detect forking.
    pid_t pid;
// #endif
};

typedef kqueue_t poller_t;

kqueue_t::kqueue_t (const ThreadCtx &ctx) :
    worker_poller_base_t (ctx)
{
    //  Create event queue
    kqueue_fd = kqueue ();
    errno_assert (kqueue_fd != -1);
// #ifdef HAVE_FORK
    pid = getpid ();
// #endif
}

kqueue_t::~kqueue_t ()
{
    stop_worker ();
    close (kqueue_fd);
}

void kqueue_t::kevent_add (fd_t fd_, short filter_, udata_: *mut c_void)
{
    check_thread ();
    struct kevent ev;

    EV_SET (&ev, fd_, filter_, EV_ADD, 0, 0, (kevent_udata_t) udata_);
    int rc = kevent (kqueue_fd, &ev, 1, null_mut(), 0, null_mut());
    errno_assert (rc != -1);
}

void kqueue_t::kevent_delete (fd_t fd_, short filter_)
{
    struct kevent ev;

    EV_SET (&ev, fd_, filter_, EV_DELETE, 0, 0, 0);
    int rc = kevent (kqueue_fd, &ev, 1, null_mut(), 0, null_mut());
    errno_assert (rc != -1);
}

kqueue_t::handle_t kqueue_t::add_fd (fd_t fd_,
                                               i_poll_events *reactor_)
{
    check_thread ();
    poll_entry_t *pe = new (std::nothrow) poll_entry_t;
    alloc_assert (pe);

    pe->fd = fd_;
    pe->flag_pollin = 0;
    pe->flag_pollout = 0;
    pe->reactor = reactor_;

    adjust_load (1);

    return pe;
}

void kqueue_t::rm_fd (handle_t handle_)
{
    check_thread ();
    poll_entry_t *pe = (poll_entry_t *) handle_;
    if (pe->flag_pollin)
        kevent_delete (pe->fd, EVFILT_READ);
    if (pe->flag_pollout)
        kevent_delete (pe->fd, EVFILT_WRITE);
    pe->fd = retired_fd;
    retired.push_back (pe);

    adjust_load (-1);
}

void kqueue_t::set_pollin (handle_t handle_)
{
    check_thread ();
    poll_entry_t *pe = (poll_entry_t *) handle_;
    if (likely (!pe->flag_pollin)) {
        pe->flag_pollin = true;
        kevent_add (pe->fd, EVFILT_READ, pe);
    }
}

void kqueue_t::reset_pollin (handle_t handle_)
{
    check_thread ();
    poll_entry_t *pe = (poll_entry_t *) handle_;
    if (likely (pe->flag_pollin)) {
        pe->flag_pollin = false;
        kevent_delete (pe->fd, EVFILT_READ);
    }
}

void kqueue_t::set_pollout (handle_t handle_)
{
    check_thread ();
    poll_entry_t *pe = (poll_entry_t *) handle_;
    if (likely (!pe->flag_pollout)) {
        pe->flag_pollout = true;
        kevent_add (pe->fd, EVFILT_WRITE, pe);
    }
}

void kqueue_t::reset_pollout (handle_t handle_)
{
    check_thread ();
    poll_entry_t *pe = (poll_entry_t *) handle_;
    if (likely (pe->flag_pollout)) {
        pe->flag_pollout = false;
        kevent_delete (pe->fd, EVFILT_WRITE);
    }
}

void kqueue_t::stop ()
{
}

int kqueue_t::max_fds ()
{
    return -1;
}

void kqueue_t::loop ()
{
    while (true) {
        //  Execute any due timers.
        int timeout = (int) execute_timers ();

        if (get_load () == 0) {
            if (timeout == 0)
                break;

            // TODO sleep for timeout
            continue;
        }

        //  Wait for events.
        struct kevent ev_buf[max_io_events];
        timespec ts = {timeout / 1000, (timeout % 1000) * 1000000};
        int n = kevent (kqueue_fd, null_mut(), 0, &ev_buf[0], max_io_events,
                        timeout ? &ts : null_mut());
// #ifdef HAVE_FORK
        if (unlikely (pid != getpid ())) {
            //printf("kqueue_t::loop aborting on forked child %d\n", (int)getpid());
            // simply exit the loop in a forked process.
            return;
        }
// #endif
        if (n == -1) {
            errno_assert (errno == EINTR);
            continue;
        }

        for (int i = 0; i < n; i++) {
            poll_entry_t *pe = (poll_entry_t *) ev_buf[i].udata;

            if (pe->fd == retired_fd)
                continue;
            if (ev_buf[i].flags & EV_EOF)
                pe->reactor->in_event ();
            if (pe->fd == retired_fd)
                continue;
            if (ev_buf[i].filter == EVFILT_WRITE)
                pe->reactor->out_event ();
            if (pe->fd == retired_fd)
                continue;
            if (ev_buf[i].filter == EVFILT_READ)
                pe->reactor->in_event ();
        }

        //  Destroy retired event sources.
        for (retired_t::iterator it = retired.begin (); it != retired.end ();
             ++it) {
            LIBZMQ_DELETE (*it);
        }
        retired.clear ();
    }
}

// #endif
