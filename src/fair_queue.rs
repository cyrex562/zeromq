

use std::ptr::null_mut;
use crate::array::ZmqArray;
use crate::msg::{MSG_MORE, ZmqMsg};
use crate::pipe::ZmqPipe;

pub type ZmqPipes = ZmqArray<ZmqPipe, 1>;

pub struct ZmqFairQueue {
    pub _pipes: ZmqPipes,
    pub _active: usize,
    pub _current: usize,
    pub _more: bool,

}

impl ZmqFairQueue {
    pub fn new() -> Self {
        Self {
            _pipes: ZmqPipes::new(),
            _active: 0,
            _current: 0,
            _more: false,
        }
    }

    pub fn attach(&mut self, pipe_: &mut ZmqPipe) {
        self._pipes.push_back(pipe_);
        self._pipes.swap(self._active, self._pipes.size() - 1);
        self._active += 1;
    }

    pub fn pipe_terminated(&mut self, pipe_: &mut ZmqPipe) {
        let index = self._pipes.index(pipe_).unwrap();
        if index < self._active {
            self._active -= 1;
            self._pipes.swap(index, self._active);
            if self._current == self._active {
                self._current = 0;
            }
        }
        self._pipes.erase(pipe_);
    }

    pub fn activated(&mut self, pipe_: &mut ZmqPipe) {
        self._pipes.swap(self._pipes.index(pipe_).unwrap(), self._active);
    }

    pub unsafe fn recv(&mut self, msg_: &mut ZmqMsg) -> i32 {
        self.recvpipe(msg_, null_mut())
    }

    pub unsafe fn recvpipe(&mut self, msg_: &mut ZmqMsg, pipe_: &mut Option<&mut ZmqPipe>) -> i32 {
        let mut rc = (*msg_).close();

        while self._active > 0 {
            let fetched = (*self._pipes[self._current]).read(msg_);

            if fetched {
                if pipe_ != null_mut() {
                    *pipe_ = self._pipes[self._current];
                }
                self._more = msg_.flags() & MSG_MORE != 0;
                if !self._more {
                    self._current = self._current + 1 % self._active;
                }
                return 0;
            }

            self._active -= 1;
            self._pipes.swap(self._current, self._active);
            if self._current == self._active {
                self._current = 0;
            }
        }

        rc = (*msg_).init2();
        return -1;
    }

    pub fn has_in(&mut self) -> bool {
        if self._more {
            return true;
        }

        while self._active > 0 {
            if self._pipes[self._current].check_read() {
                return true;
            }

            self._active -= 1;
            self._pipes.swap(self._current, self._active);
            if self._current == self._active {
                self._current = 0;
            }
        }
        
        return false;
    }
}
