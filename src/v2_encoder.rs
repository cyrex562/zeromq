use crate::defines::{MSG_COMMAND, MSG_MORE};
use crate::encoder::ZmqEncoderBase;
use crate::utils::put_u64;

pub struct V2Encoder {
    pub encoder_base: ZmqEncoderBase<V2Encoder>,
    pub _tmpbuf: [u8; 10],
}

impl V2Encoder {
    pub fn new(bufsize_: usize) -> Self {
        let mut out = Self {
            encoder_base: ZmqEncoderBase::new(bufsize_),
            _tmpbuf: [0; 10],
        };
        out.next_step(None, 0, out.message_ready, true);
        out
    }

    // void zmq::v2_encoder_t::message_ready ()
    pub unsafe fn message_ready(&mut self) {
        //  Encode flags.
        let mut size = self.in_progress().size();
        let mut header_size = 2; // flags byte + size byte
        let mut protocol_flags = &mut _tmp_buf[0];
        protocol_flags = 0;
        if (self.in_progress().flags() & MSG_MORE) {
            protocol_flags |= more_flag;
        }
        if (self.in_progress().size() > u8::MAX) {
            protocol_flags |= large_flag;
        }
        if (self.in_progress().flags() & MSG_COMMAND) {
            protocol_flags |= command_flag;
        }
        if (self.in_progress().is_subscribe() || self.in_progress().is_cancel()) {
            size += 1;
        }

        //  Encode the message length. For messages less then 256 bytes,
        //  the length is encoded as 8-bit unsigned integer. For larger
        //  messages, 64-bit unsigned integer in network byte order is used.
        if ((size > u8::MAX)) {
            put_u64(self._tmp_buf + 1, size);
            header_size = 9; // flags byte + size 8 bytes
        } else {
            self._tmp_buf[1] = (size as u8);
        }

        //  Encode the subscribe/cancel byte. This is done in the encoder as
        //  opposed to when the subscribe message is created to allow different
        //  protocol behaviour on the wire in the v3.1 and legacy encoders.
        //  It results in the work being done multiple times in case the sub
        //  is sending the subscription/cancel to multiple pubs, but it cannot
        //  be avoided. This processing can be moved to xsub once support for
        //  ZMTP < 3.1 is dropped.
        if (self.in_progress().is_subscribe()) {
            self._tmp_buf[header_size] = 1;
            header_size += 1;
        } else if (self.in_progress().is_cancel()) {
            self._tmp_buf[header_size] = 0;
            header_size += 1;
        }

        self.next_step(self._tmp_buf, header_size, size_ready, false);
    }

    // void zmq::v2_encoder_t::size_ready ()
    pub unsafe fn size_ready(&mut self) {
        //  Write message body into the buffer.
        self.next_step(self.in_progress().data(), self.in_progress().size(),
                       &V2Encoder::message_ready, true);
    }
}
