use crate::encoder::ZmqEncoder;

// pub struct ZmqRawEncoder {
//     pub encoder_base: ZmqEncoder,
//
// }

// impl ZmqRawEncoder {
//     pub fn new(bufsize_: usize) -> Self {
//         let mut out = Self {
//             encoder_base: ZmqEncoder::new(bufsize_),
//         };
//         out.encoder_base.next_step(None, 0, true, out.raw_message_ready);
//         out
//     }
//
//
// }
 pub fn raw_message_ready(encoder: &mut ZmqEncoder) {
        encoder.next_step((encoder.in_progress()).data_mut(), encoder.in_progress().size(), true, raw_message_ready)
    }
