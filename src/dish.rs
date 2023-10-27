

use std::collections::HashSet;
use std::ffi::c_void;
use crate::address::ZmqAddress;
use crate::ctx::ZmqContext;
use crate::defines::{MSG_COMMAND, MSG_MORE, ZMQ_DISH, ZMQ_GROUP_MAX_LENGTH};
use crate::fair_queue::ZmqFairQueue;
use crate::io_thread::ZmqIoThread;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::session_base::ZmqSessionBase;
use crate::socket_base::ZmqSocketBase;

pub type ZmqSubscriptions = HashSet<String>;

pub struct ZmqDish<'a> {
    pub socket_base: ZmqSocketBase<'a>,
    pub _fq: ZmqFairQueue,
    pub _dist: dist_t,
    pub _subscriptions: ZmqSubscriptions,
    pub _has_message: bool,
    pub _message: ZmqMsg,
}

impl ZmqDish {
    pub unsafe fn new(options: &mut ZmqOptions, parent_: &mut ZmqContext, tid_: u32, sid_: i32) -> Self {
        options.type_ = ZMQ_DISH;
        options.linger = 0;
        let mut out = Self {
            socket_base: ZmqSocketBase::new(parent_, tid_, sid_, false),
            _fq: ZmqFairQueue::new(),
            _dist: dist_t::new(),
            _subscriptions: ZmqSubscriptions::new(),
            _has_message: false,
            _message: ZmqMsg::new(),
        };

        let rc = out._message.init2();

        out
    }

    pub fn xattach_pipe(&mut self, pipe_: &mut ZmqPipe, subscribe_to_all_: bool)  {
        self._fq.attach(pipe_);
        self._dist.attach(pipe_);

        self.send_subscriptions(pipe_);
    }

    pub fn xread_activated(&mut self, pipe_: &mut ZmqPipe) {
        self._fq.activated(pipe_);
    }

    pub fn xwrite_activated(&mut self, pipe_: &mut ZmqPipe) {
        self._dist.activated(pipe_);
    }

    pub fn xpipe_terminated(&mut self, pipe_: &mut ZmqPipe) {
        self._fq.terminated(pipe_);
        self._dist.terminated(pipe_);
    }

    pub fn xhiccuped(&mut self, pipe_: &mut ZmqPipe) {
        self.send_subscriptions(pipe_)
    }

    pub fn xjoin(&mut self, group_: &str) -> i32 {
        if group_.len() > ZMQ_GROUP_MAX_LENGTH {
            return -1;
        }

        self._subscriptions.insert(group_.to_string());

        let mut msg = ZmqMsg::new();
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

        let mut msg = ZmqMsg::new();
        let mut rc = msg.init_leave();

        rc = msg.set_group(group_);;

        rc = self._dist.send_to_all(&mut msg);

        let mut rc2 = msg.close();

        rc
    }

    pub fn xsend(&mut self, msg_: &mut ZmqMsg) -> i32 {
        unimplemented!()
    }

    pub fn xhas_out(&mut self) -> bool {
        true
    }

    pub unsafe fn xrecv(&mut self, msg_: &mut ZmqMsg) -> i32 {
        if self._has_message {
            let mut rc = msg_.move_(self._message);
            self._has_message = false;
            return 0;
        }

        self.xxrecv(msg_)
    }

    pub unsafe fn xxrecv(&mut self, msg_: &mut ZmqMsg) -> i32 {
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

    pub unsafe fn send_subscriptions(&mut self pipe_: &mut ZmqPipe)
    {
        for it in self._subscriptions.iter_mut() {
            let mut msg = ZmqMsg::new();
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
    pub session_base: ZmqSessionBase<'a>,
    pub _state: dish_session_state_t,
    pub _group_msg: ZmqMsg,
}

impl dish_session_t {
    pub unsafe fn new(io_thread_: &mut ZmqIoThread, connect_: bool, socket_: &mut ZmqSocketBase, options_: &mut ZmqOptions, addr_: ZmqAddress) -> Self {
        let mut out = Self {
            session_base: ZmqSessionBase::new(io_thread_, connect_, socket_, options_, addr_),
            _state: dish_session_state_t::group,
            _group_msg: ZmqMsg::new(),
        };

        out
    }

    pub unsafe fn push_msg(&mut self, msg_: &mut ZmqMsg) -> i32 {
        if self._state == dish_session_state_t::group {
            if msg_.flags() & MSG_MORE != MSG_MORE {
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

        if msg_.flags() & MSG_MORE != MSG_MORE {
            return -1;
        }

        rc = self.session_base.push_msg(msg_);;
        if rc == 0 {
            self._state = dish_session_state_t::group;
        }

        rc
    }

    pub unsafe fn pull_msg(&mut self, msg_: &mut ZmqMsg) -> i32
    {
        let mut rc = self.session_base.pull_msg(msg_);
        if rc != 0 {
            return rc;
        }

        if msg_.is_join() == false && msg_.is_leave() == false {
            return rc;
        }

        let group_length = msg_.group().len();

        let mut command_ = ZmqMsg::new();
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

        command_.set_flags(MSG_COMMAND);
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
