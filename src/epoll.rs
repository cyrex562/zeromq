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
// #if defined ZMQ_IOTHREAD_POLLER_USE_EPOLL
// #include "epoll.hpp"

// #if !defined ZMQ_HAVE_WINDOWS
// #include <unistd.h>
// #endif

// #include <stdlib.h>
// #include <string.h>
// #include <signal.h>
// #include <algorithm>
// #include <new>

use std::ffi::c_void;
use crate::fd::ZmqFileDesc;
use crate::poller_base::WorkerPollerBase;
use crate::thread_ctx::ThreadCtx;

// #include "macros.hpp"
// #include "err.hpp"
// #include "config.hpp"
// #include "i_poll_events.hpp"

pub type handle_t = *mut c_void;
#[cfg(target_os="windows")]
pub type epoll_fd_t = HANDLE;
#[cfg(target_os="linux")]
pub type epoll_fd_t = ZmqFileDesc;

#[derive(Default,Debug,Clone)]
pub struct ZmqPollEntry
{
    // ZmqFileDesc fd;
    pub fd: ZmqFileDesc,
    // epoll_event ev;
    pub ev: epoll_event,
    // i_poll_events *events;
    pub events: Vec<i_poll_events>,
}

// enum
//     {
//         epoll_retired_fd = retired_fd
//     };

pub struct ZmqEpoll
{
// public:
//     typedef void *handle_t;
    pub epoll_retired_fd: epoll_fd_t,


  // private:
// #if defined ZMQ_HAVE_WINDOWS
//     typedef HANDLE epoll_fd_t;
//     static const epoll_fd_t epoll_retired_fd;
// #else
//     typedef ZmqFileDesc epoll_fd_t;

// #endif

    //  Main event loop.


    //  Main epoll file descriptor
    // epoll_fd_t _epoll_fd;
    pub _epoll_fd: epoll_fd_t,


    //  List of retired event sources.
    // typedef std::vector<ZmqPollEntry *> retired_t;
    // retired_t _retired;
    pub _retired: Vec<ZmqPollEntry>,

    // public WorkerPollerBase
    pub worker_poller_base: WorkerPollerBase

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqEpoll)
}

impl ZmqEpoll {
    // ZmqEpoll (const ThreadCtx &ctx);
    pub fn new(ctx: &ThreadCtx) -> Self {
        let mut out = Self {
            worker_poller_base: WorkerPollerBase::new(ctx),
            ..Default::default()
        }
        if cfg!(target_feature = "poller_use_epoll_cloexec") {
           out._epoll_fd = epoll_create(EPOLL_CLOEXEC) ;
        } else {
            out._epoll_fd = epoll_create(1);
        }
        out
    }

    // ~ZmqEpoll () ;
    //  "poller" concept.
    // handle_t add_fd (fd: ZmqFileDesc, i_poll_events *events_);
    // void rm_fd (handle_t handle_);
    // void set_pollin (handle_t handle_);
    // void reset_pollin (handle_t handle_);
    // void set_pollout (handle_t handle_);
    // void reset_pollout (handle_t handle_);
    // void stop ();
    // void loop () ;
    // static int max_fds ();

    pub fn add_fd (&mut self, fd: ZmqFileDesc, events: &mut [i_poll_events]) -> ZmqPollEntry
    {
        self.check_thread();
        // ZmqPollEntry *pe = new (std::nothrow) ZmqPollEntry;
        let mut pe: ZmqPollEntry = ZmqPollEntry::default();
        // alloc_assert (pe);

        //  The memset is not actually needed. It's here to prevent debugging
        //  tools to complain about using uninitialised memory.
        // memset (pe, 0, mem::size_of::<ZmqPollEntry>());

        pe.fd = fd;
        pe.ev.events = 0;
        pe.ev.data.ptr = pe.clone();
        pe.events = events_;

        let rc: i32 = epoll_ctl(_epoll_fd, EPOLL_CTL_ADD, fd, &pe.ev);
        // errno_assert (rc != -1);

        //  Increase the load metric of the thread.
        adjust_load (1);

        return pe;
    }

    pub fn rm_fd (&mut self , handle_: ZmqPollEntry)
    {
    check_thread ();
    // ZmqPollEntry *pe = static_cast<ZmqPollEntry *> (handle_);
    let rc: i32 = epoll_ctl (_epoll_fd, EPOLL_CTL_DEL, pe.fd, &pe.ev);
    errno_assert (rc != -1);
    pe.fd = retired_fd;
    _retired.push_back (pe);

    //  Decrease the load metric of the thread.
    adjust_load (-1);
    }

    void ZmqEpoll::set_pollin (handle_t handle_)
    {
    check_thread ();
    ZmqPollEntry *pe = static_cast<ZmqPollEntry *> (handle_);
    pe.ev.events |= EPOLLIN;
    let rc: i32 = epoll_ctl (_epoll_fd, EPOLL_CTL_MOD, pe.fd, &pe.ev);
    errno_assert (rc != -1);
    }

    void ZmqEpoll::reset_pollin (handle_t handle_)
    {
    check_thread ();
    ZmqPollEntry *pe = static_cast<ZmqPollEntry *> (handle_);
    pe.ev.events &= ~(static_cast<u32> (EPOLLIN));
    let rc: i32 = epoll_ctl (_epoll_fd, EPOLL_CTL_MOD, pe.fd, &pe.ev);
    errno_assert (rc != -1);
    }

    void ZmqEpoll::set_pollout (handle_t handle_)
    {
    check_thread ();
    ZmqPollEntry *pe = static_cast<ZmqPollEntry *> (handle_);
    pe.ev.events |= EPOLLOUT;
    let rc: i32 = epoll_ctl (_epoll_fd, EPOLL_CTL_MOD, pe.fd, &pe.ev);
    errno_assert (rc != -1);
    }

    void ZmqEpoll::reset_pollout (handle_t handle_)
    {
    check_thread ();
    ZmqPollEntry *pe = static_cast<ZmqPollEntry *> (handle_);
    pe.ev.events &= ~(static_cast<u32> (EPOLLOUT));
    let rc: i32 = epoll_ctl (_epoll_fd, EPOLL_CTL_MOD, pe.fd, &pe.ev);
    errno_assert (rc != -1);
    }

    void ZmqEpoll::stop ()
    {
    check_thread ();
    }

    int ZmqEpoll::max_fds ()
    {
    return -1;
    }

    void ZmqEpoll::loop ()
    {
    epoll_event ev_buf[max_io_events];

    while (true) {
    //  Execute any due timers.
    let timeout: i32 = static_cast<int> (execute_timers ());

    if (get_load () == 0) {
    if (timeout == 0)
    break;

    // TODO sleep for timeout
    continue;
    }

    //  Wait for events.
    let n: i32 = epoll_wait (_epoll_fd, &ev_buf[0], max_io_events,
    timeout ? timeout : -1);
    if (n == -1) {
    errno_assert (errno == EINTR);
    continue;
    }

    for (int i = 0; i < n; i+= 1) {
    const ZmqPollEntry *const pe =
    static_cast<const ZmqPollEntry *> (ev_buf[i].data.ptr);

    if (null_mut() == pe)
    continue;
    if (null_mut() == pe.events)
    continue;
    if (pe.fd == retired_fd)
    continue;
    if (ev_buf[i].events & (EPOLLERR | EPOLLHUP))
    pe.events.in_event ();
    if (pe.fd == retired_fd)
    continue;
    if (ev_buf[i].events & EPOLLOUT)
    pe.events.out_event ();
    if (pe.fd == retired_fd)
    continue;
    if (ev_buf[i].events & EPOLLIN)
    pe.events.in_event ();
    }

    //  Destroy retired event sources.
    for (retired_t::iterator it = _retired.begin (), end = _retired.end ();
    it != end; += 1it) {
    LIBZMQ_DELETE (*it);
    }
    _retired.clear ();
    }
    }

}

// typedef ZmqEpoll Poller;
pub type Poller = ZmqEpoll;

// #ifdef ZMQ_HAVE_WINDOWS
// const ZmqEpoll::epoll_fd_t ZmqEpoll::epoll_retired_fd =
//   INVALID_HANDLE_VALUE;
// #endif

// ZmqEpoll::ZmqEpoll (const ThreadCtx &ctx) :
//     WorkerPollerBase (ctx)
// {
// // #ifdef ZMQ_IOTHREAD_POLLER_USE_EPOLL_CLOEXEC
//     //  Setting this option result in sane behaviour when exec() functions
//     //  are used. Old sockets are closed and don't block TCP ports, avoid
//     //  leaks, etc.
//     _epoll_fd = epoll_create1 (EPOLL_CLOEXEC);
// // #else
//     _epoll_fd = epoll_create (1);
// // #endif
//     errno_assert (_epoll_fd != epoll_retired_fd);
// }

// ZmqEpoll::~ZmqEpoll ()
// {
//     //  Wait till the worker thread exits.
//     stop_worker ();
//
// // #ifdef ZMQ_HAVE_WINDOWS
//     epoll_close (_epoll_fd);
// // #else
//     close (_epoll_fd);
// // #endif
//     for (retired_t::iterator it = _retired.begin (), end = _retired.end ();
//          it != end; += 1it) {
//         LIBZMQ_DELETE (*it);
//     }
// }

// #endif
