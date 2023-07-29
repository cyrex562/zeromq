use std::ptr::null_mut;
use crate::ypipe_base::ypipe_base_t;
use crate::yqueue::yqueue_t;

pub struct ypipe_t<T: Clone + PartialEq, const N: usize> {
    pub base: ypipe_base_t<T>,
    pub _queue: yqueue_t<T,N>,
    pub _w: *mut T,
    pub _r: *mut T,
    pub _f: *mut T,
    pub _c: *mut T
}

impl <T: Clone + PartialEq, const N: usize> ypipe_t<T,N>
{
    pub unsafe fn new() -> Self {
        let mut out = Self  {
            base: ypipe_base_t::new(),
            _queue: yqueue_t::new(),
            _w: std::ptr::null_mut(),
            _r: std::ptr::null_mut(),
            _f: std::ptr::null_mut(),
            _c: std::ptr::null_mut()
        };
        out._queue.push();
        out._r = out._queue.back_mut();
        out._w = out._r;
        out._f = out._r;
        out._c = out._queue.back_mut();
        out
    }

    pub unsafe fn write(&mut self, value_: &mut T, incomplete_: bool)
    {
        self._queue.set_back(value_);
        self._queue.push();
        if !incomplete_ {
            self._f = &mut *self._queue.back_mut();
        }
    }
    
    pub unsafe fn unwrite(&mut self, value_: *mut T) -> bool {
        if self._f == self._queue.back_mut() {
            return false;
        }
        self._queue.unpush();
        *value_ = self._queue.back().clone();
        return true;
    }

    pub fn flush(&mut self) -> bool {
        if self._w == self._f {
            return true;
        }

        if self._c != self._w {
            self._c = self._f;
            self._w = self._f;
            return false;
        }

        self._w = self._f;
        return true;
    }

    pub unsafe fn check_read(&mut self) ->  bool
    {
        if self._queue.front_mut() != self._r && self._r != null_mut() {
            return true;
        }

        self._r = self._c;
        if self._queue.front_mut() == self._r || self._r == null_mut() {
            return false;
        }

        return true;
    }

    pub unsafe fn read(&mut self, value_: *mut T) -> bool {
        if !self.check_read() {
            return false;
        }

        *value_ = self._queue.front().clone();
        self._queue.pop();
        return true;
    }
    
    pub unsafe fn probe(&mut self, func: fn (t: &T)->bool) -> bool {
        if !self.check_read() {
            return false;
        }

        if !func(self._queue.front()) {
            return false;
        }

        // self._queue.pop();
        return true;
    }

}