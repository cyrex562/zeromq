#![allow(non_camel_case_types)]

use std::collections::HashSet;
use std::ffi::c_void;
use crate::address::address_t;
use crate::ctx::ctx_t;
use crate::defines::{ZMQ_DISH, ZMQ_GROUP_MAX_LENGTH};
use crate::fq::fq_t;
use crate::io_thread::io_thread_t;
use crate::msg::{command, more, msg_t};
use crate::options::options_t;
use crate::pipe::pipe_t;
use crate::session_base::session_base_t;
use crate::socket_base::socket_base_t;

pub type subscriptions_t = HashSet<String>;

pub struct dish_t<'a> {
    pub socket_base: socket_base_t<'a>,
    pub _fq: fq_t,
    pub _dist: dist_t,
    pub _subscriptions: subscriptions_t,
    pub _has_message: bool,
    pub _message: msg_t,
}

impl dish_t {
    pub unsafe fn new(options: &mut options_t, parent_: &mut ctx_t, tid_: u32, sid_: i32) -> Self {
        options.type_ = ZMQ_DISH;
        options.linger = 0;
        let mut out = Self {
            socket_base: socket_base_t::new(parent_, tid_, sid_, false),
            _fq: fq_t::new(),
            _dist: dist_t::new(),
            _subscriptions: subscriptions_t::new(),
            _has_message: false,
            _message: msg_t::new(),
        };

        let rc = out._message.init2();

        out
    }

    pub fn xattach_pipe(&mut self, pipe_: &mut pipe_t, subscribe_to_all_: bool)  {
        self._fq.attach(pipe_);
        self._dist.attach(pipe_);

        self.send_subscriptions(pipe_);
    }

    pub fn xread_activated(&mut self, pipe_: &mut pipe_t) {
        self._fq.activated(pipe_);
    }

    pub fn xwrite_activated(&mut self, pipe_: &mut pipe_t) {
        self._dist.activated(pipe_);
    }

    pub fn xpipe_terminated(&mut self, pipe_: &mut pipe_t) {
        self._fq.terminated(pipe_);
        self._dist.terminated(pipe_);
    }

    pub fn xhiccuped(&mut self, pipe_: &mut pipe_t) {
        self.send_subscriptions(pipe_)
    }

    pub fn xjoin(&mut self, group_: &str) -> i32 {
        if group_.len() > ZMQ_GROUP_MAX_LENGTH {
            return -1;
        }

        self._subscriptions.insert(group_.to_string());

        let mut msg = msg_t::new();
        let mut rc = msg.init_join();

        rc = msg.set_group(group_);;

        rc = self._dist.send_to_all(&mut msg);

        let mut rc2 = msg.close();

        rc
    }

    pub fn xleave(&mut self, group_: &str) -> i32 {
        if group_.len() > ZMQ_GROUP_MAX_LENGTH {
            return -1;
        }

        self._subscriptions.remove(group_);

        let mut msg = msg_t::new();
        let mut rc = msg.init_leave();

        rc = msg.set_group(group_);;

        rc = self._dist.send_to_all(&mut msg);

        let mut rc2 = msg.close();

        rc
    }

    pub fn xsend(&mut self, msg_: &mut msg_t) -> i32 {
        unimplemented!()
    }

    pub fn xhas_out(&mut self) -> bool {
        true
    }

    pub unsafe fn xrecv(&mut self, msg_: &mut msg_t) -> i32 {
        if self._has_message {
            let mut rc = msg_.move_(self._message);
            self._has_message = false;
            return 0;
        }

        self.xxrecv(msg_)
    }

    pub unsafe fn xxrecv(&mut self, msg_: &mut msg_t) -> i32 {
        loop {
            let mut rc = self._fq.recv(msg_);
            if rc < 0 {
                return -1;
            }

            let mut count = 0;
            for x in self._subscriptions.iter() {
                if x == msg_.group() {
                    count += 1;
                }
            }
            if count == 0 {
                break;
            }
        }

        0
    }

    pub unsafe fn xhas_in(&mut self) -> bool {
        if self._has_message {
            return true;
        }

        let mut rc = self.xxrecv(&mut self._message);
        if rc < 0 {
            return false;
        }

        self._has_message = true;
        true
    }

    pub unsafe fn send_subscriptions(&mut self pipe_: &mut pipe_t)
    {
        for it in self._subscriptions.iter_mut() {
            let mut msg = msg_t::new();
            let mut rc = msg.init_join();

            rc = msg.set_group(it.as_str());

            pipe_.write(&mut msg);
            // rc = self._dist.send_to_pipe(&mut msg, pipe_);
            // let mut rc2 = msg.close();
        }

        pipe_.flush();
    }
}

pub enum dish_session_state_t {
    group,
    body
}

pub struct dish_session_t<'a> {
    pub session_base: session_base_t<'a>,
    pub _state: dish_session_state_t,
    pub _group_msg: msg_t,
}

impl dish_session_t {
    pub unsafe fn new(io_thread_: &mut io_thread_t, connect_: bool, socket_: &mut socket_base_t, options_: &mut options_t, addr_: address_t) -> Self {
        let mut out = Self {
            session_base: session_base_t::new(io_thread_, connect_, socket_, options_, addr_),
            _state: dish_session_state_t::group,
            _group_msg: msg_t::new(),
        };

        out
    }

    pub unsafe fn push_msg(&mut self, msg_: &mut msg_t) -> i32 {
        if self._state == dish_session_state_t::group {
            if msg_.flags() & more != more {
                return -1;
            }

            if msg_.size() > ZMQ_GROUP_MAX_LENGTH {
                return -1;
            }

            self._group_msg = msg_.clone();
            self._state = dish_session_state_t::body;

            let mut rc = msg_.init2();
            return 0;
        }

        let group_setting = msg_.group();
        if group_setting.is_empty() {
            // goto has_group
        } else {
            let mut rc = msg_.set_group(&self._group_msg.group());
        }

        let mut rc = self._group_msg.close();

        if msg_.flags() & more != more {
            return -1;
        }

        rc = self.session_base.push_msg(msg_);;
        if rc == 0 {
            self._state = dish_session_state_t::group;
        }

        rc
    }

    pub unsafe fn pull_msg(&mut self, msg_: &mut msg_t) -> i32
    {
        let mut rc = self.session_base.pull_msg(msg_);
        if rc != 0 {
            return rc;
        }

        if msg_.is_join() == false && msg_.is_leave() == false {
            return rc;
        }

        let group_length = msg_.group().len();

        let mut command_ = msg_t::new();
        let mut offset = 0i32;

        if msg_.is_join() {
            rc = command_.init_size(group_length + 5);
            offset = 5;
            libc::memcpy(command_.data(), "\x04JOIN".as_ptr() as *const c_void, 5);
        } else {
            rc = command_.init_size(group_length + 6);
            offset = 6;
            libc::memcpy(command_.data(), "\x05LEAVE".as_ptr() as *const c_void, 6);
        }

        command_.set_flags(command);
        let mut command_data = command_.data();
        libc::memcpy(command_data.add(offset), msg_.group().as_ptr() as *const c_void, group_length);

        rc = msg_.close();

        *msg_ = command_;

        return 0;
    }

    pub unsafe fn reset(&mut self) {
        self.session_base.reset();
        self._state = dish_session_state_t::group;
    }
}
