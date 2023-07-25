use crate::defines::{ZmqFileDesc, ZmqHandle};
use crate::poller_base::PollerBase;
use crate::thread_context::ZmqThreadContext;
use libc::{epoll_ctl, POLLERR, POLLHUP, POLLIN, POLLOUT};

struct ZmqPollEntry {
    pub fd: ZmqFileDesc,
    pub flag_pollin: bool,
    pub flag_pollout: bool,
    pub events: ZmqPollEvents,
}

#[derive(Default, Debug, Clone)]
pub struct ZmqPollEvents {}

#[derive(Default, Debug, Clone)]
pub struct PollSet<'a> {
    pub poller_base: PollerBase,
    pub ctx: &'a ZmqThreadContext,
    //  Main pollset file descriptor
    pub pollset_fd: ZmqFileDesc,
    //  List of retired event sources.
    pub retired: Vec<ZmqPollEntry>,
    //  This table stores data for registered descriptors.
    // typedef std::vector<ZmqPollEntry *> fd_table_t;
    // fd_table_t fd_table;
    pub fd_table: Vec<ZmqPollEntry>,
    //  If true, thread is in the process of shutting down.
    pub stopping: bool,
    //  Handle of the physical thread doing the I/O work.
    // ZmqThread worker;
    pub worker: ZmqHandle,
    // ZMQ_NON_COPYABLE_NOR_MOVABLE (PollSet)
}

impl PollSet {
    pub fn new(ctx: &ZmqThreadContext) -> Self {
        Self {
            poller_base: PollerBase::new(),
            // pollset_fd: pollset_create(-1),
            retired: Vec::new(),
            fd_table: Vec::new(),
            worker: 0,
            ctx: ctx,
            stopping: false,
            ..Default::default()
        }
    }

    pub fn add_fd(&mut self, fd: ZmqFileDesc, events_: &mut ZmqPollEvents) -> ZmqFileDesc {
        let pe = ZmqPollEntry::default();

        pe.fd = fd;
        pe.flag_pollin = false;
        pe.flag_pollout = false;
        pe.events = events_;

        // TODO
        // let pc = epoll_ctl(pollset_fd, EPOLL_CTL_ADD, fd, &mut pe.events);
        // pc.fd = fd;
        // pc.cmd = PS_ADD;
        // pc.events = 0;

        // TODO
        // let rc = pollset_ctl(pollset_fd, &pc, 1);
        // errno_assert (rc != -1);

        //  Increase the load metric of the thread.
        self.adjust_load(1);

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
        // pollset_ctl(pollset_fd, &pc, 1);

        // self.fd_table[pe.fd] = null_mut();

        //pe.fd = retired_fd;
        // self.retired.push_back(pe);

        //  Decrease the load metric of the thread.
        self.adjust_load(-1);
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

    pub fn start(&mut self, ctx: &ZmqThreadContext) {
        ctx.start_thread(self.worker, self.worker_routine, self);
    }

    pub fn stop(&mut self) {
        self.stopping = true;
    }

    pub fn max_fds(&mut self) -> i32 {
        return -1;
    }

    pub fn loop_(&mut self) {
        // struct pollfd
        // polldata_array[max_io_events];

        while !self.stopping {
            //  Execute any due timers.
            let timeout = self.execute_timers();

            //  Wait for events.
            // let n = pollset_poll(
            //     pollset_fd,
            //     polldata_array,
            //     max_io_events,
            //     if timeout { timeout } else { -1 },
            // );
            // if (n == -1) {
            //     // errno_assert (errno == EINTR);
            //     continue;
            // }

            // for (int i = 0; i < n; i+= 1)
            // for i in 0..n {
            //     // ZmqPollEntry *pe = fd_table[polldata_array[i].fd];
            //     if (!pe) {
            //         continue;
            //     }
            //
            //     if (pe.fd == retired_fd) {
            //         continue;
            //     }
            //     if (polldata_array[i].revents & (POLLERR | POLLHUP)) {
            //         pe.events.in_event();
            //     }
            //     if (pe.fd == retired_fd) {
            //         continue;
            //     }
            //     if (polldata_array[i].revents & POLLOUT) {
            //         pe.events.out_event();
            //     }
            //     if (pe.fd == retired_fd) {
            //         continue;
            //     }
            //     if (polldata_array[i].revents & POLLIN) {
            //         pe.events.in_event();
            //     }
            // }

            //  Destroy retired event sources.
            // for (retired_t::iterator it = retired.begin (); it != retired.end ();
            // += 1it)
            // LIBZMQ_DELETE (*it);
            for retired in self.retired {
                // LIBZMQ_DELETE(retired);
            }
            self.retired.clear();
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
//     worker.Stop ();
//
//     pollset_destroy (pollset_fd);
//     for (retired_t::iterator it = retired.begin (); it != retired.end (); += 1it)
//         LIBZMQ_DELETE (*it);
// }

// #endif
