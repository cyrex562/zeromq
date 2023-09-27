#![allow(non_camel_case_types)]

use std::ptr::null_mut;

use crate::i_encoder::i_encoder;
use crate::msg::msg_t;

pub type step_t = fn();

pub struct encoder_base_t<T> {
    pub _write_pos: *mut u8,
    pub _to_write: usize,
    pub _next: Option<step_t>,
    pub _new_msg_flag: bool,
    pub _buf_size: usize,
    pub _buf: *mut u8,
    pub _in_progress: *mut msg_t
}

impl <T> encoder_base_t<T>
{
    pub fn new(buf_size_: usize) -> Self {
        let mut out = Self {
            _write_pos: null_mut(),
            _to_write: 0,
            _next: None,
            _new_msg_flag: false,
            _buf_size: buf_size_,
            _buf: null_mut(),
            _in_progress: null_mut()
        };
        out._buf = unsafe { libc::malloc(buf_size_) as *mut u8 };
        out
    }
    
    pub fn next_step(&mut self, write_pos_: *mut u8, to_write_: usize, new_msg_flag_: bool, next_: step_t) {
        self._write_pos = write_pos_;
        self._to_write = to_write_;
        self._new_msg_flag = new_msg_flag_;
        self._next = Some(next_);
    }
    
    pub fn in_progress(&self) -> *mut msg_t {
        self._in_progress
    }
}

impl <T> i_encoder for encoder_base_t<T> {
    unsafe fn encode(&mut self, data_: *mut *mut u8, size_: usize) -> usize {
        let buffer = if *data_ != null_mut() { *data_ } else { self._buf };
        let buffersize = if *data_ != null_mut() { size_ } else { self._buf_size };
        if self._in_progress == null_mut() {
            return 0
        }
        let mut pos = 0usize;
        while pos < buffersize {
           if self._to_write == 0 {
               if self._new_msg_flag {
                   let mut rc = self._in_progress.close();
                   rc = self._in_progress.init2();
                   self._in_progress = null_mut();
                   break;
               }
               self._next.unwrap()();
           }
            
            if pos == 0 && *data_ == null_mut() && self._to_write >= buffersize {
                *data_ = self._write_pos;
                pos = self._to_write;
                self._write_pos = null_mut();
                self._to_write = 0;
                return pos;
            }
            
            let to_copy = std::cmp::min(self._to_write, buffersize - pos);
            std::ptr::copy_nonoverlapping(self._write_pos, buffer.add(pos), to_copy);
            self._write_pos = self._write_pos.add(to_copy);
            self._to_write -= to_copy;
        }
        
        *data_ = buffer;
        return pos;
    }

    fn load_msg(&mut self, msg_: *mut msg_t) {
        self._in_progress = msg_;
        self._next.unwrap()();
    }
}
