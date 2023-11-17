use crate::defines::dbuffer::ZmqDbuffer;
use crate::ypipe::ypipe_base::ZmqYPipeBase;

pub struct YPipeConflate<'a, T: Default> {
    pub base: ZmqYPipeBase<T>,
    pub dbuffer: ZmqDbuffer<'a,T>,
    pub reader_awake: bool,
}

impl<'a, T: Default> YPipeConflate<'a, T> {
    pub fn new() -> Self {
        Self {
            base: ZmqYPipeBase::new(),
            dbuffer: ZmqDbuffer::new(),
            reader_awake: false,
        }
    }

    pub fn write(&mut self, value_: &mut T, incomplete: bool) {
        self.dbuffer.write(value_);
    }

    pub fn unwrite(&mut self, x: &mut T) -> bool {
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

    pub fn read(&mut self, value_: &mut T) -> bool {
        if !self.check_read() {
            return false;
        }
        self.dbuffer.read(value_)
    }

    pub fn probe(&mut self, func: fn(&mut T) -> bool) -> bool {
        self.dbuffer.probe(func)
    }
}
