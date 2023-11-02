use crate::encoder::ZmqEncoder;
use crate::defines::MSG_MORE;
use crate::utils::put_u64;

// pub struct V1Encoder {
//     // pub encoder_base: ZmqEncoder<V1Encoder>,
//     pub _tmpbuf: [u8;11],
// }

// impl V1Encoder
// {
//     pub fn new(bufsize_: usize) -> Self {
//         let mut out = Self {
//             encoder_base: ZmqEncoder::new(bufsize_),
//             _tmpbuf: [0;11],
//         };
//         out.next_step(None, 0, v1e_message_ready, true);
//     }
//
//
// }

pub fn v1e_size_ready(encoder: &mut ZmqEncoder)  {
    encoder.next_step(encoder.in_progress().data(), encoder.in_progress().size(), true, v1e_message_ready);
}

pub fn v1e_message_ready(encoder: &mut ZmqEncoder) {
    let mut header_size = 2; // flags byte + size byte
    //  Get the message size.
    let mut size = encoder.in_progress().size ();

    //  Account for the 'flags' byte.
    size += 1;

    //  Account for the subscribe/cancel byte.
    if encoder.in_progress().is_subscribe() || encoder.in_progress().is_cancel() {
        size += 1;
    }

    //  For messages less than 255 bytes long, write one byte of message size.
    //  For longer messages write 0xff escape character followed by 8-byte
    //  message size. In both cases 'flags' field follows.
    if size < u8::MAX as usize {
        encoder.tmp_buf[0] = size as u8;
        encoder.tmp_buf[1] = (encoder.in_progress().flags() & MSG_MORE);
    } else {
        encoder.tmp_buf[0] = u8::MAX;
        put_u64 (&mut encoder.tmp_buf[1..], size as u64);
        encoder.tmp_buf[9] = (encoder.in_progress().flags () & MSG_MORE);
        header_size = 10;
    }

    //  Encode the subscribe/cancel byte. This is Done in the encoder as
    //  opposed to when the subscribe message is created to allow different
    //  protocol behaviour on the wire in the v3.1 and legacy encoders.
    //  It results in the work being Done multiple times in case the sub
    //  is sending the subscription/cancel to multiple pubs, but it cannot
    //  be avoided. This processing can be moved to xsub once support for
    //  ZMTP < 3.1 is dropped.
    if encoder.in_progress ().is_subscribe () {
        encoder.tmp_buf[header_size] = 1;
        header_size += 1;
    }
    else if encoder.in_progress ().is_cancel () {
        encoder.tmp_buf[header_size] = 0;
        header_size += 1;
    }

    encoder.next_step (&mut encoder.tmp_buf, header_size,  false, v1e_size_ready);
}
