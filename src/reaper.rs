#![allow(non_camel_case_types)]

use std::ffi::c_void;
use libc::{EAGAIN, EINTR, getpid};
use crate::command::command_t;
use crate::ctx::ctx_t;
use crate::defines::handle_t;
use crate::endpoint::endpoint_uri_pair_t;
use crate::fd::retired_fd;
use crate::i_engine::i_engine;
use crate::i_poll_events::i_poll_events;
use crate::mailbox::mailbox_t;
use crate::object::{object_ops, object_t};
use crate::own::own_t;
use crate::pipe::pipe_t;
use crate::poller::poller_t;
use crate::socket_base::socket_base_t;
use crate::utils::get_errno;

pub struct reaper_t {
    pub _object: object_t,
    pub _mailbox: mailbox_t,
    pub _mailbox_handle: handle_t,
    pub _poller: *mut poller_t,
    pub _sockets: i32,
    pub _terminating: bool,
    #[cfg(feature="have_fork")]
    pub _pid: pid_t,
}

impl i_poll_events for reaper_t {
    unsafe fn in_event(&mut self) {
        loop {
            #[cfg(feature="have_fork")]
            {
                if self._pid != libc::getpid() {
                    return;
                }
            }

            let mut cmd: command_t = command_t::new();
            let rc = self._mailbox.recv(&mut cmd);
            if rc != 0 && get_errno() == EINTR {
                continue;
            }
            if rc != 0  && get_errno() == EAGAIN {
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

impl reaper_t {
    pub unsafe fn new(ctx_: *mut ctx_t, tid_: u32) -> Self
    {
        let mut out = Self {
            _mailbox: mailbox_t::new(ctx_, tid_),
            _mailbox_handle: 0 as handle_t,
            _poller: &mut poller_t::new(ctx_, 1),
            _sockets: 0,
            _terminating: false,
            #[cfg(feature="have_fork")]
            _pid: 0,
            _object: object_t::new(ctx_, tid_)
        };
        if out._mailbox.get_fd() != retired_fd {
            (*out._poller).add(out._mailbox.get_fd(), &mut out);
            out._sockets += 1;
            out._poller.set_pollin(out._mailbox_handle);
        }

        out
    }

    pub unsafe fn get_mailbox(&mut self) -> *mut mailbox_t
    {
        &mut self._mailbox
    }

    pub fn start(&mut self)
    {
        self._poller.start("Reaper")
    }

    pub unsafe fn stop(&mut self)
    {
        if self.get_mailbox().valid() {
            self._object.send_stop();
        }
    }
}

impl object_ops for reaper_t {
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

    fn process_own(&mut self, object_: *mut own_t) {
        todo!()
    }

    fn process_attach(&mut self, engine_: *mut dyn i_engine) {
        todo!()
    }

    fn process_bind(&mut self, pipe_: *mut pipe_t) {
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

    fn process_pipe_peer_stats(&mut self, queue_count_: u64, socket_base: *mut own_t, endpoint_pair_: *mut endpoint_uri_pair_t) {
        todo!()
    }

    fn process_pipe_stats_publish(&mut self, outbound_queue_count_: u64, inbound_queue_count: u64, endpoint_pair_: *mut endpoint_uri_pair_t) {
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

    fn process_pipe_term_req(&mut self, object_: *mut own_t) {
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

    unsafe fn process_reap(&mut self, socket_: *mut socket_base_t)
    {
        socket_.start_reaping(self._poller);
        self._sockets += 1;
    }

    unsafe fn process_reaped(&mut self)
    {
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
