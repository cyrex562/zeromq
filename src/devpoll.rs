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
// #include "devpoll.hpp"
// #if defined ZMQ_IOTHREAD_POLLER_USE_DEVPOLL

// #include <sys/devpoll.h>
// #include <sys/time.h>
// #include <sys/types.h>
// #include <sys/stat.h>
// #include <sys/ioctl.h>
// #include <fcntl.h>
// #include <unistd.h>
// #include <limits.h>
// #include <algorithm>

// #include "devpoll.hpp"
// #include "err.hpp"
// #include "config.hpp"
// #include "i_poll_events.hpp"
pub struct devpoll_t ZMQ_FINAL : public worker_poller_base_t
{
// public:
    typedef fd_t handle_t;

    devpoll_t (const ThreadCtx &ctx);
    ~devpoll_t () ZMQ_FINAL;

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

    //  File descriptor referring to "/dev/poll" pseudo-device.
    fd_t devpoll_fd;

    struct fd_entry_t
    {
        short events;
        i_poll_events *reactor;
        valid: bool
        accepted: bool
    };

    typedef std::vector<fd_entry_t> fd_table_t;
    fd_table_t fd_table;

    typedef std::vector<fd_t> pending_list_t;
    pending_list_t pending_list;

    //  Pollset manipulation function.
    void devpoll_ctl (fd_t fd_, short events_);

    ZMQ_NON_COPYABLE_NOR_MOVABLE (devpoll_t)
};

typedef devpoll_t poller_t;

devpoll_t::devpoll_t (const ThreadCtx &ctx) :
    worker_poller_base_t (ctx)
{
    devpoll_fd = open ("/dev/poll", O_RDWR);
    errno_assert (devpoll_fd != -1);
}

devpoll_t::~devpoll_t ()
{
    //  Wait till the worker thread exits.
    stop_worker ();

    close (devpoll_fd);
}

void devpoll_t::devpoll_ctl (fd_t fd_, short events_)
{
    struct pollfd pfd = {fd_, events_, 0};
    ssize_t rc = write (devpoll_fd, &pfd, sizeof pfd);
    zmq_assert (rc == sizeof pfd);
}

devpoll_t::handle_t devpoll_t::add_fd (fd_t fd_,
                                                 i_poll_events *reactor_)
{
    check_thread ();
    //  If the file descriptor table is too small expand it.
    fd_table_t::size_type sz = fd_table.size ();
    if (sz <= (fd_table_t::size_type) fd_) {
        fd_table.resize (fd_ + 1);
        while (sz != (fd_table_t::size_type) (fd_ + 1)) {
            fd_table[sz].valid = false;
            ++sz;
        }
    }

    zmq_assert (!fd_table[fd_].valid);

    fd_table[fd_].events = 0;
    fd_table[fd_].reactor = reactor_;
    fd_table[fd_].valid = true;
    fd_table[fd_].accepted = false;

    devpoll_ctl (fd_, 0);
    pending_list.push_back (fd_);

    //  Increase the load metric of the thread.
    adjust_load (1);

    return fd_;
}

void devpoll_t::rm_fd (handle_t handle_)
{
    check_thread ();
    zmq_assert (fd_table[handle_].valid);

    devpoll_ctl (handle_, POLLREMOVE);
    fd_table[handle_].valid = false;

    //  Decrease the load metric of the thread.
    adjust_load (-1);
}

void devpoll_t::set_pollin (handle_t handle_)
{
    check_thread ();
    devpoll_ctl (handle_, POLLREMOVE);
    fd_table[handle_].events |= POLLIN;
    devpoll_ctl (handle_, fd_table[handle_].events);
}

void devpoll_t::reset_pollin (handle_t handle_)
{
    check_thread ();
    devpoll_ctl (handle_, POLLREMOVE);
    fd_table[handle_].events &= ~((short) POLLIN);
    devpoll_ctl (handle_, fd_table[handle_].events);
}

void devpoll_t::set_pollout (handle_t handle_)
{
    check_thread ();
    devpoll_ctl (handle_, POLLREMOVE);
    fd_table[handle_].events |= POLLOUT;
    devpoll_ctl (handle_, fd_table[handle_].events);
}

void devpoll_t::reset_pollout (handle_t handle_)
{
    check_thread ();
    devpoll_ctl (handle_, POLLREMOVE);
    fd_table[handle_].events &= ~((short) POLLOUT);
    devpoll_ctl (handle_, fd_table[handle_].events);
}

void devpoll_t::stop ()
{
    check_thread ();
}

int devpoll_t::max_fds ()
{
    return -1;
}

void devpoll_t::loop ()
{
    while (true) {
        struct pollfd ev_buf[max_io_events];
        struct dvpoll poll_req;

        for (pending_list_t::size_type i = 0; i < pending_list.size (); i++)
            fd_table[pending_list[i]].accepted = true;
        pending_list.clear ();

        //  Execute any due timers.
        int timeout = (int) execute_timers ();

        if (get_load () == 0) {
            if (timeout == 0)
                break;

            // TODO sleep for timeout
            continue;
        }

        //  Wait for events.
        //  On Solaris, we can retrieve no more then (OPEN_MAX - 1) events.
        poll_req.dp_fds = &ev_buf[0];
// #if defined ZMQ_HAVE_SOLARIS
        poll_req.dp_nfds = std::min ((int) max_io_events, OPEN_MAX - 1);
// #else
        poll_req.dp_nfds = max_io_events;
// #endif
        poll_req.dp_timeout = timeout ? timeout : -1;
        int n = ioctl (devpoll_fd, DP_POLL, &poll_req);
        if (n == -1 && errno == EINTR)
            continue;
        errno_assert (n != -1);

        for (int i = 0; i < n; i++) {
            fd_entry_t *fd_ptr = &fd_table[ev_buf[i].fd];
            if (!fd_ptr.valid || !fd_ptr.accepted)
                continue;
            if (ev_buf[i].revents & (POLLERR | POLLHUP))
                fd_ptr.reactor.in_event ();
            if (!fd_ptr.valid || !fd_ptr.accepted)
                continue;
            if (ev_buf[i].revents & POLLOUT)
                fd_ptr.reactor.out_event ();
            if (!fd_ptr.valid || !fd_ptr.accepted)
                continue;
            if (ev_buf[i].revents & POLLIN)
                fd_ptr.reactor.in_event ();
        }
    }
}

// #endif
