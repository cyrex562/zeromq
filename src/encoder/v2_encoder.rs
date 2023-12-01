use crate::defines::{COMMAND_FLAG, LARGE_FLAG, MORE_FLAG, ZMQ_MSG_COMMAND, ZMQ_MSG_MORE};
use crate::encoder::ZmqEncoder;
use crate::utils::put_u64;

// pub struct V2Encoder {
//     // pub encoder_base: ZmqEncoder<V2Encoder>,
//     pub _tmpbuf: [u8; 10],
// }
//
// impl V2Encoder {
//     pub fn new(bufsize_: usize) -> Self {
//         let mut out = Self {
//             encoder_base: ZmqEncoder::new(bufsize_),
//             _tmpbuf: [0; 10],
//         };
//         out.next_step(None, 0, v2e_message_ready, true);
//         out
//     }
// }

pub fn v2e_message_ready(encoder: &mut ZmqEncoder) {
    //  Encode flags.
    let mut size = encoder.in_progress().size();
    let mut header_size = 2; // flags byte + size byte
    let mut protocol_flags = &mut encoder.tmp_buf[0];
    *protocol_flags = 0;
    if encoder.in_progress().flags() & ZMQ_MSG_MORE !=0 {
        *protocol_flags |= MORE_FLAG;
    }
    if encoder.in_progress().size() > u8::MAX as usize {
        *protocol_flags |= LARGE_FLAG;
    }
    if encoder.in_progress().flags() & ZMQ_MSG_COMMAND != 0{
        *protocol_flags |= COMMAND_FLAG;
    }
    if encoder.in_progress().is_subscribe() || encoder.in_progress().is_cancel() {
        size += 1;
    }

    //  Encode the message length. For messages less then 256 bytes,
    //  the length is encoded as 8-bit unsigned integer. For larger
    //  messages, 64-bit unsigned integer in network byte order is used.
    if size > u8::MAX as usize {
        put_u64(&mut encoder.tmp_buf[1..], size as u64);
        header_size = 9; // flags byte + size 8 bytes
    } else {
        encoder.tmp_buf[1] = size as u8;
    }

    //  Encode the subscribe/cancel byte. This is Done in the ENCODER as
    //  opposed to when the subscribe message is created to allow different
    //  protocol behaviour on the wire in the v3.1 and legacy encoders.
    //  It results in the work being Done multiple times in case the sub
    //  is sending the subscription/cancel to multiple pubs, but it cannot
    //  be avoided. This processing can be moved to xsub once support for
    //  ZMTP < 3.1 is dropped.
    if encoder.in_progress().is_subscribe() {
        encoder.tmp_buf[header_size] = 1;
        header_size += 1;
    } else if encoder.in_progress().is_cancel() {
        encoder.tmp_buf[header_size] = 0;
        header_size += 1;
    }

    encoder.next_step(&mut encoder.tmp_buf, header_size, false, v2e_size_ready);
}

// void zmq::v2_encoder_t::size_ready ()
pub fn v2e_size_ready(encoder: &mut ZmqEncoder) {
    //  Write message body into the buffer.
    encoder.next_step(
        encoder.in_progress().data_mut(),
        encoder.in_progress().size(),
        true,
        v2e_message_ready,
    );
}
