use anyhow::anyhow;
use std::mem;
use crate::config::CRYPTO_BOX_NONCEBYTES;
use crate::message::{CANCEL_CMD_NAME, CANCEL_CMD_NAME_SIZE, SUB_CMD_NAME, SUB_CMD_NAME_SIZE, ZMQ_MSG_COMMAND, ZMQ_MSG_MORE, ZmqMessage};
use crate::utils::copy_bytes;
use crate::zmq_hdr::{ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_SEQUENCE, ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_MESSAGE, ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND};

#[derive(Default,Debug,Clone)]
pub struct ZmqCurveEncoding
{
    // public:
    // typedef u64 nonce_t;
//   private:
    // const char *_encode_nonce_prefix;
    pub encode_nonce_prefix: Vec<u8>,
    // const char *_decode_nonce_prefix;
    pub decode_nonce_prefix: Vec<u8>,
    // nonce_t _cn_nonce;
    pub cn_nonce: ZmqNonce,
    // nonce_t _cn_peer_nonce;
    pub cn_peer_nonce: ZmqNonce,
    //  Intermediary buffer used to speed up boxing and unboxing.
    // uint8_t _cn_precom[crypto_box_BEFORENMBYTES];
    pub cn_precom: Vec<u8>,
    // const _downgrade_sub: bool
    pub downgrade_sub: bool,
    // ZMQ_NON_COPYABLE_NOR_MOVABLE (curve_encoding_t)
}

impl ZmqCurveEncoding {
    // curve_encoding_t (encode_nonce_prefix_: *const c_char,
    //     decode_nonce_prefix_: *const c_char,
    //     const downgrade_sub_: bool);

// int encode (msg: &mut ZmqMessage);

// int decode (msg: &mut ZmqMessage error_event_code_: *mut i32);

    // uint8_t *get_writable_precom_buffer () { return _cn_precom; }
    pub fn get_writable_precom_buffer(&mut self) -> &mut [u8] {
        &mut self.cn_precom
    }

    // const uint8_t *get_precom_buffer () const { return _cn_precom; }
    pub fn get_precom_buffer(&self) -> &[u8] {
        &self.cn_precom
    }

    // nonce_t get_and_inc_nonce () { return _cn_nonce++; }
    pub fn get_and_inc_nonce(&mut self) -> ZmqNonce {
        self.cn_nonce += 1;
        self.cn_nonce
    }

    // void set_peer_nonce (nonce_t peer_nonce_) { _cn_peer_nonce = peer_nonce_; }
    pub fn set_peer_nonce(&mut self, peer_nonce: ZmqNonce) {
        self.cn_peer_nonce = peer_nonce;
    }

    // int check_validity (msg: &mut ZmqMessage error_event_code_: *mut i32);

    // pub impl curve_encoding_t

// curve_encoding_t::curve_encoding_t (encode_nonce_prefix_: * const c_char,
// decode_nonce_prefix_: * const c_char,
// const downgrade_sub_: bool):
// _encode_nonce_prefix (encode_nonce_prefix_),
// _decode_nonce_prefix (decode_nonce_prefix_),
// _cn_nonce (1),
// _cn_peer_nonce (1),
// _downgrade_sub (downgrade_sub_)
// {}
    pub fn new(encode_nonce_prefix: &str,
               decode_nonce_prefix: &str,
               downgrade_sub: bool) -> Self {
        Self {
            encode_nonce_prefix: encode_nonce_prefix.as_bytes().into_vec(),
            decode_nonce_prefix: decode_nonce_prefix.as_bytes().into_vec(),
            cn_peer_nonce: 1,
            cn_nonce: 1,
            downgrade_sub: downgrade_sub,
            cn_precom: vec![],
        }
    }

    pub fn check_validity(&mut self, msg: &mut ZmqMessage, error_event_code: &mut u32) -> anyhow::Result<()>
    {
        let size = msg.size();
        let message = msg.data().unwrap();

        // if (size < message_command_len
        // || 0 != memcmp (message, message_command, message_command_len))

        if size < message_command_len ||
            messsage != message_command

        {
            *error_event_code = ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND;
            // errno = EPROTO;
            // return - 1;
            return Err(anyhow!("EPROTO: unexpected command: {}", &message))
        }

        if (size < message_header_len + CRYPTO_BOX_MACBYTES + flags_len) {
            *error_event_code = ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_MESSAGE;
// errno = EPROTO;
// return - 1;
            return Err(anyhow!("EPROTO: malformed command message"));
        }

        {
            let nonce: u64 = get_uint64(message + message_command_len);
            if (nonce <= _cn_peer_nonce) {
                *error_event_code = ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_SEQUENCE;
// errno = EPROTO;
// return - 1;
                return Err(anyhow!("EPROTO: invalid sequence"));
            }
            set_peer_nonce(nonce);
        }

// return 0;
        Ok(())
    }

pub fn encode (&mut self, msg: & mut ZmqMessage) -> anyhow::Result<()>
{
let mut sub_cancel_len = 0usize;
let mut message_nonce: [u8; CRYPTO_BOX_NONCEBYTES as usize] = [0;CRYPTO_BOX_NONCEBYTES];
// memcpy (message_nonce, _encode_nonce_prefix, nonce_prefix_len);
copy_bytes(&mut message_nonce, 0, _encode_nonce_prefix, 0, nonce_prefix_len);
    put_uint64 (message_nonce + nonce_prefix_len, get_and_inc_nonce ());

if (msg.is_subscribe () || msg.is_cancel ()) {
if (self.downgrade_sub) {
    sub_cancel_len = 1;
}
else {
    sub_cancel_len = if msg.is_cancel() { ZmqMessage::CANCEL_CMD_NAME_SIZE } else { ZmqMessage::SUB_CMD_NAME_SIZE };
}
}

// #ifdef ZMQ_HAVE_CRYPTO_BOX_EASY_FNS
let mlen = flags_len + sub_cancel_len + msg.size ();
// std::vector < uint8_t > message_plaintext (mlen);
// #else
let mut message_plaintext: Vec<u8> = Vec::with_capacity(mlen);
//     const size_t mlen =
// CRYPTO_BOX_ZEROBYTES + flags_len + sub_cancel_len + msg.size ();
// std::vector < uint8_t > message_plaintext_with_zerobytes (mlen);
// uint8_t * const message_plaintext =
// & message_plaintext_with_zerobytes[CRYPTO_BOX_ZEROBYTES];

// std::fill (message_plaintext_with_zerobytes.begin (),
// message_plaintext_with_zerobytes.begin () + CRYPTO_BOX_ZEROBYTES,
// 0);
message_plaintext.fill(0);
// #endif

let flags = msg.flags () & flag_mask;
message_plaintext[0] = flags;

// For backward compatibility subscribe/cancel command messages are not stored with
// the message flags, and are encoded in the encoder, so that messages for < 3.0 peers
// can be encoded in the "old" 0/1 way rather than as commands.
if (sub_cancel_len == 1) {
    message_plaintext[flags_len] = if msg.is_subscribe() {
        1
    } else { 0 };
}
else if (sub_cancel_len == ZmqMessage::SUB_CMD_NAME_SIZE) {
    message_plaintext[0] |= ZMQ_MSG_COMMAND;
    // memcpy ( &message_plaintext[flags_len], SUB_CMD_NAME,     ZmqMessage::SUB_CMD_NAME_SIZE);
    copy_bytes(&mut mesage_plaintext, flags_len, SUB_CMD_NAME, 0, SUB_CMD_NAME_SIZE);

} else if (sub_cancel_len == ZmqMessage::CANCEL_CMD_NAME_SIZE) {
    message_plaintext[0] |= ZMQ_MSG_COMMAND;
    // memcpy ( &message_plaintext[flags_len], CANCEL_CMD_NAME,
    // ZmqMessage::CANCEL_CMD_NAME_SIZE);
    copy_bytes(&mut message_plaintext, flags_len, CANCEL_CMD_NAME, 0, CANCEL_CMD_NAME_SIZE)
}

// this is copying the data from insecure memory, so there is no point in
// using secure_allocator_t for message_plaintext
if (msg.size () > 0) {
    // memcpy(&message_plaintext[flags_len + sub_cancel_len], msg.data(),
    //        msg.size());
    copy_bytes(&mut message_plaintext, flags_len + sub_cancel_len, msg.data().unwrap().as_slice(), 0, msg.size())
}

// #ifdef ZMQ_HAVE_CRYPTO_BOX_EASY_FNS
let mut  msg_box: ZmqMessage = ZmqMessage::default();
msg_box.init_size (message_header_len + mlen + CRYPTO_BOX_MACBYTES)?;
// zmq_assert (rc == 0);

crypto_box_easy_afternm (
&mut msg_box.data().unwrap() + message_header_len,
& message_plaintext[0], mlen, message_nonce, _cn_precom)?;
// zmq_assert (rc == 0);

// msg.move (msg_box);
msg_box = msg.clone();

// uint8_t * const message = static_cast <uint8_t * > (msg.data ());
let message = msg.data_mut().unwrap().as_slice();
// #else
// std::vector < uint8_t > message_box (mlen);
//
// int rc =
// crypto_box_afternm ( & message_box[0], & message_plaintext_with_zerobytes[0],
// mlen, message_nonce, _cn_precom);
// zmq_assert (rc == 0);
//
// rc = msg.close ();
// zmq_assert (rc == 0);
//
// rc = msg.init_size (16 + mlen - CRYPTO_BOX_BOXZEROBYTES);
// zmq_assert (rc == 0);
//
// uint8_t *const message = static_cast < uint8_t * > (msg.data ());
//
// memcpy (message + message_header_len, & message_box[CRYPTO_BOX_BOXZEROBYTES],
// mlen - CRYPTO_BOX_BOXZEROBYTES);
// #endif

// memcpy (message, message_command, message_command_len);
copy_bytes(message, 0, message_command, 0, message_command_len);

// memcpy (message + message_command_len, message_nonce + nonce_prefix_len,
// mem::size_of::< ZmqNonce > ());

// return 0;
Ok(())
}

pub fn decode (&mut self, msg: & mut ZmqMessage, error_event_code_: &mut u32) -> anyhow::Result<()>
{
    check_validity (msg, error_event_code_)?;
    // if (0 != rc) {
    // return rc;


// uint8_t * const message = static_cast < uint8_t * > (msg.data ());
let message = msg.data().unwrap().as_slice();

// uint8_t message_nonce[CRYPTO_BOX_NONCEBYTES];
let mut message_nonce: [u8; CRYPTO_BOX_NONCEBYTES as usize] = [0;CRYPTO_BOX_NONCEBYTES];

    // memcpy (message_nonce, _decode_nonce_prefix, nonce_prefix_len);
    copy_bytes(&mut message_nonce, 0, self.decode_nonce_prefix.as_slice(), 0, nonce_prefix_len);

// memcpy (message_nonce + nonce_prefix_len, message + message_command_len,
// mem::size_of::< ZmqNonce >());
    copy_bytes(&mut message_nonce, nonce_prefix_len as usize, message, message_command_len as usize, mem::size_of::<ZmqNonce>());

// #ifdef ZMQ_HAVE_CRYPTO_BOX_EASY_FNS
let clen = msg.size () - message_header_len;

// uint8_t * const message_plaintext = message + message_header_len;
let message_plaintext = message[message_header_len..];

crypto_box_open_easy_afternm (message_plaintext,
message + message_header_len, clen,
message_nonce, _cn_precom)?;
// #else
// const size_t clen =
// CRYPTO_BOX_BOXZEROBYTES + msg.size () - message_header_len;
//
// std::vector <uint8_t > message_plaintext_with_zerobytes (clen);
// std::vector< uint8_t > message_box (clen);
//
// std::fill (message_box.begin (),
// message_box.begin () + CRYPTO_BOX_BOXZEROBYTES, 0);
// memcpy ( & message_box[CRYPTO_BOX_BOXZEROBYTES], message + message_header_len,
// msg.size () - message_header_len);
//
// rc = crypto_box_open_afternm ( & message_plaintext_with_zerobytes[0],
// & message_box[0], clen, message_nonce,
// _cn_precom);
//
// const uint8_t * const message_plaintext =
// &message_plaintext_with_zerobytes[CRYPTO_BOX_ZEROBYTES];
// #endif

// if (rc == 0) {
let flags = message_plaintext[0];

// #ifdef ZMQ_HAVE_CRYPTO_BOX_EASY_FNS
let plaintext_size = clen - flags_len - CRYPTO_BOX_MACBYTES;

if (plaintext_size > 0) {
// memmove (msg.data (), & message_plaintext[flags_len],
// plaintext_size);

}

msg.shrink (plaintext_size);
// // #else
// rc = msg.close ();
// zmq_assert (rc == 0);
//
// rc = msg.init_size (clen - flags_len - CRYPTO_BOX_ZEROBYTES);
// zmq_assert (rc == 0);
//
// // this is copying the data to insecure memory, so there is no point in
// // using secure_allocator_t for message_plaintext
// if (msg.size () > 0) {
// memcpy (msg.data (), & message_plaintext[flags_len],
// msg.size ());
// }
// // #endif

msg.set_flags (flags & flag_mask);
// } else {
// // CURVE I : connection key used for MESSAGE is wrong
// * error_event_code_ = ZMQ_PROTOCOL_ERROR_ZMTP_CRYPTOGRAPHIC;
// errno = EPROTO;
// }

// return rc;
    Ok(())
// }
}
