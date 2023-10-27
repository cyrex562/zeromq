use crate::defines::{cancel_cmd_name, sub_cmd_name, MSG_COMMAND, MSG_MORE};
use crate::encoder::ZmqEncoderBase;
use crate::msg::{CANCEL_CMD_NAME_SIZE, SUB_CMD_NAME_SIZE};
use crate::utils::put_u64;
use crate::v2_protocol::{COMMAND_FLAG, LARGE_FLAG, MORE_FLAG};

pub struct V31Encoder {
    pub encoder_base: ZmqEncoderBase<V31Decoder>,
    pub _tmp_buf: Vec<u8>,
}

impl V31Decoder {
    pub fn new(bufsize_: usize) -> Self {
        let mut out = Self {
            encoder_base: ZmqEncoderBase::new(bufsize_),
            _tmp_buf: vec![0; 11],
        };
        out.next_step(None, 0, out.message_ready, true);
        out
    }

    // void zmq::v3_1_encoder_t::message_ready ()
    pub unsafe fn message_ready(&mut self) {
        //  Encode flags.
        let mut size = self.in_progress().size();
        let mut header_size = 2; // flags byte + size byte
        let mut protocol_flags = &mut self._tmp_buf[0];
        *protocol_flags = 0;
        if (self.in_progress().flags() & MSG_MORE) {
            *protocol_flags |= MORE_FLAG;
        }
        if (self.in_progress().flags() & MSG_COMMAND
            || self.in_progress().is_subscribe()
            || self.in_progress().is_cancel())
        {
            *protocol_flags |= COMMAND_FLAG;
            if (self.in_progress().is_subscribe()) {
                size += SUB_CMD_NAME_SIZE;
            } else if (self.in_progress().is_cancel()) {
                size += CANCEL_CMD_NAME_SIZE;
            }
        }
        // Calculate LARGE_FLAG after COMMAND_FLAG. Subscribe or cancel commands
        // increase the message size.
        if (size > u8::MAX) {
            *protocol_flags |= LARGE_FLAG;
        }

        //  Encode the message length. For messages less then 256 bytes,
        //  the length is encoded as 8-bit unsigned integer. For larger
        //  messages, 64-bit unsigned integer in network byte order is used.
        if (size > u8::MAX) {
            put_u64(self._tmp_buf.as_mut_ptr().add(1), size);
            header_size = 9; // flags byte + size 8 bytes
        } else {
            self._tmp_buf[1] = (size);
        }

        //  Encode the sub/cancel command string. This is Done in the encoder as
        //  opposed to when the subscribe message is created to allow different
        //  protocol behaviour on the wire in the v3.1 and legacy encoders.
        //  It results in the work being Done multiple times in case the sub
        //  is sending the subscription/cancel to multiple pubs, but it cannot
        //  be avoided. This processing can be moved to xsub once support for
        //  ZMTP < 3.1 is dropped.
        if (self.in_progress().is_subscribe()) {
            libc::memcpy(self._tmp_buf + header_size, sub_cmd_name, SUB_CMD_NAME_SIZE);
            header_size += SUB_CMD_NAME_SIZE;
        } else if (self.in_progress().is_cancel()) {
            libc::memcpy(
                self._tmp_buf + header_size,
                cancel_cmd_name,
                CANCEL_CMD_NAME_SIZE,
            );
            header_size += CANCEL_CMD_NAME_SIZE;
        }

        self.next_step(self._tmp_buf, header_size, self.size_ready, false);
    }

    pub unsafe fn size_ready(&mut self) {
        //  Write message body into the buffer.
        self.next_step(
            self.in_progress().data(),
            self.in_progress().size(),
            message_ready,
            true,
        );
    }
}
