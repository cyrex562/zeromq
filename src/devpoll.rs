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

use std::ffi::CString;
use libc::{EINTR, O_RDWR, open, write};
use windows::Win32::Networking::WinSock::{POLLERR, POLLHUP, POLLIN, POLLOUT};
use crate::context::ZmqContext;
use crate::defines::ZmqHandle;
use crate::fd::ZmqFileDesc;
use crate::poller_base::WorkerPollerBase;
use crate::thread_ctx::ThreadCtx;

// typedef DevPoll Poller;
pub type Poller = DevPoll;

pub struct FdEntry {
    // short events;
    pub events: i16,
    // i_poll_events *reactor;
    pub reactor: i_poll_events,
    // valid: bool
    pub valid: bool,
    // accepted: bool
    pub accepted: bool,
}

#[derive(Clone, Debug, Default)]
pub struct DevPoll {
    //
    //  File descriptor referring to "/dev/poll" pseudo-device.
    // ZmqFileDesc devpoll_fd;
    pub devpoll_fd: ZmqFileDesc,
    // fd_table_t fd_table;
    pub fd_table: Vec<FdEntry>,
    // pending_list_t pending_list;
    pub pending_list: Vec<ZmqFileDesc>,
    //  Pollset manipulation function.
    // // ZMQ_NON_COPYABLE_NOR_MOVABLE (DevPoll)
    pub worker_poller_base: WorkerPollerBase,
}

impl DevPoll {
    // DevPoll (const ThreadCtx &ctx);
    // DevPoll::DevPoll (const ThreadCtx &ctx) :
    // WorkerPollerBase (ctx)
    pub fn new(ctx: &mut ZmqContext) -> Self {
        unsafe { devpoll_fd = open(CString::from(String::from("/dev/poll")).into_raw(), O_RDWR); }
        // errno_assert (devpoll_fd != -1);
        Self {
            worker_poller_base: WorkerPollerBase::new(ctx),
            ..Default::default()
        }
    }
    // ~DevPoll () ;
    // DevPoll::~DevPoll ()
    // {
    //     //  Wait till the worker thread exits.
    //     stop_worker ();

    //     close (devpoll_fd);
    // }

    //  "poller" concept.
    // handle_t add_fd (fd: ZmqFileDesc, i_poll_events *events_);
    pub fn add_fd(&mut self, fd: &ZmqHandle, reactor_: &mut i_poll_events) -> ZmqHandle {
        check_thread();
        //  If the file descriptor table is too small expand it.
        let sz = self.fd_table.size();
        if (sz <= fd) {
            self.fd_table.resize(fd + 1, FdEntry::default());
            while (sz != (fd + 1)) {
                self.fd_table[sz].valid = false;
                sz += 1;
            }
        }

        // zmq_assert (!fd_table[fd].valid);

        self.fd_table[fd].events = 0;
        self.fd_table[fd].reactor = reactor_;
        self.fd_table[fd].valid = true;
        self.fd_table[fd].accepted = false;

        self.devpoll_ctl(fd, 0);
        self.pending_list.push_back(fd);

        //  Increase the load metric of the thread.
        self.adjust_load(1);

        return fd.clone();
    }

    // void rm_fd (handle_t handle_);
    pub fn rm_fd(&mut self, handle_: &ZmqHandle) {
        check_thread();
        // zmq_assert (fd_table[handle_].valid);

        self.devpoll_ctl(handle_, POLLREMOVE);
        self.fd_table[handle_].valid = false;

        //  Decrease the load metric of the thread.
        self.adjust_load(-1);
    }

    // void set_pollin (handle_t handle_);
    pub fn set_pollin(&mut self, handle_: &ZmqHandle) {
        check_thread();
        self.devpoll_ctl(handle_, POLLREMOVE);
        self.fd_table[handle_].events |= POLLIN;
        self.devpoll_ctl(handle_, self.fd_table[handle_].events);
    }

    // void reset_pollin (handle_t handle_);
    pub fn reset_pollin(&mut self, handle_: &ZmqHandle) {
        check_thread();
        self.devpoll_ctl(handle_, POLLREMOVE);
        self.fd_table[handle_].events &= !(POLLIN);
        self.devpoll_ctl(handle_, fd_table[handle_].events);
    }

    // void set_pollout (handle_t handle_);
    pub fn set_pollout(&mut self, handle_: &ZmqHandle) {
        check_thread();
        self.devpoll_ctl(handle_, POLLREMOVE);
        self.fd_table[handle_].events |= POLLOUT;
        self.devpoll_ctl(handle_, self.fd_table[handle_].events);
    }

    // void reset_pollout (handle_t handle_);
    pub fn reset_pollout(&mut self, handle_: &ZmqHandle) {
        check_thread();
        self.devpoll_ctl(handle_, POLLREMOVE);
        self.fd_table[handle_].events &= !(POLLOUT);
        self.devpoll_ctl(handle_, self.fd_table[handle_].events);
    }

    // void stop ();
    pub fn stop(&mut self) {
        check_thread();
    }

    // static int max_fds ();
    pub fn max_fds(&mut self) -> i32 {
        return -1;
    }

    //
    //  Main event loop.

    // void loop () ;

    // void devpoll_ctl (fd: ZmqFileDesc, short events_);
    pub fn devpoll_ctl(&mut self, fd: &ZmqHandle, events_: i16) {
        // let mut pfd = PollFd{fd, events_, 0};
        let rc = unsafe { write(devpoll_fd, &pfd, pfd.len()) };
        // zmq_assert (rc == pfd.len());
    }
    pub fn loop_fn(&mut self) {
        loop {
            let mut ev_buf: [pollfd; max_io_events] = [pollfd::default(); max_io_events];
            let mut poll_req: dvpoll;

            // for (pending_list_t::size_type i = 0; i < pending_list.size (); i+= 1)
            for i in 0..self.pending_list.len() {
                fd_table[pending_list[i]].accepted = true;
            }
            self.pending_list.clear();

            //  Execute any due timers.
            let timeout = execute_timers();

            if (self.get_load() == 0) {
                if (timeout == 0) {
                    break;
                }

                // TODO sleep for timeout
                continue;
            }

            //  Wait for events.
            //  On Solaris, we can retrieve no more then (OPEN_MAX - 1) events.
            poll_req.dp_fds = &ev_buf[0];
            // #if defined ZMQ_HAVE_SOLARIS
            poll_req.dp_nfds = i32::min(max_io_events, OPEN_MAX - 1);
            // #else
            poll_req.dp_nfds = max_io_events;
            // #endif
            poll_req.dp_timeout = if timeout { timeout } else { -1 };
            // TODO
            // let n = ioctl (devpoll_fd, DP_POLL, &poll_req);
            if (n == -1 && errno == EINTR) {
                continue;
            }
            // errno_assert (n != -1);

            // for (int i = 0; i < n; i+= 1)
            for i in 0..n {
                FdEntry * fd_ptr = &fd_table[ev_buf[i].fd];
                if (!fd_ptr.valid || !fd_ptr.accepted) {
                    continue;
                }
                if (ev_buf[i].revents & (POLLERR | POLLHUP)) {
                    fd_ptr.reactor.in_event();
                }
                if (!fd_ptr.valid || !fd_ptr.accepted) {
                    continue;
                }
                if (ev_buf[i].revents & POLLOUT) {
                    fd_ptr.reactor.out_event();
                }
                if (!fd_ptr.valid || !fd_ptr.accepted) {
                    continue;
                }
                if (ev_buf[i].revents & POLLIN) {
                    fd_ptr.reactor.in_event();
                }
            }
        }
    }
}
