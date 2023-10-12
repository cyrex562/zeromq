use crate::decoder_allocators::{allocator, shared_message_memory_allocator};
use crate::i_decoder::i_decoder;
use crate::msg::msg_t;

pub struct raw_decoder_t {
    // pub decoder: dyn i_decoder
    pub _in_progress: msg_t,
    pub _allocator: shared_message_memory_allocator,
}

impl raw_decoder_t {
    pub unsafe fn new(bufsize_: usize) -> Self {
        let mut out = Self {
            _in_progress: msg_t::new(),
            _allocator: shared_message_memory_allocator::new(bufsize_, 1),
        };
        let rc = out._in_progress.init2();
        out
    }

    pub unsafe fn get_buffer(&mut self, data_: &mut [u8], size_: &mut usize) {
        // self._allocator.get_buffer(data_, size_);
        data_.as_mut_ptr() = self._allocator.allocate();
        *size_ = self._allocator.size();
    }

    pub unsafe fn decode(&mut self, data_: &mut [u8], size_: usize, bytes_used: &mut usize) -> i32 {
        let rc = self._in_progress.init(
            data_,
            size_,
            shared_message_memory_allocator::call_dec_ref,
            self._allocator.buffer(),
            self._allocator.provide_content(),
        );
        if self._in_progress.is_zcmsg() {
            self._allocator.advance_content();
            self._allocator.release();
        }

        *bytes_used = size_;
        1
    }
}
