use crate::address::ZmqAddress;
use crate::context::ZmqContext;
use crate::defines::ZMQ_GROUP_MAX_LENGTH;
use crate::message::{ZmqMessage, ZMQ_MSG_COMMAND, ZMQ_MSG_MORE};
use crate::session_base::ZmqSessionBase;
use crate::thread_context::ZmqThreadContext;
use crate::utils::copy_bytes;
use libc::EFAULT;

pub enum DishSessionState {
    Group,
    Body,
}

pub struct DishSession<'a> {
    pub _group_msg: ZmqMessage,
    pub session_base: ZmqSessionBase<'a>,
}

impl DishSession {
    pub fn new(
        io_thread: &mut ZmqThreadContext,
        connect_: bool,
        socket: &mut ZmqSocketbase,
        ctx: &mut ZmqContext,
        addr: &mut ZmqAddress,
    ) -> Self {
        DishSession {
            session_base: ZmqSessionBase::new(cx, io_thread, connect_, socket, addr),
            _group_msg: ZmqMessage::default(),
        }
    }

    //  Overrides of the functions from ZmqSessionBase.
    pub fn push_msg(&mut self, msg: &mut ZmqMessage) -> i32 {
        if self._state == DishSessionState::Group {
            if (msg.flags() & ZMQ_MSG_MORE) != ZMQ_MSG_MORE {
                errno = EFAULT;
                return -1;
            }

            if msg.size() > ZMQ_GROUP_MAX_LENGTH {
                errno = EFAULT;
                return -1;
            }

            self._group_msg = msg.clone();
            self._state = DishSessionState::Body;

            msg.init2().expect("TODO: panic message");
            // errno_assert (rc == 0);
            return 0;
        }
        let group_setting = msg.group();
        let mut rc: i32;
        if group_setting[0] != 0 {
            // goto has_group;
        }

        //  Set the message Group
        rc = msg.set_group2(
            String::from_utf8_lossy(self._group_msg.data()).as_ref(),
            self._group_msg.size(),
        );
        // errno_assert (rc == 0);

        //  We set the Group, so we don't need the group_msg anymore
        self._group_msg.close().expect("TODO: panic message");
        // errno_assert (rc == 0);
        // has_group:
        //  Thread safe socket doesn't support multipart messages
        if (msg.flags() & ZMQ_MSG_MORE) == ZMQ_MSG_MORE {
            errno = EFAULT;
            return -1;
        }

        //  Push message to dish socket
        rc = self.push_msg(msg);

        if rc == 0 {
            self._state = DishSessionState::Group;
        }

        return rc;
    }

    // int pull_msg (msg: &mut ZmqMessage);
    pub fn pull_msg(&mut self, msg: &mut ZmqMessage) -> i32 {
        let rc = self.socket_base.pull_msg(msg);

        if rc != 0 {
            return rc;
        }

        if !msg.is_join() && !msg.is_leave() {
            return rc;
        }

        let group_length: i32 = msg.group().len() as i32;

        let mut command: ZmqMessage = ZmqMessage::default();
        let mut offset: i32;

        if msg.is_join() {
            rc = command.init_size((group_length + 5) as usize);
            errno_assert(rc == 0);
            offset = 5;
            copy_bytes(command.data_mut(), 0, b"\x04JOIN", 0, 5);
        } else {
            rc = command.init_size((group_length + 6) as usize);
            errno_assert(rc == 0);
            offset = 6;
            copy_bytes(command.data_mut(), 0, b"\x05LEAVE", 0, 6);
        }

        command.set_flags(ZMQ_MSG_COMMAND);
        let mut command_data = (command.data_mut());

        //  Copy the Group
        copy_bytes(
            command_data,
            offset as usize,
            msg.group().as_bytes(),
            0,
            group_length as usize,
        );

        //  Close the join message
        rc = msg.close();
        errno_assert(rc == 0);

        *msg = command;

        return 0;
    }

    // void reset ();

    pub fn reset(&mut self) {
        self.session_base.reset();
        self._state = DishSessionState::Group;
    }
}
