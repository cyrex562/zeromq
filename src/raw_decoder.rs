use crate::decoder::ZmqDecoderBase;
use crate::decoder_allocators::{ZmqAllocator, ZmqSharedMessageMemoryAllocator};
use crate::err::ZmqError;
use crate::msg::{ZmqContent, ZmqMsg};

pub struct ZmqRawDecoder {
    // pub decoder: dyn i_decoder
    pub _in_progress: ZmqMsg,
    pub _allocator: ZmqSharedMessageMemoryAllocator,
}

impl ZmqRawDecoder {
    pub unsafe fn new(bufsize_: usize) -> Self {
        let mut out = Self {
            _in_progress: ZmqMsg::new(),
            _allocator: ZmqSharedMessageMemoryAllocator::new(bufsize_, 1),
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
            ZmqSharedMessageMemoryAllocator::call_dec_ref,
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

pub fn raw_decode(decoder: &mut ZmqDecoderBase, data: &mut [u8], bytes_used: &mut usize) -> Result<usize,ZmqError>
{
    match decoder._in_progress.init3(
        data,
        decoder.buf.as_mut_slice(),
        &mut ZmqContent::default()
    ) {
        Ok(_) => {},
        Err(e) => {
            return Err(e);
        }
    };
    if decoder._in_progress.is_zcmsg() {
        // TODO
        // decoder._allocator.advance_content();
        // decoder._allocator.release();
    }
    Ok(data.len())
}
