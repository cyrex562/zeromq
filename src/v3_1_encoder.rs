use crate::encoder::encoder_base_t;
use crate::msg::{cancel_cmd_name, cancel_cmd_name_size, MSG_COMMAND, MSG_MORE, sub_cmd_name, sub_cmd_name_size};
use crate::utils::put_u64;
use crate::v2_protocol::{command_flag, large_flag, more_flag};

pub struct v3_1_encoder_t {
    pub encoder_base: encoder_base_t<v3_1_encoder_t>,
    pub _tmp_buf: Vec<u8>,
}

impl v3_1_encoder_t {
    pub fn new(bufsize_: usize) -> Self {
        let mut out = Self {
            encoder_base: encoder_base_t::new(bufsize_),
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
            *protocol_flags |= more_flag;
        }
        if (self.in_progress().flags() & MSG_COMMAND || self.in_progress().is_subscribe() || self.in_progress().is_cancel()) {
            *protocol_flags |= command_flag;
            if (self.in_progress().is_subscribe()) {
                size += sub_cmd_name_size;
            } else if (self.in_progress().is_cancel()) {
                size += cancel_cmd_name_size;
            }
        }
        // Calculate large_flag after command_flag. Subscribe or cancel commands
        // increase the message size.
        if (size > u8::MAX) {
            *protocol_flags |= large_flag;
        }

        //  Encode the message length. For messages less then 256 bytes,
        //  the length is encoded as 8-bit unsigned integer. For larger
        //  messages, 64-bit unsigned integer in network byte order is used.
        if ((size > u8::MAX)) {
            put_u64(self._tmp_buf.as_mut_ptr().add(1), size);
            header_size = 9; // flags byte + size 8 bytes
        } else {
            self._tmp_buf[1] = (size);
        }

        //  Encode the sub/cancel command string. This is done in the encoder as
        //  opposed to when the subscribe message is created to allow different
        //  protocol behaviour on the wire in the v3.1 and legacy encoders.
        //  It results in the work being done multiple times in case the sub
        //  is sending the subscription/cancel to multiple pubs, but it cannot
        //  be avoided. This processing can be moved to xsub once support for
        //  ZMTP < 3.1 is dropped.
        if (self.in_progress().is_subscribe()) {
            libc::memcpy(self._tmp_buf + header_size, sub_cmd_name,
                         sub_cmd_name_size);
            header_size += sub_cmd_name_size;
        } else if (self.in_progress().is_cancel()) {
            libc::memcpy(self._tmp_buf + header_size, cancel_cmd_name,
                         cancel_cmd_name_size);
            header_size += cancel_cmd_name_size;
        }

        self.next_step(self._tmp_buf, header_size, self.size_ready, false);
    }

    pub unsafe fn size_ready(&mut self) {
        //  Write message body into the buffer.
        self.next_step(self.in_progress().data(), self.in_progress().size(),
                       message_ready, true);
    }
}
