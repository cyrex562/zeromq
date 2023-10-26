

use std::ffi::c_void;
use std::ptr::null_mut;
use libc::size_t;
use crate::i_decoder::IDecoder;
use crate::msg::ZmqMsg;
use crate::decoder_allocators::{ZmqAllocator, ZmqCSingleAllocator};

pub type StepFn = fn(&mut [u8]) -> i32;

pub struct ZmqDecoderBase<T, A: ZmqAllocator> {
    pub _next: Option<StepFn>,
    pub _read_pos: usize,
    pub _to_read: usize,
    pub _allocator: A,
    pub _buf: Vec<u8>,

}

impl <T, A: ZmqAllocator> ZmqDecoderBase<T, A> {
    pub fn new(buf_size_: usize) -> Self {
        let mut out = Self {
            _next: None,
            _read_pos: null_mut(),
            _to_read: 0,
            _allocator: A::new(buf_size_),
            _buf: null_mut(),
        };
        out._buf = out._allocator.allocate();
        out
    }

    pub fn next_step(&mut self, read_pos_: usize, to_read_: usize, next_: StepFn) {
        self._read_pos = read_pos_;
        self._to_read = to_read_;
        self._next = Some(next_);
    }

    pub fn get_allocator(&mut self) -> &mut A {
        &mut self._allocator
    }
}

impl<T,A: ZmqAllocator> IDecoder for ZmqDecoderBase<T, A> {
    unsafe fn get_buffer(&mut self) -> Vec<u8>
    {
        // self._buf = self._allocator.allocate();
        self._buf = vec![];
        if self._to_read >= self._allocator.size() {
            *data_ = self._read_pos;
            *size_ = self._to_read;
            return;
        }
        *data_ = self._buf;
        *size_ = self._allocator.size();
    }

    unsafe fn resize_buffer(&mut self, size_: usize) {
        self._allocator.resize(size_);
    }

    unsafe fn decode(&mut self, data_: *mut u8, size_: usize, bytes_used: &mut size_t) -> i32 {
        *bytes_used = 0;
        if data_ == self._read_pos {
            self._read_pos = self._read_pos.add(size_);
            self._to_read-=size_;
            *bytes_used = size_;

            while self._to_read == 0 {
               let rc = self._next.unwrap()(data_.add(*bytes_used));
                if rc != 0 {return rc;}
            }
            return 0;
        }

        while *bytes_used < size_ {
            let to_copy = std::cmp::min(self._to_read, size_ - *bytes_used);
            unsafe {
                std::ptr::copy_nonoverlapping(data_.add(*bytes_used), self._read_pos, to_copy);
            }
            self._read_pos = self._read_pos.add(to_copy);
            self._to_read -= to_copy;
            *bytes_used = *bytes_used + to_copy;
            if self._to_read == 0 {
                let rc = self._next.unwrap()(data_.add(*bytes_used));
                if rc != 0 {return rc;}
            }
        }

        return 0;
    }

    fn msg(&mut self) -> &mut ZmqMsg {
        todo!()
    }
}
