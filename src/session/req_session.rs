use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::session_base::ZmqSession;

pub enum req_session_state {
    bottom,
    request_id,
    body,
}

pub struct req_session_t<'a> {
    pub session_base: ZmqSession<'a>,
    pub _state: req_session_state,
}

impl req_session_t {
    pub fn new(
        io_thread_: &mut io_thread_t,
        connect_: bool,
        socket_: &mut socket_base_t,
        options_: &ZmqOptions,
        addr_: address_t,
    ) -> Self {
        Self {
            session_base: ZmqSession::new(io_thread_, connect_, socket_, options_, addr_),
            _state: req_session_state::bottom,
        }
    }

    // int zmq::req_session_t::push_msg (msg_t *msg_)
    pub unsafe fn push_msg(&mut self, msg_: &mut ZmqMsg) -> i32 {
        //  Ignore commands, they are processed by the engine and should not
        //  affect the state machine.
        if (msg_.flags() & ZmqMsg::command) {
            return 0;
        }

        match (self._state) {
            req_session_state::bottom => {
                if (msg_.flags() == ZmqMsg::more) {
                    //  In case option ZMQ_CORRELATE is on, allow request_id to be
                    //  transferred as first frame (would be too cumbersome to check
                    //  whether the option is actually on or not).
                    if (msg_.size() == 4) {
                        self._state = req_session_state::request_id;
                        return ZmqSession::push_msg(msg_);
                    }
                    if (msg_.size() == 0) {
                        self._state = req_session_state::body;
                        return ZmqSession::push_msg(msg_);
                    }
                }
            }

            req_session_state::request_id => {
                if (msg_.flags() == ZmqMsg::more && msg_.size() == 0) {
                    self._state = req_session_state::body;
                    return ZmqSession::push_msg(msg_);
                }
            }

            req_session_state::body => {
                if (msg_.flags() == ZmqMsg::more) {
                    return ZmqSession::push_msg(msg_);
                }
                if (msg_.flags() == 0) {
                    self._state = req_session_state::bottom;
                    return ZmqSession::push_msg(msg_);
                }
            }
        }
        // errno = EFAULT;
        return -1;
    }

    pub unsafe fn reset(&mut self) {
        self.session_base.reset();
        self._state = req_session_state::bottom;
    }
}