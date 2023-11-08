use crate::msg::ZmqMsg;
use crate::pipe::pipes::ZmqPipes;
use crate::pipe::ZmqPipe;
use std::ptr::null_mut;
use crate::err::ZmqError;
use crate::err::ZmqError::PipeError;

#[derive(Default, Debug, Clone)]
pub struct ZmqLoadBalancer<'a> {
    pub _pipes: ZmqPipes<'a>,
    pub _active: usize,
    pub _current: usize,
    pub _more: bool,
    pub _dropping: bool,
}

impl ZmqLoadBalancer {
    pub fn new() -> ZmqLoadBalancer {
        ZmqLoadBalancer {
            _pipes: ZmqPipes::new(),
            _active: 0,
            _current: 0,
            _more: false,
            _dropping: false,
        }
    }

    pub fn attach(&mut self, pipe_: &mut ZmqPipe) {
        self._pipes.push_back(pipe_);
        self.activated(pipe_);
    }

    pub fn pipe_terminated(&mut self, pipe_: &mut ZmqPipe) {
        let index = self._pipes.index(pipe_).unwrap();

        if index == self._current && self._more == true {
            self._dropping = true;
        }

        if index < self._active {
            self._active -= 1;
            self._pipes.swap(index, self._active);
            if (self._current == self._active) {
                self._current = 0;
            }
        }
        self._pipes.erase(pipe_);
    }

    pub fn activated(&mut self, pipe_: &mut ZmqPipe) {
        self._pipes
            .swap(self._pipes.index(pipe_).unwrap(), self._active);
        self._active += 1;
    }

    pub fn send(&mut self, msg_: &mut ZmqMsg) -> Result<(),ZmqError> {
        self.sendpipe(msg_, &mut None)
    }

    pub fn sendpipe(&mut self, msg_: &mut ZmqMsg, pipe_: &mut Option<&mut ZmqPipe>) -> Result<(),ZmqError> {
        if self._dropping {
            self._more = msg_.flags() & ZmqMsg::MORE != 0;
            self._dropping = self._more;

            (msg_).close()?;

            (msg_).init2()?;
        }

        while self._active > 0 {
            if (*self._pipes[self._current]).write(msg_) {
                if pipe_ != null_mut() {
                    *pipe_ = self._pipes[self._current];
                    break;
                }
            }

            if self._more {
                self._pipes[self._current].rollback();
                self._dropping = msg_.flags() & ZmqMsg::MORE != 0;
                self._more = false;
                return Err(PipeError("pipe is null"));
            }

            self._active -= 1;
            if self._current < self._active {
                self._pipes.swap(self._current, self._active);
            } else {
                self._current = 0;
            }
        }

        if self._active == 0 {
            return Err(PipeError("pipe is null"));
        }

        self._more = msg_.flags() & ZmqMsg::MORE != 0;
        if self._more {
            self._pipes[self._current].flush();
            self._current += 1;
            if self._current >= self._active {
                self._current = 0;
            }
        }

        (msg_).init2()?;
        Ok(())
    }

    pub fn has_out(&mut self) -> bool {
        if self._more {
            return true;
        }

        while self._active > 0 {
            if (*self._pipes[self._current]).chech_write() {
                return true;
            }

            self._active -= 1;
            self._pipes.swap(self._current, self._active);
            if self._current >= self._active {
                self._current = 0;
            }
        }

        return false;
    }
}
