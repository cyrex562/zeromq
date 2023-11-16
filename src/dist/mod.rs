use crate::defines::ZMQ_MSG_MORE;
use crate::msg::ZmqMsg;
use crate::pipe::pipes::ZmqPipes;
use crate::pipe::ZmqPipe;

#[derive(Default, Debug, Clone)]
pub struct ZmqDist<'a> {
    pub _pipes: ZmqPipes<'a>,
    pub _matching: usize,
    pub _active: usize,
    pub _eligible: usize,
    pub _more: bool,
}

impl ZmqDist {
    pub fn new() -> Self {
        Self {
            _pipes: ZmqPipes::new(),
            _matching: 0,
            _active: 0,
            _eligible: 0,
            _more: false,
        }
    }

    pub fn attach(&mut self, pipe_: &mut ZmqPipe) {
        //  If we are in the middle of sending a message, we'll add new pipe
        //  into the list of eligible pipes. Otherwise we add it to the list
        //  of Active pipes.
        if self._more {
            self._pipes.push_back(pipe_);
            self._pipes.swap(self._eligible, self._pipes.size() - 1);
            self._eligible += 1;
        } else {
            self._pipes.push_back(pipe_);
            self._pipes.swap(self._active, self._pipes.size() - 1);
            self._active += 1;
            self._eligible += 1;
        }
    }

    pub fn has_pipe(&mut self, pipe_: &mut ZmqPipe) -> bool {
        self._pipes.has_item(pipe_)
    }

    pub fn match_(&mut self, pipe_: &mut ZmqPipe) {
        if self._pipes.index(pipe_).unwrap() < self._matching {
            return;
        }

        if self._pipes.index(pipe_).unwrap() >= self._eligible {
            return;
        }

        self._pipes.swap(self._pipes.index(pipe_).unwrap(), self._matching);
        self._matching += 1;
    }

    pub fn reverse_match(&mut self) {
        let mut prev_matching = self._matching;
        self.unmatch();
        for i in prev_matching..self._eligible {
            self._pipes.swap(i, self._matching);
            self._matching += 1;
        }
    }

    pub fn unmatch(&mut self) {
        self._matching = 0;
    }

    pub fn pipe_terminated(&mut self, pipe_: &mut ZmqPipe) {
        if self._pipes.index(pipe_).unwrap() < self._matching {
            self._pipes.swap(self._pipes.index(pipe_).unwrap(), self._matching - 1);
            self._matching -= 1;
        }
        if self._pipes.index(pipe_).unwrap() < self._active {
            self._pipes.swap(self._pipes.index(pipe_).unwrwap(), self._active - 1);
            self._active -= 1;
        }
        if self._pipes.index(pipe_).unwrap() < self._eligible {
            self._pipes.swap(self._pipes.index(pipe_).unwrap(), self._eligible - 1);
            self._eligible -= 1;
        }

        self._pipes.erase(pipe_);
    }

    pub fn activated(&mut self, pipe_: &mut ZmqPipe) {
        //  Move the pipe from passive to eligible state.
        if self._eligible < self._pipes.size() {
            self._pipes.swap(self._pipes.index(pipe_).unwrap(), self._eligible);
            self._eligible += 1;
        }

        //  If there's no message being sent at the moment, move it to
        //  the Active state.
        if !self._more && self._active < self._pipes.size() {
            self._pipes.swap(self._eligible - 1, self._active);
            self._active += 1;
        }
    }

    pub fn send_to_all(&mut self, msg_: &mut ZmqMsg) -> Result<(),ZmqError> {
        self._matching = self._active;
        self.send_to_matching(msg_)
    }

    pub fn send_to_matching(&mut self, msg_: &mut ZmqMsg) -> Result<(),ZmqError> {
        //  Is this end of a multipart message?
        let msg_more = msg_.flag_set(ZMQ_MSG_MORE);

        //  Push the message to matching pipes.
        self.distribute(msg_);

        //  If multipart message is fully sent, activate all the eligible pipes.
        if !msg_more {
            self._active = self._eligible;
        }

        self._more = msg_more;

        Ok(())
    }

    pub fn distribute(&mut self, msg_: &mut ZmqMsg) {
        //  If there are no matching pipes available, simply drop the message.
        if self._matching == 0 {
            let mut rc = msg_.close();
            // errno_assert (rc == 0);
            rc = msg_.init2();
            // errno_assert (rc == 0);
            return;
        }

        if msg_.is_vsm() {
            // for (pipes_t::size_type i = 0; i < _matching;)
            for i in 0..self._matching {
                if !self.write(self._pipes[i], msg_) {
                    //  Use same index again because entry will have been removed.
                } else {
                    // i += 1;
                }
            }
            let mut rc = msg_.init2();
            // errno_assert (rc == 0);
            return;
        }

        //  Add matching-1 references to the message. We already hold one reference,
        //  that's why -1.
        msg_.add_refs(((self._matching) - 1) as i32);

        //  Push copy of the message to each matching pipe.
        let mut failed = 0;
        // for (pipes_t::size_type i = 0; i < _matching;)
        for i in 0..self._matching {
            if !self.write(self._pipes[i], msg_) {
                failed += 1;
                //  Use same index again because entry will have been removed.
            } else {
                // i += 1;
            }
        }
        if failed {
            msg_.rm_refs(failed);
        }

        //  Detach the original message from the data buffer. Note that we don't
        //  close the message. That's because we've already
        // used all the references.
        let rc = msg_.init2();
        // errno_assert (rc == 0);
    }

    pub fn has_out(&mut self) -> bool {
        true
    }

    pub fn write(&mut self, pipe_: &mut ZmqPipe, msg_: &mut ZmqMsg) -> bool {
        if pipe_.write(msg_).is_err() {
            self._pipes.swap(self._pipes.index(pipe_).unwrap(), self._matching - 1);
            self._matching -= 1;
            self._pipes.swap(self._pipes.index(pipe_).unwrap(), self._active - 1);
            self._active -= 1;
            self._pipes.swap(self._active, self._eligible - 1);
            self._eligible -= 1;
            return false;
        }
        if msg_.flag_clear(ZMQ_MSG_MORE) {
            for i in 0..self._matching {
                if self._pipes[i].check_hwm() {
                    return false;
                }
            }
        }

        return true;
    }

    pub fn check_hwm(&mut self) -> bool {
        for i in 0..self._matching {
            if self._pipes[i].check_hwm() {
                return false;
            }
        }
        return true;
    }
}