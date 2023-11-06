use crate::defines::{ZMQ_GROUP_MAX_LENGTH, ZMQ_MSG_COMMAND, ZMQ_MSG_MORE};
use crate::msg::ZmqMsg;
use std::ffi::c_void;
use crate::err::ZmqError;
use crate::err::ZmqError::SessionError;
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


pub unsafe fn dish_sess_push_msg(session: &mut ZmqSession, msg_: &mut ZmqMsg) -> Result<(), ZmqError> {
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

    if session.push_msg(msg_).is_ok() {
        session._state = ZmqSessionState::Group;
    }

    Ok(())
}

pub unsafe fn dish_sess_pull_msg(session: &mut ZmqSession, msg_: &mut ZmqMsg) -> Result<(), ZmqError> {
    session.pull_msg(msg_)?;
    if msg_.is_join() == false && msg_.is_leave() == false {
        return Err(SessionError("invalid message"));
    }

    let group_length = msg_.group().len();

    let mut command_ = ZmqMsg::new();
    let mut offset = 0i32;

    if msg_.is_join() {
        command_.init_size(group_length + 5)?;
        offset = 5;
        libc::memcpy(command_.data(), "\x04JOIN".as_ptr() as *const c_void, 5);
    } else {
        command_.init_size(group_length + 6)?;
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

    msg_.close()?;

    *msg_ = command_;

    return Ok(());
}

pub unsafe fn dish_sess_reset(session: &mut ZmqSession) {
    session.reset();
    session._state = ZmqSessionState::Group;
}
