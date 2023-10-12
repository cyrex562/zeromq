#![allow(non_camel_case_types)]

use std::ptr::null_mut;
use crate::array::array_t;
use crate::msg::{more, msg_t};
use crate::pipe::pipe_t;

pub type pipes_t = array_t<pipe_t, 1>;

pub struct fq_t {
    pub _pipes: pipes_t,
    pub _active: usize,
    pub _current: usize,
    pub _more: bool,

}

impl fq_t {
    pub fn new() -> Self {
        Self {
            _pipes: pipes_t::new(),
            _active: 0,
            _current: 0,
            _more: false,
        }
    }

    pub fn attach(&mut self, pipe_: *mut pipe_t) {
        self._pipes.push_back(pipe_);
        self._pipes.swap(self._active, self._pipes.size() - 1);
        self._active += 1;
    }

    pub fn pipe_terminated(&mut self, pipe_: *mut pipe_t) {
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

    pub fn activated(&mut self, pipe_: *mut pipe_t) {
        self._pipes.swap(self._pipes.index(pipe_).unwrap(), self._active);
    }

    pub unsafe fn recv(&mut self, msg_: *mut msg_t) -> i32 {
        self.recvpipe(msg_, null_mut())
    }

    pub unsafe fn recvpipe(&mut self, msg_: *mut msg_t, pipe_: &mut Option<&mut pipe_t>) -> i32 {
        let mut rc = (*msg_).close();

        while self._active > 0 {
            let fetched = (*self._pipes[self._current]).read(msg_);

            if fetched {
                if pipe_ != null_mut() {
                    *pipe_ = self._pipes[self._current];
                }
                self._more = msg_.flags() & more != 0;
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
