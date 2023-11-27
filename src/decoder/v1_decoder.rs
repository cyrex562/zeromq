use libc::size_t;

use crate::decoder::ZmqDecoder;
use crate::defines::err::ZmqError;
use crate::defines::err::ZmqError::DecoderError;
use crate::defines::ZMQ_MSG_MORE;
use crate::msg::ZmqMsg;
use crate::utils::get_u64;

// #[derive(Default, Debug, Clone)]
// pub struct V1Decoder {
//     // pub decoder_base: ZmqDecoderBase<V1Decoder>,
//     pub _tmpbuf: [u8; 8],
//     pub _in_progress: ZmqMsg,
//     pub _max_msg_size: i64,
// }

// impl V1Decoder {
//     pub fn new(bufsize: usize, max_msg_size: i64) -> Self {
//         let mut out = Self {
//             decoder_base: ZmqDecoder::new(bufsize),
//             _tmpbuf: [0; 8],
//             _in_progress: ZmqMsg::default(),
//             _max_msg_size: 0,
//         };
//
//         let rc = self._in_progress.init2();
//         out.next_step(out._tmpbuf, v1d_one_byte_size_ready);
//         out
//     }
// }

// int zmq::v1_decoder_t::one_byte_size_ready (unsigned char const *)
// pub unsafe fn one_byte_size_ready(&mut self, buf: &[u8]) -> i32
// {
//     //  First byte of size is read. If it is UCHAR_MAX (0xff) read 8-byte size.
//     //  Otherwise allocate the buffer for message data and read the
//     //  message data into it.
//     if (*self._tmpbuf == u8::MAX) {
//         self.next_step(self._tmpbuf, 8, &V1Decoder::eight_byte_size_ready);
//     }
//     else {
//         //  There has to be at least one byte (the flags) in the message).
//         if (!*self._tmpbuf) {
//             // errno = EPROTO;
//             return -1;
//         }
//
//         if self._max_msg_size >= 0 &&(self._tmpbuf[0] - 1) > self._max_msg_size as u8 {
//             // errno = EMSGSIZE;
//             return -1;
//         }
//
//         let mut rc = self._in_progress.close ();
//         // assert (rc == 0);
//         rc = self._in_progress.init_size (*self._tmpbuf - 1);
//         if (rc != 0) {
//             // errno_assert (errno == ENOMEM);
//             rc = self._in_progress.init ();
//             // errno_assert (rc == 0);
//             // errno = ENOMEM;
//             return -1;
//         }
//
//         self.next_step (self._tmpbuf, 1, &V1Decoder::flags_ready);
//     }
//     return 0;
// }
// int zmq::v1_decoder_t::one_byte_size_ready (unsigned char const *)
pub fn v1d_one_byte_size_ready(decoder: &mut ZmqDecoder, buf: &mut [u8]) -> Result<(), ZmqError> {
    //  First byte of size is read. If it is UCHAR_MAX (0xff) read 8-byte size.
    //  Otherwise allocate the buffer for message data and read the
    //  message data into it.
    if decoder.tmpbuf[0] == u8::MAX {
        decoder.next_step(&mut decoder.tmpbuf, 0,8, v1d_eight_byte_size_ready);
    } else {
        //  There has to be at least one byte (the flags) in the message).
        // if !decoder._tmpbuf {
        //     // errno = EPROTO;
        //     return Err(DecoderError("EPROTO"));
        // }

        if decoder.max_msg_size >= 0 && (decoder.tmpbuf[0] - 1) > decoder.max_msg_size as u8 {
            // errno = EMSGSIZE;
            return Err(DecoderError("EMSGSIZE"));
        }

        let mut rc = decoder.in_progress.close();
        // assert (rc == 0);
        if decoder.in_progress.init_size((decoder.tmpbuf[0] - 1) as usize).is_err() {
            // errno_assert (errno == ENOMEM);
            decoder.in_progress.init2()?;
            // errno_assert (rc == 0);
            // errno = ENOMEM;
            return Err(DecoderError("ENOMEM"));
        }

        decoder.next_step(&mut decoder.tmpbuf, 0,1, v1d_flags_ready);
    }
    return Ok(());
}

// pub fn msg(&mut self) -> &mut ZmqMsg {
//         self._in_progress.refm()
//     }
pub fn msg<'a>(decoder: &mut ZmqDecoder) -> &'a mut ZmqMsg {
    &mut decoder.in_progress
}

// int zmq::v1_decoder_t::eight_byte_size_ready (unsigned char const *)
// pub unsafe fn eight_byte_size_ready(&mut self, buf: &[u8]) -> i32
// {
//     //  8-byte payload length is read. Allocate the buffer
//     //  for message body and read the message data into it.
//     let payload_length = get_u64(self._tmpbuf.as_ptr());
//
//     //  There has to be at least one byte (the flags) in the message).
//     if payload_length == 0 {
//         // errno = EPROTO;
//         return -1;
//     }
//
//     //  Message size must not exceed the maximum allowed size.
//     if self._max_msg_size >= 0 && payload_length - 1 > (self._max_msg_size) as u64 {
//         // errno = EMSGSIZE;
//         return -1;
//     }
//
// // #ifndef __aarch64__
// //     //  Message size must fit within range of size_t data type.
// //     if (payload_length - 1 > std::numeric_limits<size_t>::max ()) {
// //         errno = EMSGSIZE;
// //         return -1;
// //     }
// // #endif
//
//     let msg_size = (payload_length - 1);
//
//     let mut rc = self._in_progress.close ();
//     // assert (rc == 0);
//     rc = self._in_progress.init_size (msg_size);
//     if (rc != 0) {
//         // errno_assert (errno == ENOMEM);
//         rc = self._in_progress.init2 ();
//         // errno_assert (rc == 0);
//         // errno = ENOMEM;
//         return -1;
//     }
//
//     self.next_step (self._tmpbuf, 1, &V1Decoder::flags_ready);
//     return 0;
// }
// int zmq::v1_decoder_t::eight_byte_size_ready (unsigned char const *)
pub fn v1d_eight_byte_size_ready(decoder: &mut ZmqDecoder, buf: &mut [u8]) -> Result<(), ZmqError> {
    //  8-byte payload length is read. Allocate the buffer
    //  for message body and read the message data into it.
    let payload_length = get_u64(&decoder.tmpbuf);

    //  There has to be at least one byte (the flags) in the message).
    if payload_length == 0 {
        // errno = EPROTO;
        return Err(DecoderError("EPROTO"));
    }

    //  Message size must not exceed the maximum allowed size.
    if decoder.max_msg_size >= 0 && payload_length - 1 > (decoder.max_msg_size) as u64 {
        // errno = EMSGSIZE;
        return Err(DecoderError("EMSGSIZE"));
    }

    // #ifndef __aarch64__
    //     //  Message size must fit within range of size_t data type.
    //     if (payload_length - 1 > std::numeric_limits<size_t>::max ()) {
    //         errno = EMSGSIZE;
    //         return -1;
    //     }
    // #endif

    let msg_size = (payload_length - 1);

    decoder.in_progress.close()?;
    // assert (rc == 0);
    if decoder.in_progress.init_size(msg_size as size_t).is_err() {
        // errno_assert (errno == ENOMEM);
        decoder.in_progress.init2()?;
        // errno_assert (rc == 0);
        // errno = ENOMEM;
        return Err(DecoderError("ENOMEM"));
    }

    decoder.next_step(&mut decoder.tmpbuf, 0,1, v1d_flags_ready);
    Ok(())
}

// int zmq::v1_decoder_t::flags_ready (unsigned char const *)
// pub unsafe fn flags_ready(&mut self, buf: &[u8]) -> i32
// {
//     //  Store the flags from the wire into the message structure.
//     self._in_progress.set_flags (self._tmpbuf[0] & ZMQ_MSG_MORE);
//
//     self.next_step (self._in_progress.data_mut(), self._in_progress.size (),
//                     &V1Decoder::message_ready);
//
//     return 0;
// }
pub fn v1d_flags_ready(decoder: &mut ZmqDecoder, buf: &mut [u8]) -> Result<(), ZmqError> {
    //  Store the flags from the wire into the message structure.
    decoder.in_progress.set_flags(decoder.tmpbuf[0] & ZMQ_MSG_MORE);

    decoder.next_step(
        decoder.in_progress.data_mut(),
        0,
        decoder.in_progress.size(),
        v1d_message_ready,
    );

    return Ok(());
}

// int zmq::v1_decoder_t::message_ready (unsigned char const *)
// pub unsafe fn message_ready(&mut self, buf: &[u8]) -> i32
// {
//     //  Message is completely read. Push it further and start reading
//     //  new message. (in_progress is a 0-byte message after this point.)
//     self.next_step (self._tmpbuf, 1, &V1Decoder::one_byte_size_ready);
//     return 1;
// }
pub fn v1d_message_ready(decoder: &mut ZmqDecoder, buf: &mut [u8]) -> Result<(), ZmqError> {
    //  Message is completely read. Push it further and start reading
    //  new message. (in_progress is a 0-byte message after this point.)
    decoder.next_step(&mut decoder.tmpbuf, 0,1, v1d_one_byte_size_ready);
    return Ok(());
}
