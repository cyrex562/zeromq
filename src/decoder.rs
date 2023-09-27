use std::ptr::null_mut;
use libc::size_t;
use crate::i_decoder::i_decoder;
use crate::msg::msg_t;

pub type step_t = fn(*mut u8) -> i32;

pub struct decoder_base_t<T, A: c_single_alloctor> {
    pub _next: Option<step_t>,
    pub _read_pos: *mut u8,
    pub _to_read: usize,
    pub _allocator: A,
    pub _buf: *mut u8,

}

impl <T, A: c_single_alloctor> decoder_base_t <T, A: c_single_alloctor> {
    pub fn new(buf_size_: usize) -> Self {
        Self {
            _next: None,
            _read_pos: null_mut(),
            _to_read: 0,
            _allocator: A::new(buf_size_),
        }
    }
}

impl i_decoder for decoder_base_t<T, A: c_single_allocator> {
    fn get_buffer(&mut self, data_: *mut *mut u8, size_: *mut usize) {
        todo!()
    }

    fn resize_buffer(&mut self, size_: usize) {
        todo!()
    }

    fn decode(&mut self, data_: *mut u8, size_: usize, processed_: &size_t) -> i32 {
        todo!()
    }

    fn msg(&mut self) -> *mut msg_t {
        todo!()
    }
}
