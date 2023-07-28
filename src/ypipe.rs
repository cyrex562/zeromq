use std::ptr::null_mut;
use crate::ypipe_base::ypipe_base_t;
use crate::yqueue::yqueue_t;

pub struct ypipe_t<T, const N: usize> {
    pub base: ypipe_base_t<T>,
    pub _queue: yqueue_t<T,N>,
    pub _w: *mut T,
    pub _r: *mut T,
    pub _f: *mut T,
    pub _c: *mut T
}

impl <T, const N: usize> ypipe_t<T,N>
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
        if self._queue.front() != self._r && self._r != null_mut() {
            return true;
        }

        self._r = self._c;
        if self._queue.front() == self._r || self._r == null_mut() {
            return false;
        }

        return true;
    }


}