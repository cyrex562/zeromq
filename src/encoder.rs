

use std::ptr::null_mut;

use crate::i_encoder::IEncoder;
use crate::msg::ZmqMsg;
use crate::v1_encoder::v1e_message_ready;
use crate::v2_encoder::v2e_message_ready;
use crate::v3_1_encoder::v3_1e_message_ready;

pub type StepFn = fn(&mut ZmqEncoder);

pub enum EncoderType {
    V1Encoder,
    V2Encoder,
    V31Encoder,
}

pub struct ZmqEncoder {
    pub _write_pos: usize,
    pub _to_write: usize,
    pub _next: Option<StepFn>,
    pub _new_msg_flag: bool,
    pub _buf_size: usize,
    pub buf: Vec<u8>,
    pub in_progress: ZmqMsg,
    pub tmp_buf: [u8;11],
    pub encoder_type: EncoderType,
}

impl ZmqEncoder
{
    pub fn new(buf_size_: usize) -> Self {
        let mut out = Self {
            _write_pos: 0,
            _to_write: 0,
            _next: None,
            _new_msg_flag: false,
            _buf_size: buf_size_,
            buf: Vec::with_capacity(buf_size_),
            in_progress: ZmqMsg::default()
        };
        // out._buf = unsafe { libc::malloc(buf_size_) as *mut u8 };
        match encoder_type {
            EncoderType::V1Encoder => {
                out.next_step(0, 0, true, v1e_message_ready );
            },
            EncoderType::V2Encoder => {
                out.next_step(0, 0, true, v2e_message_ready );
            },
            EncoderType::V31Encoder => {
                out.next_step(0, 0, true, v3_1e_message_ready);
            },
        }
        out
    }
    
    pub fn next_step(&mut self, write_pos_: usize, to_write_: usize, new_msg_flag_: bool, next_: StepFn) {
        self._write_pos = write_pos_;
        self._to_write = to_write_;
        self._new_msg_flag = new_msg_flag_;
        self._next = Some(next_);
    }
    
    pub fn in_progress(&mut self) -> &mut ZmqMsg {
        &mut self.in_progress
    }

    pub fn encode(&mut self, data_: &mut [u8], size_: usize) -> usize {
        let buffer = if *data_ != null_mut() { *data_ } else { self.buf };
        let buffersize = if *data_ != null_mut() { size_ } else { self._buf_size };
        if self.in_progress == ZmqMsg::default() {
            return 0
        }
        let mut pos = 0usize;
        while pos < buffersize {
           if self._to_write == 0 {
               if self._new_msg_flag {
                   let mut rc = self.in_progress.close();
                   rc = self.in_progress.init2();
                   self.in_progress = ZmqMsg::default();
                   break;
               }
               self._next.unwrap()(self);
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

    fn load_msg(&mut self, msg_: &mut ZmqMsg) {
        self.in_progress = msg_.clone();
        self._next.unwrap()(self);
    }
}
