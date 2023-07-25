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
// #include "poll.hpp"
// #if defined ZMQ_IOTHREAD_POLLER_USE_POLL

// #include <sys/types.h>
// #include <sys/time.h>
// #include <poll.h>
// #include <algorithm>

// #include "poll.hpp"
// #include "err.hpp"
// #include "config.hpp"
// #include "i_poll_events.hpp"
//  typedef ZmqFileDesc handle_t;

use crate::defines::{ZmqFileDesc, RETIRED_FD};
use crate::poller_base::WorkerPollerBase;
use crate::poller_event::ZmqPollerEvent;
use crate::pollset::ZmqPollEvents;
use crate::thread_context::ZmqThreadContext;
#[cfg(target_os = "linux")]
use libc::{poll, pollfd, POLLERR, POLLHUP, POLLIN, POLLOUT};
use std::thread::Thread;

// struct pollfd
//   {
//     int fd;                        /* File descriptor to poll.  */
//     short int events;                /* Types of events poller cares about.  */
//     short int revents;                /* Types of events that actually occurred.  */
//   };
#[cfg(target_os = "windows")]
pub struct pollfd {
    pub fd: ZmqFileDesc,
    pub events: i16,
    pub revents: i16,
}

#[derive(Default, Debug, Clone)]
struct FdEntry {
    pub index: ZmqFileDesc,
    pub events: Vec<ZmqPollerEvent>,
}

#[derive(Default, Debug, Clone)]
pub struct ZmqPoll<'a> {
    // : public WorkerPollerBase
    pub base: WorkerPollerBase<'a>,
    //  This table stores data for registered descriptors.
    // typedef std::vector<FdEntry> fd_table_t;
    // fd_table_t fd_table;
    pub fd_table: Vec<FdEntry>,
    //  Pollset to pass to the poll function.
    // typedef std::vector<pollfd> PollSet;
    // PollSet pollset;
    pub pollset: Vec<pollfd>,
    //  If true, there's at least one retired event source.
    retired: bool,
    // ZMQ_NON_COPYABLE_NOR_MOVABLE (poll_t)
}

impl ZmqPoll {
    // poll_t (const ThreadCtx &ctx);
    // ~poll_t ();
    //  "poller" concept.
    //  These methods may only be called from an event callback; add_fd may also be called before start.
    // handle_t add_fd (fd: ZmqFileDesc, ZmqPollEventsInterface *events_);
    // void rm_fd (handle_t handle_);
    // void set_pollin (handle_t handle_);
    // void reset_pollin (handle_t handle_);
    // void set_pollout (handle_t handle_);
    // void reset_pollout (handle_t handle_);
    // void Stop ();
    // static int max_fds ();
    //  Main event loop.
    // void loop () ;
    // void cleanup_retired ();

    pub fn new(ctx: &mut ZmqThreadContext) -> Self {
        Self {
            base: WorkerPollerBase::new(ctx),
            fd_table: vec![],
            pollset: vec![],
            retired: false,
        }
    }

    pub fn add_fd(&mut self, fd: ZmqFileDesc, events_: &mut [ZmqPollEvents]) -> ZmqFileDesc {
        self.base.check_thread();
        // zmq_assert (fd != retired_fd);

        //  If the file descriptor table is too small expand it.
        let sz = self.fd_table.size();
        if (sz <= fd) {
            // self.fd_table.resize(fd + 1);
            // while sz != (fd_table_t::size_type)(fd + 1) {
            //     self.fd_table[sz].index = retired_fd;
            //     // += 1sz;
            //     sz += 1;
            // }
            self.fd_table.resize((fd + 1) as usize, FdEntry::default());
        }

        let pfd = pollfd {
            fd,
            events: 0,
            revents: 0,
        };
        self.pollset.push_back(pfd);
        // zmq_assert (fd_table[fd].index == retired_fd);

        self.fd_table[fd].index = self.pollset.size() - 1;
        self.fd_table[fd].events = events_;

        //  Increase the load metric of the thread.
        self.adjust_load(1);

        return fd;
    }

    pub fn rm_fd(&mut self, handle_: ZmqFileDesc) {
        self.base.check_thread();
        let mut index: ZmqFileDesc = self.fd_table[handle_].index;
        // zmq_assert (index != retired_fd);

        //  Mark the fd as unused.
        self.pollset[index].fd = RETIRED_FD;
        self.fd_table[handle_].index = RETIRED_FD;
        self.retired = true;

        //  Decrease the load metric of the thread.
        self.adjust_load(-1);
    }

    pub fn set_pollin(&mut self, handle_: ZmqFileDesc) {
        self.base.check_thread();
        let mut index: ZmqFileDesc = self.fd_table[handle_].index;
        self.pollset[index].events |= POLLIN;
    }

    pub fn reset_pollin(&mut self, handle_: ZmqFileDesc) {
        self.base.check_thread();
        let mut index: ZmqFileDesc = self.fd_table[handle_].index;
        self.pollset[index].events &= !(POLLIN);
    }

    pub fn set_pollout(&mut self, handle_: ZmqFileDesc) {
        self.base.check_thread();
        let mut index: ZmqFileDesc = self.fd_table[handle_].index;
        self.pollset[index].events |= POLLOUT;
    }

    pub fn reset_pollout(&mut self, handle_: ZmqFileDesc) {
        self.base.check_thread();
        let mut index: ZmqFileDesc = self.fd_table[handle_].index;
        self.pollset[index].events &= !(POLLOUT);
    }

    pub fn stop(&mut self) {
        self.base.check_thread();
        //  no-op... thread is stopped when no more fds or timers are registered
    }

    pub fn max_fds(&mut self) -> i32 {
        return -1;
    }

    pub fn loop_(&mut self) {
        loop {
            //  Execute any due timers.
            let timeout = self.base.execute_timers();

            self.cleanup_retired();

            if self.pollset.empty() {
                // zmq_assert (get_load () == 0);

                if timeout == 0 {
                    break;
                }

                // TODO sleep for timeout
                continue;
            }

            //  Wait for events.
            let mut rc = 0i32;
            unsafe {
                rc = poll(
                    &mut self.pollset[0],
                    (self.pollset.size()),
                    if timeout { timeout } else { -1 },
                );
            }
            if (rc == -1) {
                // errno_assert (errno == EINTR);
                continue;
            }

            //  If there are no events (i.e. it's a timeout) there's no point
            //  in checking the pollset.
            if (rc == 0) {
                continue;
            }

            // for (PollSet::size_type i = 0; i != pollset.size (); i+= 1)
            for i in 0..self.pollset.len() {
                // zmq_assert (!(pollset[i].revents & POLLNVAL));
                if (self.pollset[i].fd == RETIRED_FD) {
                    continue;
                }
                if (self.pollset[i].revents & (POLLERR | POLLHUP)) {
                    self.fd_table[self.pollset[i].fd].events.in_event();
                }
                if (self.pollset[i].fd == RETIRED_FD) {
                    continue;
                }
                if (self.pollset[i].revents & POLLOUT) {
                    self.fd_table[self.pollset[i].fd].events.out_event();
                }
                if (self.pollset[i].fd == RETIRED_FD) {
                    continue;
                }
                if (self.pollset[i].revents & POLLIN) {
                    self.fd_table[self.pollset[i].fd].events.in_event();
                }
            }
        }
    }

    pub fn cleanup_retired(&mut self) {
        //  Clean up the pollset and update the fd_table accordingly.
        if (self.retired) {
            let mut i = 0;
            while (i < self.pollset.size()) {
                if (self.pollset[i].fd == RETIRED_FD) {
                    self.pollset.erase(self.pollset.begin() + i);
                } else {
                    self.fd_table[self.pollset[i].fd].index = i;
                    i += 1;
                }
            }
            self.retired = false;
        }
    }
}

// typedef poll_t Poller;

// #endif
