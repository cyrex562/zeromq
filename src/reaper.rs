use crate::command::ZmqCommand;
use crate::ctx::ZmqContext;
use crate::defines::ZmqHandle;
use crate::endpoint::ZmqEndpointUriPair;
use crate::fd::retired_fd;
use crate::i_engine::IEngine;
use crate::i_poll_events::IPollEvents;
use crate::mailbox::ZmqMailbox;
use crate::object::{object_ops, ZmqObject};
use crate::own::ZmqOwn;
use crate::pipe::ZmqPipe;
use crate::poller::ZmqPoller;
use crate::socket_base::ZmqSocketBase;
use crate::utils::get_errno;
use libc::{getpid, EAGAIN, EINTR};
use std::ffi::c_void;

pub struct ZmqReaper {
    pub _object: ZmqObject,
    pub _mailbox: ZmqMailbox,
    pub _mailbox_handle: ZmqHandle,
    pub _poller: *mut ZmqPoller,
    pub _sockets: i32,
    pub _terminating: bool,
    #[cfg(feature = "have_fork")]
    pub _pid: pid_t,
}

impl IPollEvents for ZmqReaper {
    unsafe fn in_event(&mut self) {
        loop {
            #[cfg(feature = "have_fork")]
            {
                if self._pid != libc::getpid() {
                    return;
                }
            }

            let mut cmd: ZmqCommand = ZmqCommand::new();
            let rc = self._mailbox.recv(&mut cmd);
            if rc != 0 && get_errno() == EINTR {
                continue;
            }
            if rc != 0 && get_errno() == EAGAIN {
                break;
            }

            cmd.destination.process_command(cmd);
        }
    }

    fn out_event(&mut self) {
        unimplemented!()
    }

    fn timer_event(&mut self, id_: i32) {
        unimplemented!()
    }
}

impl ZmqReaper {
    pub fn new(ctx_: *mut ZmqContext, tid_: u32) -> Self {
        let mut out = Self {
            _mailbox: ZmqMailbox::new(ctx_, tid_),
            _mailbox_handle: 0 as ZmqHandle,
            _poller: &mut ZmqPoller::new(ctx_, 1),
            _sockets: 0,
            _terminating: false,
            #[cfg(feature = "have_fork")]
            _pid: 0,
            _object: ZmqObject::new(ctx_, tid_),
        };
        if out._mailbox.get_fd() != retired_fd {
            (*out._poller).add(out._mailbox.get_fd(), &mut out);
            out._sockets += 1;
            out._poller.set_pollin(out._mailbox_handle);
        }

        out
    }

    pub fn get_mailbox(&mut self) -> &mut ZmqMailbox {
        &mut self._mailbox
    }

    pub fn start(&mut self) {
        self._poller.start("Reaper")
    }

    pub fn stop(&mut self) {
        if self.get_mailbox().valid() {
            self._object.send_stop();
        }
    }
}

impl object_ops for ZmqReaper {
    unsafe fn process_stop(&mut self) {
        self._terminating = true;
        if self._sockets == 0 {
            self._poller.rm_fd(self._mailbox_handle);
            self._poller.stop();
        }
    }

    fn process_plug(&mut self) {
        todo!()
    }

    fn process_own(&mut self, object_: *mut ZmqOwn) {
        todo!()
    }

    fn process_attach(&mut self, engine_: *mut dyn IEngine) {
        todo!()
    }

    fn process_bind(&mut self, pipe_: *mut ZmqPipe) {
        todo!()
    }

    fn process_activate_read(&mut self) {
        todo!()
    }

    fn process_activate_write(&mut self, msgs_read_: u64) {
        todo!()
    }

    fn process_hiccup(&mut self, pipe_: *mut c_void) {
        todo!()
    }

    fn process_pipe_peer_stats(
        &mut self,
        queue_count_: u64,
        socket_base: *mut ZmqOwn,
        endpoint_pair_: *mut ZmqEndpointUriPair,
    ) {
        todo!()
    }

    fn process_pipe_stats_publish(
        &mut self,
        outbound_queue_count_: u64,
        inbound_queue_count: u64,
        endpoint_pair_: *mut ZmqEndpointUriPair,
    ) {
        todo!()
    }

    fn process_pipe_term(&mut self) {
        todo!()
    }

    fn process_pipe_term_ack(&mut self) {
        todo!()
    }

    fn process_pipe_hwm(&mut self, inhwm_: i32, outhwm_: i32) {
        todo!()
    }

    fn process_pipe_term_req(&mut self, object_: *mut ZmqOwn) {
        todo!()
    }

    fn process_term(&mut self, linger_: i32) {
        todo!()
    }

    fn process_term_ack(&mut self) {
        todo!()
    }

    fn process_term_endpoint(&mut self, endpoint_: &str) {
        todo!()
    }

    unsafe fn process_reap(&mut self, socket_: *mut ZmqSocketBase) {
        socket_.start_reaping(self._poller);
        self._sockets += 1;
    }

    unsafe fn process_reaped(&mut self) {
        self._sockets -= 1;
        if self._terminating && self._sockets == 0 {
            self._poller.rm_fd(self._mailbox_handle);
            self._poller.stop();
        }
    }

    fn process_conn_failed(&mut self) {
        todo!()
    }

    fn process_seqnum(&mut self) {
        todo!()
    }
}
