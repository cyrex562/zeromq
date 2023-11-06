use crate::address::ZmqAddress;
use crate::defines::{ZMQ_MSG_COMMAND, ZMQ_MSG_MORE};
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::socket::ZmqSocket;
use std::ffi::c_void;
use crate::err::ZmqError;
use crate::io::io_thread::ZmqIoThread;
use crate::session::ZmqSession;

pub enum radio_session_state {
    group,
    body,
}

pub struct radio_session_t<'a> {
    pub session_base: ZmqSession<'a>,
    pub _state: radio_session_state,
    pub _pending_msg: ZmqMsg,
}

impl radio_session_t {
    pub unsafe fn new(
        io_thread_: &mut ZmqIoThread,
        connect_: bool,
        socket_: &mut ZmqSocket,
        options_: &ZmqOptions,
        addr_: ZmqAddress,
    ) -> Self {
        Self {
            session_base: ZmqSession::new(io_thread_, connect_, socket_, addr_),
            _state: radio_session_state::group,
            _pending_msg: ZmqMsg::default(),
        }
    }

    pub unsafe fn push_msg(&mut self, msg_: &mut ZmqMsg) -> Result<(),ZmqError> {
        if msg_.flag_set(ZMQ_MSG_COMMAND) {
            let mut command_data = msg_.data_mut();
            let data_size = msg_.size();

            let mut group_length = 0usize;
            let mut group = String::new();

            let mut join_leave_msg = ZmqMsg::new();
            let mut rc = 0i32;

            //  Set the msg type to either JOIN or LEAVE
            if data_size >= 5 && command_data.to_string() == "\x04JOIN" {
                group_length = (data_size) - 5;
                group = command_data[5..].to_string();
                rc = join_leave_msg.init_join();
            } else if data_size >= 6 && command_data.to_string() == "\x05LEAVE" {
                group_length = (data_size) - 6;
                group = command_data[6..].to_string();
                rc = join_leave_msg.init_leave();
            }
            //  If it is not a JOIN or LEAVE just push the message
            else {
                self.session_base.push_msg(msg_)?;
                // return session_base_t::push_msg(msg_);
            }

            // errno_assert (rc == 0);

            //  Set the group
            rc = join_leave_msg.set_group(group, group_length);
            // errno_assert (rc == 0);

            //  Close the current command
            msg_.close()?;
            // errno_assert (rc == 0);

            //  Push the join or leave command
            *msg_ = join_leave_msg;
            return self.session_base.push_msg(msg_);
        }
        return self.session_base.push_msg(msg_);
    }

    pub unsafe fn pull_msg(&mut self, msg_: &mut ZmqMsg) -> Result<(),ZmqError> {
        if self._state == radio_session_state::group {
            self.session_base.pull_msg(&mut self._pending_msg)?;

            let group = self._pending_msg.group();
            let length = group.len();

            //  First frame is the group
            msg_.init_size(length)?;
            // errno_assert (rc == 0);
            msg_.set_flags(ZMQ_MSG_MORE);
            // libc::memcpy(
            //     msg_.data_mut() as *mut c_void,
            //     group.as_ptr() as *const c_void,
            //     length,
            // );
            msg_.data_mut().clone_from_slice(group.as_bytes());

            //  Next status is the body
            self._state = radio_session_state::body;
            return Ok(());
        }
        *msg_ = self._pending_msg;
        self._state = radio_session_state::group;
        return Ok(());
    }

    pub unsafe fn reset(&mut self) {
        self.session_base.reset();
        self._state = radio_session_state::group;
    }
}
