use crate::address::ZmqAddress;
use crate::defines::{ZMQ_GROUP_MAX_LENGTH, ZMQ_MSG_COMMAND, ZMQ_MSG_MORE};
use crate::io_thread::ZmqIoThread;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::session_base::ZmqSession;
use crate::socket::ZmqSocket;
use std::ffi::c_void;

pub struct dish_session_t<'a> {
    pub session_base: ZmqSession<'a>,
    pub _state: dish_session_state_t,
    pub _group_msg: ZmqMsg,
}

impl dish_session_t {
    pub unsafe fn new(
        io_thread_: &mut ZmqIoThread,
        connect_: bool,
        socket_: &mut ZmqSocket,
        options_: &mut ZmqOptions,
        addr_: ZmqAddress,
    ) -> Self {
        let mut out = Self {
            session_base: ZmqSession::new(io_thread_, connect_, socket_, options_, addr_),
            _state: dish_session_state_t::group,
            _group_msg: ZmqMsg::new(),
        };

        out
    }

    pub unsafe fn push_msg(&mut self, msg_: &mut ZmqMsg) -> i32 {
        if self._state == dish_session_state_t::group {
            if msg_.flags() & ZMQ_MSG_MORE != ZMQ_MSG_MORE {
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

        if msg_.flags() & ZMQ_MSG_MORE != ZMQ_MSG_MORE {
            return -1;
        }

        rc = self.session_base.push_msg(msg_);
        if rc == 0 {
            self._state = dish_session_state_t::group;
        }

        rc
    }

    pub unsafe fn pull_msg(&mut self, msg_: &mut ZmqMsg) -> i32 {
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

        command_.set_flags(ZMQ_MSG_COMMAND);
        let mut command_data = command_.data();
        libc::memcpy(
            command_data.add(offset),
            msg_.group().as_ptr() as *const c_void,
            group_length,
        );

        rc = msg_.close();

        *msg_ = command_;

        return 0;
    }

    pub unsafe fn reset(&mut self) {
        self.session_base.reset();
        self._state = dish_session_state_t::group;
    }
}

pub enum dish_session_state_t {
    group,
    body,
}
