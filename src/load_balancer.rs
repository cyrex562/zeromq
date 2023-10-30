use std::ptr::null_mut;
use crate::zmq_pipe::ZmqPipes;
use crate::msg::ZmqMsg;
use crate::pipe::ZmqPipe;


#[derive(Default,Debug,Clone)]
pub struct ZmqLoadBalancer
{
    pub _pipes: ZmqPipes,
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

    pub fn attach(&mut self, pipe_: *mut ZmqPipe) {
        self._pipes.push_back(pipe_);
        self.activated(pipe_);
    }

    pub fn pipe_terminated(&mut self, pipe_: *mut ZmqPipe) {
        let index = self._pipes.index(pipe_).unwrap();

        if index == self._current && self._more == true{
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

    pub fn activated(&mut self, pipe_: *mut ZmqPipe) {
        self._pipes.swap(self._pipes.index(pipe_).unwrap(), self._active);
        self._active += 1;
    }

    pub unsafe fn send(&mut self, msg_: *mut ZmqMsg) -> i32 {
        self.sendpipe(msg_, null_mut())
    }

    pub unsafe fn sendpipe(&mut self, msg_: &mut ZmqMsg, pipe_: &mut Option<&mut ZmqPipe>) -> i32 {
        if self._dropping {
            self._more = msg_.flags() & ZmqMsg::MORE != 0;
            self._dropping = self._more;

            (*msg_).close();

            (*msg_).init2();

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
                return -2;
            }

            self._active -= 1;
            if self._current < self._active {
                self._pipes.swap(self._current, self._active);
            } else {
                self._current = 0;
            }
        }

        if self._active == 0 {
            return -1;
        }

        self._more = msg_.flags() & ZmqMsg::MORE != 0;
        if self._more {
            self._pipes[self._current].flush();
            self._current += 1;
            if self._current >= self._active {
                self._current = 0;
            }
        }

        (*msg_).init2();
        return 0;
    }

    pub unsafe fn has_out(&mut self) -> bool {
        if self._more {
            return true;
        }

        while self._active > 0 {
            if (*self._pipes[self._current]).chech_write() { return true;}

            self._active -= 1;
            self._pipes.swap(self._current, self._active);
            if self._current >= self._active {
                self._current = 0;
            }
        }

        return false;
    }
}
