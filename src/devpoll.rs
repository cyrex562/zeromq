use std::ffi::CString;

#[cfg(target_os = "windows")]
use libc::{open, O_RDWR};

pub const POLLREMOVE: i16 = 0x2000;

#[cfg(target_os = "linux")]
use libc::{open, pollfd, write, EINTR, O_RDWR};

use windows::Win32::Networking::WinSock::{POLLERR, POLLHUP, POLLIN, POLLOUT};

use crate::context::ZmqContext;
use crate::defines::ZmqFileDesc;
use crate::defines::ZmqHandle;
use crate::events::ZmqEvents;
use crate::poller_base::WorkerPollerBase;

// typedef DevPoll Poller;
pub type ZmqPoller<'a> = DevPoll<'a>;

pub struct FdEntry {
    // short events;
    pub events: i16,
    // i_poll_events *reactor;
    pub reactor: ZmqEvents,
    // valid: bool
    pub valid: bool,
    // accepted: bool
    pub accepted: bool,
}

#[derive(Clone, Debug, Default)]
pub struct DevPoll<'a> {
    //  File descriptor referring to "/dev/poll" pseudo-device.
    pub devpoll_fd: ZmqFileDesc,
    pub fd_table: Vec<FdEntry>,
    pub pending_list: Vec<ZmqFileDesc>,
    pub base: WorkerPollerBase<'a>,
}

impl<'a> DevPoll<'a> {
    pub fn new(ctx: &mut ZmqContext) -> Self {
        let devpoll_fd =
            unsafe { open(CString::from(String::from("/dev/poll")).into_raw(), O_RDWR) };
        // errno_assert (devpoll_fd != -1);
        Self {
            base: WorkerPollerBase::new(&mut ctx.thread_ctx),
            devpoll_fd,
            ..Default::default()
        }
    }

    pub fn add_fd(&mut self, fd: &ZmqHandle, reactor_: &mut ZmqEvents) -> ZmqHandle {
        self.base.check_thread();
        //  If the file descriptor table is too small expand it.
        let sz = self.fd_table.size();
        // if (sz <= fd) {
        //     self.fd_table.resize(fd + 1, FdEntry::default());
        //     while sz != (fd + 1) {
        //         self.fd_table[sz].valid = false;
        //         sz += 1;
        //     }
        // }

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

    pub fn rm_fd(&mut self, handle_: &ZmqHandle) {
        self.base.check_thread();
        // zmq_assert (fd_table[handle_].valid);

        self.devpoll_ctl(handle_, POLLREMOVE);
        self.fd_table[handle_].valid = false;

        //  Decrease the load metric of the thread.
        self.adjust_load(-1);
    }

    pub fn set_pollin(&mut self, handle_: &ZmqHandle) {
        self.base.check_thread();
        self.devpoll_ctl(handle_, POLLREMOVE);
        self.fd_table[handle_].events |= POLLIN;
        self.devpoll_ctl(handle_, self.fd_table[handle_].events);
    }

    pub fn reset_pollin(&mut self, handle_: &ZmqHandle) {
        self.base.check_thread();
        self.devpoll_ctl(handle_, POLLREMOVE);
        self.fd_table[handle_].events &= !(POLLIN);
        self.devpoll_ctl(handle_, self.fd_table[handle_].events);
    }

    pub fn set_pollout(&mut self, handle_: &ZmqHandle) {
        self.base.check_thread();
        self.devpoll_ctl(handle_, POLLREMOVE);
        self.fd_table[handle_].events |= POLLOUT;
        self.devpoll_ctl(handle_, self.fd_table[handle_].events);
    }

    // void reset_pollout (handle_t handle_);
    pub fn reset_pollout(&mut self, handle_: &ZmqHandle) {
        self.base.check_thread();
        self.devpoll_ctl(handle_, POLLREMOVE);
        self.fd_table[handle_].events &= !(POLLOUT);
        self.devpoll_ctl(handle_, self.fd_table[handle_].events);
    }

    // void Stop ();
    pub fn stop(&mut self) {
        self.base.check_thread();
    }

    // static int max_fds ();
    pub fn max_fds(&mut self) -> i32 {
        return -1;
    }

    //
    //  Main event loop.
    pub fn devpoll_ctl(&mut self, fd: &ZmqHandle, events_: i16) {
        // TODO: implement for linux
        // // let mut pfd = PollFd{fd, events_, 0};
        // let rc = unsafe { write(self.devpoll_fd, &pfd, pfd.len()) };
        // // zmq_assert (rc == pfd.len());
    }

    pub fn loop_fn(&mut self) {
        loop {
            // TODO: implement for linux
            // let mut ev_buf: [pollfd; max_io_events] = [pollfd::default(); max_io_events];
            // let mut poll_req: devpoll;

            // // for (pending_list_t::size_type i = 0; i < pending_list.size (); i+= 1)
            // for i in 0..self.pending_list.len() {
            //     self.fd_table[pending_list[i]].accepted = true;
            // }
            // self.pending_list.clear();

            // //  Execute any due timers.
            // let timeout = execute_timers();

            // if self.get_load() == 0 {
            //     if timeout == 0 {
            //         break;
            //     }

            //     // TODO sleep for timeout
            //     continue;
            // }

            // //  Wait for events.
            // //  On Solaris, we can retrieve no more then (OPEN_MAX - 1) events.
            // poll_req.dp_fds = &ev_buf[0];
            // // #if defined ZMQ_HAVE_SOLARIS
            // poll_req.dp_nfds = i32::min(max_io_events, OPEN_MAX - 1);
            // // #else
            // poll_req.dp_nfds = max_io_events;
            // // #endif
            // poll_req.dp_timeout = if timeout { timeout } else { -1 };
            // // TODO
            // // let n = ioctl (devpoll_fd, DP_POLL, &poll_req);
            // if n == -1 && errno == EINTR {
            //     continue;
            // }
            // // errno_assert (n != -1);

            // // for (int i = 0; i < n; i+= 1)
            // for i in 0..n {
            //     FdEntry * fd_ptr = &fd_table[ev_buf[i].fd];
            //     if !fd_ptr.valid || !fd_ptr.accepted {
            //         continue;
            //     }
            //     if ev_buf[i].revents & (POLLERR | POLLHUP) {
            //         fd_ptr.reactor.in_event();
            //     }
            //     if !fd_ptr.valid || !fd_ptr.accepted {
            //         continue;
            //     }
            //     if ev_buf[i].revents & POLLOUT {
            //         fd_ptr.reactor.out_event();
            //     }
            //     if !fd_ptr.valid || !fd_ptr.accepted {
            //         continue;
            //     }
            //     if ev_buf[i].revents & POLLIN {
            //         fd_ptr.reactor.in_event();
            //     }
            // }
        }
    }
}
