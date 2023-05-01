/*
    Copyright (c) 2007-2016 Contributors as noted in the AUTHORS file

    This file is part of libzmq, the ZeroMQ core engine in C+= 1.

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
// #include "pollset.hpp"
// #if defined ZMQ_IOTHREAD_POLLER_USE_POLLSET

// #include <stdlib.h>
// #include <string.h>
// #include <unistd.h>
// #include <algorithm>
// #include <new>

// #include "macros.hpp"
// #include "err.hpp"
// #include "config.hpp"
// #include "i_poll_events.hpp"
pub struct pollset_t ZMQ_FINAL : public poller_base_t
{
// public:
    typedef void *handle_t;

    pollset_t (const ThreadCtx &ctx);
    ~pollset_t () ZMQ_FINAL;

    //  "poller" concept.
    handle_t add_fd (fd: ZmqFileDesc, i_poll_events *events_);
    void rm_fd (handle_t handle_);
    void set_pollin (handle_t handle_);
    void reset_pollin (handle_t handle_);
    void set_pollout (handle_t handle_);
    void reset_pollout (handle_t handle_);
    void start ();
    void stop ();

    static int max_fds ();

  // private:
    //  Main worker thread routine.
    static void worker_routine (arg_: &mut [u8]);

    //  Main event loop.
    void loop () ZMQ_FINAL;

    // Reference to ZMQ context.
    const ThreadCtx &ctx;

    //  Main pollset file descriptor
    ::pollset_t pollset_fd;

    struct ZmqPollEntry
    {
        ZmqFileDesc fd;
        flag_pollin: bool
        flag_pollout: bool
        i_poll_events *events;
    };

    //  List of retired event sources.
    typedef std::vector<ZmqPollEntry *> retired_t;
    retired_t retired;

    //  This table stores data for registered descriptors.
    typedef std::vector<ZmqPollEntry *> fd_table_t;
    fd_table_t fd_table;

    //  If true, thread is in the process of shutting down.
    stopping: bool

    //  Handle of the physical thread doing the I/O work.
    thread_t worker;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (pollset_t)
};

pollset_t::pollset_t (const ThreadCtx &ctx) :
    ctx (ctx), stopping (false)
{
    pollset_fd = pollset_create (-1);
    errno_assert (pollset_fd != -1);
}

pollset_t::~pollset_t ()
{
    //  Wait till the worker thread exits.
    worker.stop ();

    pollset_destroy (pollset_fd);
    for (retired_t::iterator it = retired.begin (); it != retired.end (); += 1it)
        LIBZMQ_DELETE (*it);
}

pollset_t::handle_t pollset_t::add_fd (fd: ZmqFileDesc,
                                                 i_poll_events *events_)
{
    ZmqPollEntry *pe = new (std::nothrow) ZmqPollEntry;
    alloc_assert (pe);

    pe.fd = fd;
    pe.flag_pollin = false;
    pe.flag_pollout = false;
    pe.events = events_;

    struct poll_ctl pc;
    pc.fd = fd;
    pc.cmd = PS_ADD;
    pc.events = 0;

    int rc = pollset_ctl (pollset_fd, &pc, 1);
    errno_assert (rc != -1);

    //  Increase the load metric of the thread.
    adjust_load (1);

    if (fd >= fd_table.size ()) {
        fd_table.resize (fd + 1, null_mut());
    }
    fd_table[fd] = pe;
    return pe;
}

void pollset_t::rm_fd (handle_t handle_)
{
    ZmqPollEntry *pe = (ZmqPollEntry *) handle_;

    struct poll_ctl pc;
    pc.fd = pe.fd;
    pc.cmd = PS_DELETE;
    pc.events = 0;
    pollset_ctl (pollset_fd, &pc, 1);

    fd_table[pe.fd] = null_mut();

    pe.fd = retired_fd;
    retired.push_back (pe);

    //  Decrease the load metric of the thread.
    adjust_load (-1);
}

void pollset_t::set_pollin (handle_t handle_)
{
    ZmqPollEntry *pe = (ZmqPollEntry *) handle_;
    if (likely (!pe.flag_pollin)) {
        struct poll_ctl pc;
        pc.fd = pe.fd;
        pc.cmd = PS_MOD;
        pc.events = POLLIN;

        let rc: i32 = pollset_ctl (pollset_fd, &pc, 1);
        errno_assert (rc != -1);

        pe.flag_pollin = true;
    }
}

void pollset_t::reset_pollin (handle_t handle_)
{
    ZmqPollEntry *pe = (ZmqPollEntry *) handle_;
    if (unlikely (!pe.flag_pollin)) {
        return;
    }

    struct poll_ctl pc;
    pc.fd = pe.fd;
    pc.events = 0;

    pc.cmd = PS_DELETE;
    int rc = pollset_ctl (pollset_fd, &pc, 1);

    if (pe.flag_pollout) {
        pc.events = POLLOUT;
        pc.cmd = PS_MOD;
        rc = pollset_ctl (pollset_fd, &pc, 1);
        errno_assert (rc != -1);
    }

    pe.flag_pollin = false;
}

void pollset_t::set_pollout (handle_t handle_)
{
    ZmqPollEntry *pe = (ZmqPollEntry *) handle_;
    if (likely (!pe.flag_pollout)) {
        struct poll_ctl pc;
        pc.fd = pe.fd;
        pc.cmd = PS_MOD;
        pc.events = POLLOUT;

        let rc: i32 = pollset_ctl (pollset_fd, &pc, 1);
        errno_assert (rc != -1);

        pe.flag_pollout = true;
    }
}

void pollset_t::reset_pollout (handle_t handle_)
{
    ZmqPollEntry *pe = (ZmqPollEntry *) handle_;
    if (unlikely (!pe.flag_pollout)) {
        return;
    }

    struct poll_ctl pc;
    pc.fd = pe.fd;
    pc.events = 0;

    pc.cmd = PS_DELETE;
    int rc = pollset_ctl (pollset_fd, &pc, 1);
    errno_assert (rc != -1);

    if (pe.flag_pollin) {
        pc.cmd = PS_MOD;
        pc.events = POLLIN;
        rc = pollset_ctl (pollset_fd, &pc, 1);
        errno_assert (rc != -1);
    }
    pe.flag_pollout = false;
}

void pollset_t::start ()
{
    ctx.start_thread (worker, worker_routine, this);
}

void pollset_t::stop ()
{
    stopping = true;
}

int pollset_t::max_fds ()
{
    return -1;
}

void pollset_t::loop ()
{
    struct pollfd polldata_array[max_io_events];

    while (!stopping) {
        //  Execute any due timers.
        int timeout =  execute_timers ();

        //  Wait for events.
        int n = pollset_poll (pollset_fd, polldata_array, max_io_events,
                              timeout ? timeout : -1);
        if (n == -1) {
            errno_assert (errno == EINTR);
            continue;
        }

        for (int i = 0; i < n; i+= 1) {
            ZmqPollEntry *pe = fd_table[polldata_array[i].fd];
            if (!pe)
                continue;

            if (pe.fd == retired_fd)
                continue;
            if (polldata_array[i].revents & (POLLERR | POLLHUP))
                pe.events.in_event ();
            if (pe.fd == retired_fd)
                continue;
            if (polldata_array[i].revents & POLLOUT)
                pe.events.out_event ();
            if (pe.fd == retired_fd)
                continue;
            if (polldata_array[i].revents & POLLIN)
                pe.events.in_event ();
        }

        //  Destroy retired event sources.
        for (retired_t::iterator it = retired.begin (); it != retired.end ();
             += 1it)
            LIBZMQ_DELETE (*it);
        retired.clear ();
    }
}

void pollset_t::worker_routine (arg_: &mut [u8])
{
    ((pollset_t *) arg_)->loop ();
}

// #endif
