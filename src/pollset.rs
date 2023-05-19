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

use libc::{epoll_ctl, POLLERR, POLLHUP, POLLIN, POLLOUT};
use crate::fd::ZmqFileDesc;
use crate::poll_events_interface::ZmqPollEventsInterface;
use crate::poller_base::PollerBase;
use crate::thread_ctx::ThreadCtx;
use crate::zmtp_engine::ZmqIoThread;

struct ZmqPollEntry {
    // ZmqFileDesc fd;
    pub fd: ZmqFileDesc,
    pub flag_pollin: bool,
    pub flag_pollout: bool,
    // ZmqPollEventsInterface *events;
    pub events: ZmqPollEventsInterface,
}

pub struct PollSet<'a> {
    // : public PollerBase
    pub poller_base: PollerBase,
    //
//     typedef void *handle_t;
//     PollSet (const ThreadCtx &ctx);
//     ~PollSet () ;
    //  "poller" concept.
    // handle_t add_fd (fd: ZmqFileDesc, ZmqPollEventsInterface *events_);
    // void rm_fd (handle_t handle_);
    // void set_pollin (handle_t handle_);
    // void reset_pollin (handle_t handle_);
    // void set_pollout (handle_t handle_);
    // void reset_pollout (handle_t handle_);
    // void start ();
    // void stop ();
    // static int max_fds ();
    //  Main worker thread routine.
    // static void worker_routine (arg_: &mut [u8]);
    //  Main event loop.
    // void loop () ;
    // Reference to ZMQ context.
    // const ThreadCtx &ctx;
    pub ctx: &'a ThreadCtx,
    //  Main pollset file descriptor
    pub pollset_fd: ZmqFileDesc,
    //  List of retired event sources.
    // typedef std::vector<ZmqPollEntry *> retired_t;
    // retired_t retired;
    pub retired: Vec<ZmqPollEntry>,
    //  This table stores data for registered descriptors.
    // typedef std::vector<ZmqPollEntry *> fd_table_t;
    // fd_table_t fd_table;
    pub fd_table: Vec<ZmqPollEntry>,
    //  If true, thread is in the process of shutting down.
    pub stopping: bool,
    //  Handle of the physical thread doing the I/O work.
    // thread_t worker;
    pub worker: ZmqIoThread,

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (PollSet)
}

impl PollSet {
    pub fn new(ctx: &ThreadCtx) -> Self {
        Self {
            poller_base: PollerBase::new(),
            pollset_fd: pollset_create(-1),
            retired: Vec::new(),
            fd_table: Vec::new(),
            worker: ZmqIoThread::new(),
            ctx: ctx,
            stopping: false,
        }
    }

    pub fn add_fd(&mut self, fd: ZmqFileDesc,
                  events_: &mut ZmqPollEventsInterface) -> ZmqFileDesc {
        let pe = ZmqPollEntry::default();
        // alloc_assert (pe);

        pe.fd = fd;
        pe.flag_pollin = false;
        pe.flag_pollout = false;
        pe.events = events_;

        // TODO
        // let pc = epoll_ctl(pollset_fd, EPOLL_CTL_ADD, fd, &mut pe.events);
        // pc.fd = fd;
        // pc.cmd = PS_ADD;
        // pc.events = 0;

        let rc = pollset_ctl(pollset_fd, &pc, 1);
        // errno_assert (rc != -1);

        //  Increase the load metric of the thread.
        adjust_load(1);

        // if fd >= fd_table.size () {
        //     fd_table.resize (fd + 1, null_mut());
        // }
        // fd_table[fd] = pe;
        return pe;
    }

    pub fn rm_fd(&mut self, handle_: ZmqFileDesc) {
        // ZmqPollEntry *pe = (ZmqPollEntry *) handle_;
        // TODO
        // struct poll_ctl pc;
        // pc.fd = pe.fd;
        // pc.cmd = PS_DELETE;
        // pc.events = 0;
        pollset_ctl(pollset_fd, &pc, 1);

        fd_table[pe.fd] = null_mut();

        pe.fd = retired_fd;
        retired.push_back(pe);

        //  Decrease the load metric of the thread.
        adjust_load(-1);
    }

    pub fn set_pollin(&mut self, handle_: ZmqFileDesc) {
        // TODO
        // ZmqPollEntry *pe = (ZmqPollEntry *) handle_;
        // if ( (!pe.flag_pollin)) {
        //     struct poll_ctl pc;
        //     pc.fd = pe.fd;
        //     pc.cmd = PS_MOD;
        //     pc.events = POLLIN;
        //
        //     let rc: i32 = pollset_ctl (pollset_fd, &pc, 1);
        //     // errno_assert (rc != -1);
        //
        //     pe.flag_pollin = true;
        // }
    }

    pub fn reset_pollin(&mut self, handle_: ZmqFileDesc) {
        // ZmqPollEntry *pe = (ZmqPollEntry *) handle_;
        // if ( (!pe.flag_pollin)) {
        //     return;
        // }

        // struct poll_ctl pc;
        // pc.fd = pe.fd;
        // pc.events = 0;
        //
        // pc.cmd = PS_DELETE;
        // let rc = pollset_ctl (pollset_fd, &pc, 1);
        //
        // if (pe.flag_pollout) {
        //     pc.events = POLLOUT;
        //     pc.cmd = PS_MOD;
        //     rc = pollset_ctl (pollset_fd, &pc, 1);
        //     // errno_assert (rc != -1);
        // }
        //
        // pe.flag_pollin = false;
    }

    pub fn set_pollout(&mut self, handle_: ZmqFileDesc) {
        // ZmqPollEntry *pe = (ZmqPollEntry *) handle_;
        // if ( (!pe.flag_pollout)) {
        //     struct poll_ctl pc;
        //     pc.fd = pe.fd;
        //     pc.cmd = PS_MOD;
        //     pc.events = POLLOUT;
        //
        //     let rc: i32 = pollset_ctl (pollset_fd, &pc, 1);
        //     // errno_assert (rc != -1);
        //
        //     pe.flag_pollout = true;
        // }
    }

    pub fn reset_pollout(&mut self, handle_: ZmqFileDesc) {
        // ZmqPollEntry *pe = (ZmqPollEntry *) handle_;
        // if ( (!pe.flag_pollout)) {
        //     return;
        // }
        //
        // struct poll_ctl pc;
        // pc.fd = pe.fd;
        // pc.events = 0;
        //
        // pc.cmd = PS_DELETE;
        // int rc = pollset_ctl (pollset_fd, &pc, 1);
        // // errno_assert (rc != -1);
        //
        // if (pe.flag_pollin) {
        //     pc.cmd = PS_MOD;
        //     pc.events = POLLIN;
        //     rc = pollset_ctl (pollset_fd, &pc, 1);
        //     // errno_assert (rc != -1);
        // }
        // pe.flag_pollout = false;
    }

    pub fn start(&mut self) {
        ctx.start_thread(worker, worker_routine, this);
    }

    pub fn stop(&mut self) {
        stopping = true;
    }

    pub fn max_fds(&mut self) -> i32 {
        return -1;
    }


    pub fn loop_(&mut self) {
        // struct pollfd
        // polldata_array[max_io_events];

        while !stopping {
            //  Execute any due timers.
            let timeout = execute_timers();

            //  Wait for events.
            let n = pollset_poll(pollset_fd, polldata_array, max_io_events,
                                 if timeout { timeout } else { -1 });
            if (n == -1) {
                // errno_assert (errno == EINTR);
                continue;
            }

            // for (int i = 0; i < n; i+= 1)
            for i in 0..n {
                // ZmqPollEntry *pe = fd_table[polldata_array[i].fd];
                if (!pe) {
                    continue;
                }

                if (pe.fd == retired_fd) {
                    continue;
                }
                if (polldata_array[i].revents & (POLLERR | POLLHUP)) {
                    pe.events.in_event();
                }
                if (pe.fd == retired_fd) {
                    continue;
                }
                if (polldata_array[i].revents & POLLOUT) {
                    pe.events.out_event();
                }
                if (pe.fd == retired_fd) {
                    continue;
                }
                if (polldata_array[i].revents & POLLIN) {
                    pe.events.in_event();
                }
            }

            //  Destroy retired event sources.
            // for (retired_t::iterator it = retired.begin (); it != retired.end ();
            // += 1it)
            // LIBZMQ_DELETE (*it);
            for retired in self.retired {
                LIBZMQ_DELETE(retired);
            }
            retired.clear();
        }
    }

    pub fn worker_routine(&mut self, arg_: &mut [u8]) {
        // ((PollSet *) arg_)->loop ();
    }
}

// PollSet::PollSet (const ThreadCtx &ctx) :
//     ctx (ctx), stopping (false)
// {
//     pollset_fd = pollset_create (-1);
//     // errno_assert (pollset_fd != -1);
// }

// PollSet::~PollSet ()
// {
//     //  Wait till the worker thread exits.
//     worker.stop ();
//
//     pollset_destroy (pollset_fd);
//     for (retired_t::iterator it = retired.begin (); it != retired.end (); += 1it)
//         LIBZMQ_DELETE (*it);
// }


// #endif
