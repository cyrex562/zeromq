use crate::dealer::ZmqDealer;
use crate::defines::{MSG_MORE, ZMQ_REQ, ZMQ_REQ_CORRELATE, ZMQ_REQ_RELAXED};
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::session_base::ZmqSessionBase;

pub struct ZmqReq {
    pub dealer: ZmqDealer,
    pub _request_id: u32,
    pub _strict: bool,
}

impl ZmqReq {
    pub fn new(options: &mut ZmqOptions, parent_: &mut ctx_t, tid_: u32, sid_: i32) -> Self {
        options.type_ = ZMQ_REQ;

        Self {
            dealer: ZmqDealer::new(options, parent_, tid_, sid_),
            _request_id: 0,
            _strict: false,
        }
    }

    pub unsafe fn xsend(&mut self, msg_: &mut ZmqMsg) -> i32 {
        //  If we've sent a request and we still haven't got the reply,
        //  we can't send another request unless the strict option is disabled.
        if (self._receiving_reply) {
            if (self._strict) {
                // errno = EFSM;
                return -1;
            }

            self._receiving_reply = false;
            self._message_begins = true;
        }

        //  First part of the request is the request routing id.
        if (self._message_begins) {
            self._reply_pipe = None;

            if (self._request_id_frames_enabled) {
                self._request_id += 1;

                // msg_t id;
                let mut id = ZmqMsg::default();
                let mut rc = id.init_size(4);
                libc::memcpy(id.data_mut(), &self._request_id, 4);
                // errno_assert (rc == 0);
                id.set_flags(ZmqMsg::more);

                rc = self.dealer.sendpipe(&id, &_reply_pipe);
                if (rc != 0) {
                    return -1;
                }
            }

            // msg_t bottom;
            let mut bottom = ZmqMsg::default();
            let mut rc = bottom.init2();
            // errno_assert (rc == 0);
            bottom.set_flags(ZmqMsg::more);

            rc = self.dealer.sendpipe(&mut bottom, &self._reply_pipe);
            if (rc != 0) {
                return -1;
            }
            // zmq_assert (_reply_pipe);

            self._message_begins = false;

            // Eat all currently available messages before the request is fully
            // sent. This is Done to avoid:
            //   REQ sends request to A, A replies, B replies too.
            //   A's reply was first and matches, that is used.
            //   An hour later REQ sends a request to B. B's old reply is used.
            // msg_t drop;
            let mut drop = ZmqMsg::default();
            loop {
                rc = drop.init2();
                // errno_assert (rc == 0);
                rc = self.dealer.xrecv(&mut drop);
                if (rc != 0) {
                    break;
                }
                drop.close();
            }
        }

        let more = (msg_.flags() & ZmqMsg::more) != 0;

        let rc = self.dealer.xsend(msg_);
        if (rc != 0) {
            return rc;
        }

        //  If the request was fully sent, flip the FSM into reply-receiving state.
        if (!more) {
            self._receiving_reply = true;
            self._message_begins = true;
        }

        return 0;
    }

    // int zmq::req_t::xrecv (msg_t *msg_)
    pub unsafe fn xrecv(&mut self, msg_: &mut ZmqMsg) -> i32 {
        //  If request wasn't send, we can't wait for reply.
        if (!self._receiving_reply) {
            // errno = EFSM;
            return -1;
        }

        //  Skip messages until one with the right first frames is found.
        while (self._message_begins) {
            //  If enabled, the first frame must have the correct request_id.
            if (self._request_id_frames_enabled) {
                let rc = self.recv_reply_pipe(msg_);
                if (rc != 0) {
                    return rc;
                }

                if !(msg_.flags() & MSG_MORE)
                    || msg_.size() != size_of_val(&self._request_id)
                    || msg_.data_mut() != self._request_id
                {
                    //  Skip the remaining frames and try the next message
                    while (msg_.flags() & MSG_MORE) {
                        rc = self.recv_reply_pipe(msg_);
                        // errno_assert (rc == 0);
                    }
                    continue;
                }
            }

            //  The next frame must be 0.
            // TODO: Failing this check should also close the connection with the peer!
            let mut rc = self.recv_reply_pipe(msg_);
            if (rc != 0) {
                return rc;
            }

            if (!(msg_.flags() & MSG_MORE) || msg_.size() != 0) {
                //  Skip the remaining frames and try the next message
                while (msg_.flags() & ZmqMsg::more) {
                    rc = self.recv_reply_pipe(msg_);
                    // errno_assert (rc == 0);
                }
                continue;
            }

            self._message_begins = false;
        }

        let rc = self.recv_reply_pipe(msg_);
        if (rc != 0) {
            return rc;
        }

        //  If the reply is fully received, flip the FSM into request-sending state.
        if (!(msg_.flags() & MSG_MORE)) {
            self._receiving_reply = false;
            self._message_begins = true;
        }

        return 0;
    }

    // bool zmq::req_t::xhas_in ()
    pub unsafe fn xhas_in(&mut self) -> bool {
        //  TODO: Duplicates should be removed here.

        if (!self._receiving_reply) {
            return false;
        }

        return self.dealer.xhas_in();
    }

    // bool zmq::req_t::xhas_out ()
    pub unsafe fn xhas_out(&mut self) -> bool {
        if (self._receiving_reply && self._strict) {
            return false;
        }

        return self.dealer.xhas_out();
    }

    // int zmq::req_t::xsetsockopt (int option_,
    //                          const void *optval_,
    //                          size_t optvallen_)
    pub unsafe fn xsetsockopt(&mut self, option_: i32, optval_: &[u8], optvallen_: usize) -> i32 {
        let is_int = (optvallen_ == 4);
        let mut value = 0;
        if (is_int) {
            libc::memcpy(&value, optval_, 4);
        }

        match option_ {
            ZMQ_REQ_CORRELATE => {
                if (is_int && value >= 0) {
                    self._request_id_frames_enabled = (value != 0);
                    return 0;
                }
            }
            ZMQ_REQ_RELAXED => {
                if (is_int && value >= 0) {
                    self._strict = (value == 0);
                    return 0;
                }
            }
            _ => {}
        }

        return ZmqDealer::xsetsockopt(option_, optval_, optvallen_);
    }

    // void zmq::req_t::xpipe_terminated (pipe_t *pipe_)
    pub unsafe fn xpipe_terminated(&mut self, pipe_: &mut ZmqPipe) {
        if (self._reply_pipe == pipe_) {
            self._reply_pipe = None;
        }
        ZmqDealer::xpipe_terminated(pipe_);
    }

    pub unsafe fn recv_reply_pipe(&mut self, msg_: &mut ZmqMsg) -> i32 {
        loop {
            let mut pipe: ZmqPipe = ZmqPipe::default();
            let rc = self.dealer.recvpipe(msg_, &mut Some(&mut pipe));
            if (rc != 0) {
                return rc;
            }
            if (!self._reply_pipe || pipe == self._reply_pipe) {
                return 0;
            }
        }
    }
}

pub enum req_session_state {
    bottom,
    request_id,
    body,
}

pub struct req_session_t<'a> {
    pub session_base: ZmqSessionBase<'a>,
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
            session_base: ZmqSessionBase::new(io_thread_, connect_, socket_, options_, addr_),
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
                        return ZmqSessionBase::push_msg(msg_);
                    }
                    if (msg_.size() == 0) {
                        self._state = req_session_state::body;
                        return ZmqSessionBase::push_msg(msg_);
                    }
                }
            }

            req_session_state::request_id => {
                if (msg_.flags() == ZmqMsg::more && msg_.size() == 0) {
                    self._state = req_session_state::body;
                    return ZmqSessionBase::push_msg(msg_);
                }
            }

            req_session_state::body => {
                if (msg_.flags() == ZmqMsg::more) {
                    return ZmqSessionBase::push_msg(msg_);
                }
                if (msg_.flags() == 0) {
                    self._state = req_session_state::bottom;
                    return ZmqSessionBase::push_msg(msg_);
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
