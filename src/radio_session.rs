use crate::context::ZmqContext;
use crate::message::{ZmqMessage, ZMQ_MSG_COMMAND, ZMQ_MSG_MORE};
use crate::session_base::ZmqSessionBase;
use crate::socket::ZmqSocket;
use crate::thread_context::ZmqThreadContext;
use crate::utils::{cmp_bytes, copy_bytes};

pub enum RadioSessionState {
    group,
    body,
}

pub struct RadioSession {
    pub session_base: ZmqSessionBase,
    pub _pending_msg: ZmqMessage,
}

impl RadioSession {
    pub fn new(
        ctx: &mut ZmqContext,
        io_thread_: &mut ZmqThreadContext,
        connect_: bool,
        socket: &mut ZmqSocket,
        options: &mut ZmqContext,
        addr_: &mut UdpAddress,
    ) -> Self {
        Self {
            session_base: ZmqSessionBase::new(ctx, io_thread_, connect_, socket, options, addr_),
            _pending_msg: ZmqMessage::new(),
        }
    }

    pub fn push_msg(&mut self, msg: &mut ZmqMessage) -> i32 {
        if (msg.flags() & ZMQ_MSG_COMMAND) {
            char * command_data = (msg.data());
            let data_size = msg.size();

            group_length: i32;
            let mut group: String = String::new();

            let mut join_leave_msg: ZmqMessage = ZmqMessage::default();
            rc: i32;

            //  Set the msg type to either JOIN or LEAVE
            if data_size >= 5 && cmp_bytes(command_data, 0, b"\x04JOIN", 0, 5) == 0 {
                group_length = (data_size) - 5;
                group = command_data + 5;
                rc = join_leave_msg.init_join();
            } else if data_size >= 6 && cmp_bytes(command_data, 0, b"\x05LEAVE", 0, 6) == 0 {
                group_length = (data_size) - 6;
                group = command_data + 6;
                rc = join_leave_msg.init_leave();
            }
            //  If it is not a JOIN or LEAVE just push the message
            else {
                return self.session_base.push_msg(msg);
            }

            // errno_assert (rc == 0);

            //  Set the Group
            rc = join_leave_msg.set_group(group);
            // errno_assert (rc == 0);

            //  Close the current command
            rc = msg.close();
            // errno_assert (rc == 0);

            //  Push the join or leave command
            *msg = join_leave_msg;
            return self.session_base.push_msg(msg);
        }
        return self.session_base.push_msg(msg);
    }

    pub fn pull_msg(&mut self, msg: &mut ZmqMessage) -> anyhow::Result<()> {
        if _state == RadioSessionState::group {
            self.session_base.pull_msg(&mut _pending_msg)?;

            let group = _pending_msg.group();
            let length: usize = group.len();

            //  First frame is the Group
            rc = msg.init_size(length as usize);
            // errno_assert (rc == 0);
            msg.set_flags(ZMQ_MSG_MORE);
            copy_bytes(msg.data_mut(), 0, group, 0, length.clone());

            //  Next status is the Body
            self._state = RadioSessionState::body;
            return Ok(());
        }
        *msg = self._pending_msg;
        self._state = RadioSessionState::group;
        Ok(())
    }

    pub fn reset(&mut self) {
        self.session_base.reset();
        _state = RadioSessionState::group;
    }
}
