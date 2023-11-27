use crate::defines::{ZMQ_MSG_COMMAND, ZMQ_MSG_MORE};
use crate::defines::err::ZmqError;
use crate::defines::err::ZmqError::SessionError;
use crate::msg::ZmqMsg;
use crate::session::{ZmqSession, ZmqSessionState};

// pub enum ReqSessionState {
//     bottom,
//     request_id,
//     body,
// }

// pub struct ReqSessionT<'a> {
//     pub session_base: ZmqSession<'a>,
//     pub _state: ReqSessionState,
// }
//
// impl ReqSessionT {
//     pub fn new(
//         io_thread_: &mut ZmqIoThread,
//         connect_: bool,
//         socket_: &mut ZmqSocket,
//         options_: &ZmqOptions,
//         addr_: ZmqAddress,
//     ) -> Self {
//         Self {
//             session_base: ZmqSession::new(io_thread_, connect_, socket_, addr_),
//             _state: ReqSessionState::bottom,
//         }
//     }
//
//     // int zmq::ReqSessionT::push_msg (msg_t *msg_)
//
// }


pub fn req_sess_push_msg(session: &mut ZmqSession, msg_: &mut ZmqMsg) -> Result<(), ZmqError> {
    //  Ignore commands, they are processed by the engine and should not
    //  affect the state machine.
    if msg_.flags() & ZMQ_MSG_COMMAND {
        return Ok(());
    }

    match session._state {
        ZmqSessionState::Bottom => {
            if msg_.flags() == ZMQ_MSG_MORE {
                //  In case option ZMQ_CORRELATE is on, allow request_id to be
                //  transferred as first frame (would be too cumbersome to check
                //  whether the option is actually on or not).
                if msg_.size() == 4 {
                    session._state = ZmqSessionState::RequestId;
                    // return session.push_msg(msg_);
                    return Ok(());
                }
                if msg_.size() == 0 {
                    session._state = ZmqSessionState::Body;
                    // return session.push_msg(msg_);
                    return Ok(());
                }
            }
        }

        ZmqSessionState::RequestId => {
            if msg_.flags() == ZMQ_MSG_MORE && msg_.size() == 0 {
                session._state = ZmqSessionState::Body;
                // return session.push_msg(msg_);
                return Ok(());
            }
        }

        ZmqSessionState::Body => {
            if msg_.flags() == ZMQ_MSG_MORE {
                // return session.push_msg(msg_);
                return Ok(());
            }
            if msg_.flags() == 0 {
                session._state = ZmqSessionState::Bottom;
                // return session.push_msg(msg_);
                return Ok(());
            }
        }

        _ => {}
    }
    // errno = EFAULT;
    return Err(SessionError("invalid message"));
}

pub fn req_sess_reset(session: &mut ZmqSession) {
    // session.reset();
    session._state = ZmqSessionState::Bottom;
}
