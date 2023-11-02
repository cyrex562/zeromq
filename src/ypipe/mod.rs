use crate::defines::yqueue::YQueue;
use ypipe_base::ZmqYPipeBase;

pub mod ypipe_base;
pub mod ypipe_conflate;

pub struct ZmqYPipe<'a, T: Clone + PartialEq + Default> {
    pub base: ZmqYPipeBase<T>,
    pub queue: YQueue<T>,
    pub w: &'a mut T,
    pub r: &'a mut T,
    pub f: &'a mut T,
    pub c: &'a mut T,
}

impl<T: Clone + PartialEq + Default> ZmqYPipe<T> {
    pub fn new() -> Self {
        let mut out = Self {
            base: ZmqYPipeBase::new(),
            queue: YQueue::new(),
            w: &mut T::default(),
            r: &mut T::default(),
            f: &mut T::default(),
            c: &mut T::default(),
        };
        out.queue.push();
        out.r = out.queue.back_mut().unwrap();
        out.w = out.r;
        out.f = out.r;
        out.c = out.queue.back_mut().unwrap();
        out
    }

    pub fn write(&mut self, value_: &mut T, incomplete_: bool) {
        self.queue.set_back(value_);
        self.queue.push();
        if !incomplete_ {
            self.f = &mut *self.queue.back_mut();
        }
    }

    pub unsafe fn unwrite(&mut self, value_: *mut T) -> bool {
        if self.f == self.queue.back_mut() {
            return false;
        }
        self.queue.unpush();
        *value_ = self.queue.back().clone();
        return true;
    }

    pub fn flush(&mut self) -> bool {
        if self.w == self.f {
            return true;
        }

        if self.c != self.w {
            self.c = self.f;
            self.w = self.f;
            return false;
        }

        self.w = self.f;
        return true;
    }

    pub fn check_read(&mut self) -> bool {
        if self.queue.front_mut().unwrap() != self.r && self.r != &mut T::default() {
            return true;
        }

        self.r = self.c;
        if self.queue.front_mut().unwrap() == self.r || self.r == &mut T::default() {
            return false;
        }

        return true;
    }

    pub fn read(&mut self, value_: &mut T) -> bool {
        if !self.check_read() {
            return false;
        }

        *value_ = self.queue.front().clone();
        self.queue.pop();
        return true;
    }

    pub unsafe fn probe(&mut self, func: fn(t: &T) -> bool) -> bool {
        if !self.check_read() {
            return false;
        }

        if !func(self.queue.front().unwrap()) {
            return false;
        }

        // self._queue.pop();
        self.queue.pop_back();
        return true;
    }
}
