use std::ptr::null_mut;
use crate::array::array_t;
use crate::msg::msg_t;
use crate::pipe::pipe_t;

pub type pipes_t = array_t<pipe_t,2>;
pub struct lb_t
{
    pub _pipes: pipes_t,
    pub _active: usize,
    pub _current: usize,
    pub _more: bool,
    pub _dropping: bool,
}

impl lb_t {
    pub fn new() -> lb_t {
        lb_t {
            _pipes: pipes_t::new(),
            _active: 0,
            _current: 0,
            _more: false,
            _dropping: false,
        }
    }

    pub fn attach(&mut self, pipe_: *mut pipe_t) {
        self._pipes.push_back(pipe_);
        self.activated(pipe_);
    }

    pub fn pipe_terminated(&mut self, pipe_: *mut pipe_t) {
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

    pub fn activated(&mut self, pipe_: *mut pipe_t) {
        self._pipes.swap(self._pipes.index(pipe_).unwrap(), self._active);
        self._active += 1;
    }

    pub unsafe fn send(&mut self, msg_: *mut msg_t) -> i32 {
        self.sendpipe(msg_, null_mut())
    }

    pub unsafe fn sendpipe(&mut self, msg_: &mut msg_t, pipe_: &mut Option<&mut pipe_t>) -> i32 {
        if self._dropping {
            self._more = msg_.flags() & msg_t::MORE != 0;
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
                self._dropping = msg_.flags() & msg_t::MORE != 0;
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

        self._more = msg_.flags() & msg_t::MORE != 0;
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
