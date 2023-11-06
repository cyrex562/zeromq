use crate::address::ZmqAddress;
use crate::err::ZmqError;
use crate::err::ZmqError::SessionError;
use crate::io::io_thread::ZmqIoThread;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::session::ZmqSession;
use crate::socket::ZmqSocket;

pub enum ReqSessionState {
    bottom,
    request_id,
    body,
}

pub struct ReqSessionT<'a> {
    pub session_base: ZmqSession<'a>,
    pub _state: ReqSessionState,
}

impl ReqSessionT {
    pub fn new(
        io_thread_: &mut ZmqIoThread,
        connect_: bool,
        socket_: &mut ZmqSocket,
        options_: &ZmqOptions,
        addr_: ZmqAddress,
    ) -> Self {
        Self {
            session_base: ZmqSession::new(io_thread_, connect_, socket_, addr_),
            _state: ReqSessionState::bottom,
        }
    }

    // int zmq::ReqSessionT::push_msg (msg_t *msg_)
    pub unsafe fn push_msg(&mut self, msg_: &mut ZmqMsg) -> Result<(),ZmqError> {
        //  Ignore commands, they are processed by the engine and should not
        //  affect the state machine.
        if msg_.flags() & ZmqMsg::command {
            return Ok(());
        }

        match self._state {
            ReqSessionState::bottom => {
                if msg_.flags() == ZmqMsg::more {
                    //  In case option ZMQ_CORRELATE is on, allow request_id to be
                    //  transferred as first frame (would be too cumbersome to check
                    //  whether the option is actually on or not).
                    if msg_.size() == 4 {
                        self._state = ReqSessionState::request_id;
                        return self.session_base.push_msg(msg_);
                    }
                    if msg_.size() == 0 {
                        self._state = ReqSessionState::body;
                        return self.session_base.push_msg(msg_);
                    }
                }
            }

            ReqSessionState::request_id => {
                if msg_.flags() == ZmqMsg::more && msg_.size() == 0 {
                    self._state = ReqSessionState::body;
                    return self.session_base.push_msg(msg_);
                }
            }

            ReqSessionState::body => {
                if msg_.flags() == ZmqMsg::more {
                    return self.session_base.push_msg(msg_);
                }
                if msg_.flags() == 0 {
                    self._state = ReqSessionState::bottom;
                    return self.session_base.push_msg(msg_);
                }
            }
        }
        // errno = EFAULT;
        return Err(SessionError("invalid message"));
    }

    pub unsafe fn reset(&mut self) {
        self.session_base.reset();
        self._state = ReqSessionState::bottom;
    }
}
