#![allow(non_camel_case_types)]

use crate::msg::msg_t;
use libc::size_t;

pub trait i_decoder {
    fn get_buffer(&mut self, data_: *mut *mut u8, size_: *mut usize);
    fn resize_buffer(&mut self, size_: usize);
    fn decode(&mut self, data_: *mut u8, size_: usize, processed_: &size_t) -> i32;
    fn msg(&mut self) -> *mut msg_t;
}
