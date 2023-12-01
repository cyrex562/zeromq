use crate::defines::{ZMQ_GROUP_MAX_LENGTH, ZMQ_MSG_COMMAND, ZMQ_MSG_MORE};
use crate::defines::err::ZmqError;
use crate::defines::err::ZmqError::SessionError;
use crate::msg::ZmqMsg;
use crate::session::{ZmqSession, ZmqSessionState};

// pub struct dish_session_t<'a> {
//     pub session_base: ZmqSession<'a>,
//     pub _state: dish_session_state_t,
//     pub _group_msg: ZmqMsg,
// }

// impl dish_session_t {
//     pub unsafe fn new(
//         io_thread_: &mut ZmqIoThread,
//         connect_: bool,
//         socket_: &mut ZmqSocket,
//         options_: &mut ZmqOptions,
//         addr_: ZmqAddress,
//     ) -> Self {
//         let mut out = Self {
//             session_base: ZmqSession::new(io_thread_, connect_, socket_, addr_),
//             _state: dish_session_state_t::group,
//             _group_msg: ZmqMsg::new(),
//         };
//
//         out
//     }
//
//
// }

// pub enum dish_session_state_t {
//     group,
//     body,
// }


pub fn dish_sess_push_msg(session: &mut ZmqSession, msg_: &mut ZmqMsg) -> Result<(), ZmqError> {
    if session._state == ZmqSessionState::Group {
        if msg_.flags() & ZMQ_MSG_MORE != ZMQ_MSG_MORE {
            return Err(SessionError("no more messages"));
        }

        if msg_.size() > ZMQ_GROUP_MAX_LENGTH {
            return Err(SessionError("group message too long"));
        }

        session._group_msg = msg_.clone();
        session._state = ZmqSessionState::Body;

        msg_.init2()?;
        return Ok(());
    }

    let group_setting = msg_.group();
    if group_setting.is_empty() {
        // goto has_group
    } else {
        msg_.set_group(&session._group_msg.group())?;
    }

    session._group_msg.close()?;

    if msg_.flags() & ZMQ_MSG_MORE != ZMQ_MSG_MORE {
        return Err(SessionError("no more messages"));
    }

    // if session.push_msg(msg_).is_ok() {
    //     session._state = ZmqSessionState::Group;
    // }

    Ok(())
}

pub fn dish_sess_pull_msg(session: &mut ZmqSession, msg_: &mut ZmqMsg) -> Result<(), ZmqError> {
    session.pull_msg(msg_)?;
    if msg_.is_join() == false && msg_.is_leave() == false {
        return Err(SessionError("invalid message"));
    }

    let group_length = msg_.group().len();

    let mut command = ZmqMsg::default();
    let mut offset = 0usize;

    if msg_.is_join() {
        command.init_size(group_length + 5)?;
        offset = 5;
        // libc::memcpy(command_.data(), "\x04JOIN".as_ptr() as *const c_void, 5);
        command.data_mut().copy_from_slice("\x04JOIN".as_bytes());
    } else {
        command.init_size(group_length + 6)?;
        offset = 6;
        // libc::memcpy(command_.data(), "\x05LEAVE".as_ptr() as *const c_void, 6);
        command.data_mut().copy_from_slice("\x05LEAVE".as_bytes());
    }

    command.set_flags(ZMQ_MSG_COMMAND);
    let mut command_data = command.data_mut();
    // libc::memcpy(
    //     command_data.add(offset),
    //     msg_.group().as_ptr() as *const c_void,
    //     group_length,
    // );
    command_data[offset..].copy_from_slice(msg_.group().as_bytes());

    msg_.close()?;

    *msg_ = command;

    return Ok(());
}

pub fn dish_sess_reset(session: &mut ZmqSession) {
    // session.reset();
    session._state = ZmqSessionState::Group;
}
