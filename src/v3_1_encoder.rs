use crate::defines::{cancel_cmd_name, COMMAND_FLAG, LARGE_FLAG, MORE_FLAG, MSG_COMMAND, MSG_MORE, sub_cmd_name};
use crate::encoder::ZmqEncoder;
use crate::msg::{CANCEL_CMD_NAME_SIZE, SUB_CMD_NAME_SIZE};
use crate::utils::put_u64;

pub struct V31Encoder {
    // pub encoder_base: ZmqEncoder<V31Encoder>,
    pub _tmp_buf: Vec<u8>,
}

impl V31Encoder {
    pub fn new(bufsize_: usize) -> Self {
        let mut out = Self {
            encoder_base: ZmqEncoder::new(bufsize_),
            _tmp_buf: vec![0; 11],
        };
        out.next_step(None, 0, out.message_ready, true);
        out
    }
}

// void zmq::v3_1_encoder_t::message_ready ()
pub fn v3_1e_message_ready(encoder: &mut ZmqEncoder) {
    //  Encode flags.
    let mut size = encoder.in_progress().size();
    let mut header_size = 2; // flags byte + size byte
    let mut protocol_flags = &mut encoder._tmp_buf[0];
    *protocol_flags = 0;
    if (encoder.in_progress().flags() & MSG_MORE) {
        *protocol_flags |= MORE_FLAG;
    }
    if (encoder.in_progress().flags() & MSG_COMMAND || encoder.in_progress().is_subscribe() || encoder.in_progress().is_cancel()) {
        *protocol_flags |= COMMAND_FLAG;
        if (encoder.in_progress().is_subscribe()) {
            size += SUB_CMD_NAME_SIZE;
        } else if (encoder.in_progress().is_cancel()) {
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
        put_u64(encoder._tmp_buf.as_mut_ptr().add(1), size);
        header_size = 9; // flags byte + size 8 bytes
    } else {
        encoder._tmp_buf[1] = (size);
    }

    //  Encode the sub/cancel command string. This is Done in the encoder as
    //  opposed to when the subscribe message is created to allow different
    //  protocol behaviour on the wire in the v3.1 and legacy encoders.
    //  It results in the work being Done multiple times in case the sub
    //  is sending the subscription/cancel to multiple pubs, but it cannot
    //  be avoided. This processing can be moved to xsub once support for
    //  ZMTP < 3.1 is dropped.
    if (encoder.in_progress().is_subscribe()) {
        libc::memcpy(encoder._tmp_buf + header_size, sub_cmd_name, SUB_CMD_NAME_SIZE);
        header_size += SUB_CMD_NAME_SIZE;
    } else if (encoder.in_progress().is_cancel()) {
        libc::memcpy(
            encoder._tmp_buf + header_size,
            cancel_cmd_name,
            CANCEL_CMD_NAME_SIZE,
        );
        header_size += CANCEL_CMD_NAME_SIZE;
    }

    encoder.next_step(encoder._tmp_buf, header_size, false, v3_1e_size_ready);
}

pub fn v3_1e_size_ready(encoder: &mut ZmqEncoder) {
    //  Write message body into the buffer.
    encoder.next_step(
        encoder.in_progress().data(),
        encoder.in_progress().size(),
        true,
        v3_1e_message_ready,
    );
}
