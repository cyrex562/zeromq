use libc::size_t;

use crate::decoder::v1_decoder::v1d_one_byte_size_ready;
use crate::decoder::v2_decoder::v2d_one_byte_size_ready;
use crate::err::ZmqError;
use crate::msg::ZmqMsg;

pub mod raw_decoder;
pub mod v1_decoder;
pub mod v2_decoder;

pub type StepFn = fn(decoder: &mut ZmqDecoder, &mut [u8]) -> Result<(), ZmqError>;

pub enum DecoderType {
    V1Decoder,
    V2Decoder,
    RawDecoder,
}

pub struct ZmqDecoder<'a> {
    pub next: Option<StepFn>,
    pub read_pos: &'a mut [u8],
    pub to_read: usize,
    // pub allocator: A,
    pub buf: Vec<u8>,
    pub _in_progress: ZmqMsg,
    pub decoder_type: DecoderType,
    pub _tmpbuf: [u8; 8],
    pub _max_msg_size: i64,
    pub _zero_copy: bool,
    pub _msg_flags: u8,
}

impl ZmqDecoder {
    pub fn new(buf_size_: usize, decoder_type: DecoderType) -> Self {
        let mut out = Self {
            next: None,
            read_pos: &mut [0u8],
            to_read: 0,
            // allocator: A::new(buf_size_),
            buf: Vec::with_capacity(buf_size_),
            _in_progress: ZmqMsg::default(),
            decoder_type: decoder_type,
            _tmpbuf: [0; 8],
            _max_msg_size: 0,
            _zero_copy: false,
            _msg_flags: 0,
        };
        // out.buf = out.allocator.allocate();
        // TODO: set next step based on DECODER type
        match out.decoder_type {
            DecoderType::V1Decoder => {
                out.next_step(&mut out._tmpbuf, 0, v1d_one_byte_size_ready);
            }
            DecoderType::V2Decoder => {
                out.next_step(&mut out._tmpbuf, 0, v2d_one_byte_size_ready);
            }
            // TODO
            DecoderType::RawDecoder => {}
        }
        out
    }

    pub fn next_step(&mut self, read_pos_: &mut [u8], to_read_: usize, next_: StepFn) {
        self.read_pos = read_pos_;
        self.to_read = to_read_;
        self.next = Some(next_);
    }

    // pub fn get_allocator(&mut self) -> &mut A {
    //     &mut self.allocator
    // }

    pub fn get_buffer(&mut self) -> &Vec<u8> {
        // // self._buf = self._allocator.allocate();
        // self.buf = vec![];
        // // if self.to_read >= self.allocator.size()
        // if self.to_read >= self.buf.len()
        // {
        //     *data_ = self.read_pos;
        //     *size_ = self.to_read;
        //     return;
        // }
        // *data_ = self.buf;
        // *size_ = self.buf.len();
        self.buf.as_ref()
    }

    pub fn resize_buffer(&mut self, size_: usize) {
        // self.allocator.resize(size_);
        self.buf.resize(size_, 0);
    }

    pub fn decode(
        &mut self,
        data_: &mut [u8],
        size_: usize,
        bytes_used: &mut size_t,
    ) -> Result<(), ZmqError> {
        *bytes_used = 0;
        if data_ == self.read_pos {
            self.read_pos = &mut self.read_pos[size_..];
            self.to_read -= size_;
            *bytes_used = size_;

            while self.to_read == 0 {
                self.next.unwrap()(self, data_.add(*bytes_used))
            }
            return Ok(());
        }

        while *bytes_used < size_ {
            let to_copy = std::cmp::min(self.to_read, size_ - *bytes_used);
            // unsafe {
            //     std::ptr::copy_nonoverlapping(data_.add(*bytes_used), self.read_pos, to_copy);
            // }

            self.read_pos = &mut self.read_pos[to_copy..];
            self.to_read -= to_copy;
            *bytes_used = *bytes_used + to_copy;
            if self.to_read == 0 {
                self.next.unwrap()(self, data_.add(*bytes_used))?;
            }
        }

        Ok(())
    }

    fn msg(&mut self) -> &mut ZmqMsg {
        todo!()
    }
}
