use crate::dbuffer::ZmqDbuffer;
use crate::ypipe::ypipe_base::ZmqYPipeBase;

pub struct YPipeConflate<T: Default + Copy> {
    pub base: ZmqYPipeBase<T>,
    pub dbuffer: ZmqDbuffer<T>,
    pub reader_awake: bool,
}

impl<T: Default + Copy> YPipeConflate<T> {
    pub fn new() -> Self {
        Self {
            base: ZmqYPipeBase::new(),
            dbuffer: ZmqDbuffer::new(),
            reader_awake: false,
        }
    }

    pub unsafe fn write(&mut self, value_: &mut T, incomplete_: bool) {
        self.dbuffer.write(value_);
    }

    pub fn unwrite(&mut self, x: *mut T) -> bool {
        return false;
    }

    pub fn flush(&mut self) -> bool {
        return self.reader_awake;
    }

    pub fn check_read(&mut self) -> bool {
        let res = self.dbuffer.check_read();
        if !res {
            self.reader_awake = false;
        }

        res
    }

    pub unsafe fn read(&mut self, value_: *mut T) -> bool {
        if (!self.check_read()) {
            return false;
        }
        self.dbuffer.read(value_)
    }

    pub fn probe(&mut self, func: fn(*mut T) -> bool) -> bool {
        self.dbuffer.probe(func)
    }
}