use crate::decoder::{DecoderType, ZmqDecoder};
use crate::defines::{
    ZMQ_CURVE, ZMQ_GSSAPI, ZMQ_MSG_CANCEL, ZMQ_MSG_PING, ZMQ_MSG_PONG, ZMQ_MSG_ROUTING_ID,
    ZMQ_MSG_SUBSCRIBE, ZMQ_NULL, ZMQ_PLAIN, ZMQ_PROTOCOL_ERROR_ZMTP_MECHANISM_MISMATCH, ZMQ_PUB,
    ZMQ_XPUB,
};
use crate::encoder::{EncoderType, ZmqEncoder};
use crate::engine::stream_engine::{
    HEARTBEAT_TIMEOUT_TIMER_ID, HEARTBEAT_TTL_TIMER_ID, stream_next_handshake_command,
    stream_process_handshake_command, stream_pull_and_encode, stream_pull_msg_from_session,
    stream_push_msg_to_session, stream_read,
};
use crate::engine::ZmqEngine;
use crate::mechanism::ZmqMechanism;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::utils::{get_errno, put_u64};
use libc::EAGAIN;
use std::cmp::min;
use std::mem::{size_of, size_of_val};
use crate::defines::err::ZmqError;
use crate::defines::err::ZmqError::EngineError;
use crate::msg::defines::{CANCEL_CMD_NAME_SIZE, PING_CMD_NAME_SIZE, SUB_CMD_NAME_SIZE};

pub const ZMTP_1_0: i32 = 0;
pub const ZMTP_2_0: i32 = 1;
pub const ZMTP_3_x: i32 = 3;

pub const SIGNATURE_SIZE: usize = 10;
pub const V2_GREETING_SIZE: usize = 12;

pub const V3_GREETING_SIZE: usize = 64;

pub const REVISION_POS: usize = 10;
pub const MINOR_POS: usize = 11;

// pub struct ZmtpEngine<'a> {
//     pub stream_engine_base: ZmqStreamEngineBase<'a>,
//     pub routing_id_msg: ZmqMsg,
//     pub pong_msg: ZmqMsg,
//     pub greeting_size: usize,
//     pub greeting_recv: [u8; V3_GREETING_SIZE],
//     pub greeting_send: [u8; V3_GREETING_SIZE],
//     pub greeting_bytes_read: u32,
//     pub subscription_required: bool,
//     pub heartbeat_timeout: i32,
// }

// impl ZmtpEngine {
//     pub unsafe fn new(fd_: ZmqFd, options_: &ZmqOptions, endpoint_uri_pair_: &ZmqEndpointUriPair) -> Self
//     {
//         let mut out = Self {
//             stream_engine_base: ZmqStreamEngineBase::new(fd_, options_, endpoint_uri_pair_, true),
//             _routing_id_msg: ZmqMsg::default(),
//             _pong_msg: ZmqMsg::default(),
//             _greeting_size: V2_GREETING_SIZE,
//             _greeting_recv: [0; V3_GREETING_SIZE],
//             _greeting_send: [0; V3_GREETING_SIZE],
//             _greeting_bytes_read: 0,
//             _subscription_required: false,
//             _heartbeat_timeout: 0,
//         };
//
//         out.stream_engine_base._next_msg = &mut out._routing_id_msg;
//         out.stream_engine_base._process_msg = &mut out.process_routing_id_msg;
//         out._pong_msg.init2();
//         out._routing_id_msg.init2();
//
//         if out._options.heartbeat_interval > 0 {
//             out._heartbeat_timeout = out._options.heartbeat_timeout;
//             if out._heartbeat_timeout == -1 {
//                 out._heartbeat = out._options.heartbeat_interval;
//             }
//         }
//
//         out
//     }
// }

pub fn zmtp_plug_internal(options: &ZmqOptions, engine: &mut ZmqEngine) -> Result<(),ZmqError> {
    // start optional timer, to prevent handshake hanging on no input
    engine.set_handshake_timer();

    //  Send the 'length' and 'flags' fields of the routing id message.
    //  The 'length' field is encoded in the long format.
    engine.out_pos = &mut engine.greeting_send;
    engine.out_pos[engine.out_size] = u8::MAX;
    engine.out_size += 1;
    put_u64(
        &mut engine.out_pos[engine.out_size..],
        (options.routing_id_size + 1) as u64,
    );
    engine.out_size += 8;
    engine.out_pos[engine.out_size += 1] = 0x7f;

    engine.set_pollin();
    engine.set_pollout();
    //  Flush all the data that may have been already received downstream.
    engine.in_event(options);

    Ok(())
}

pub fn zmtp_handshake(engine: &mut ZmqEngine) -> bool {
    // zmq_assert (_greeting_bytes_read < _greeting_size);
    //  Receive the greeting.
    let rc = engine.receive_greeting();
    if rc == -1 {
        return false;
    }
    let unversioned = rc != 0;

    if !(engine.select_handshake_fun(
        unversioned,
        engine.greeting_recv[REVISION_POS],
        engine.greeting_recv[MINOR_POS],
    ))() {
        return false;
    }

    // Start polling for output if necessary.
    if engine.out_size == 0 {
        engine.set_pollout();
    }

    return true;
}

pub fn zmtp_receive_greeting(engine: &mut ZmqEngine) -> Result<(),ZmqError> {
    let mut unversioned = false;
    while engine.greeting_bytes_read < engine.greeting_size as u32 {
        let mut n = stream_read(
            engine,
            engine.greeting_recv[engine.greeting_bytes_read..],
            engine.greeting_size - engine.greeting_bytes_read,
        )?;
        if n == -1 {
            if get_errno() != EAGAIN {
                // Error(ConnectionError);
            }
            return Err(EngineError("stream read failed"));
        }

        engine.greeting_bytes_read += n;

        //  We have received at least one byte from the peer.
        //  If the first byte is not 0xff, we know that the
        //  peer is using unversioned protocol.
        if engine.greeting_recv[0] != 0xff {
            unversioned = true;
            break;
        }

        if engine.greeting_bytes_read < SIGNATURE_SIZE as u32 {
            continue;
        }

        //  Inspect the right-most bit of the 10th byte (which coincides
        //  with the 'flags' field if a regular message was sent).
        //  Zero indicates this is a header of a routing id message
        //  (i.e. the peer is using the unversioned protocol).
        if !(engine.greeting_recv[9] & 0x01) {
            unversioned = true;
            break;
        }

        //  The peer is using versioned protocol.
        engine.receive_greeting_versioned();
    }
    return if unversioned { Ok(()) } else { Err(EngineError("zmtp_receive_greeting failed")) };
}

pub fn zmtp_receive_greeting_versioned(options: &ZmqOptions, engine: &mut ZmqEngine) {
    //  Send the major version number.
    if engine.out_pos[engine.out_size..] == engine.greeting_send[SIGNATURE_SIZE..] {
        if engine.out_size == 0 {
            engine.set_pollout();
        }
        engine.out_pos[engine.out_size += 1] = 3; //  Major version number
    }

    if engine.greeting_bytes_read > SIGNATURE_SIZE as u32 {
        if engine.out_pos[engine.out_size..] == engine.greeting_send[SIGNATURE_SIZE + 1..] {
            if engine.out_size == 0 {
                engine.set_pollout();
            }

            //  Use ZMTP/2.0 to talk to older peers.
            if engine.greeting_recv[REVISION_POS] == ZMTP_1_0 as u8 || engine.greeting_recv[REVISION_POS] == ZMTP_2_0 as u8 {
                engine.out_pos[engine.out_size] = options.socket_type as u8;
                engine.out_size += 1;
            } else {
                engine.out_pos[engine.out_size] = 1; //  Minor version number
                engine.out_size += 1;
                // libc::memset (engine.out_pos[engine.out_size..], 0, 20);
                engine.out_pos[engine.out_size..].copy_from_slice(&[0; 20]);

                // zmq_assert (_options.mechanism == ZMQ_NULL
                //             || _options.mechanism == ZMQ_PLAIN
                //             || _options.mechanism == ZMQ_CURVE
                //             || _options.mechanism == ZMQ_GSSAPI);

                if options.mechanism == ZMQ_NULL {
                    // libc::memcpy(engine.out_pos + engine.out_size, "NULL", 4);
                    engine.out_pos[engine.out_size] = 'N' as u8;
                    engine.out_pos[engine.out_size + 1] = 'U' as u8;
                    engine.out_pos[engine.out_size + 2] = 'L' as u8;
                    engine.out_pos[engine.out_size + 3] = 'L' as u8;
                } else if options.mechanism == ZMQ_PLAIN {
                    // libc::memcpy(engine.out_pos + engine.out_size, "PLAIN", 5);
                    engine.out_pos[engine.out_size..].copy_from_slice(b"PLAIN");
                } else if options.mechanism == ZMQ_GSSAPI {
                    // libc::memcpy(engine.out_pos + engine.out_size, "GSSAPI", 6);
                    engine.out_pos[engine.out_size..].copy_from_slice(b"GSSAPI");
                } else if options.mechanism == ZMQ_CURVE {
                    // libc::memcpy(engine.out_pos + engine.out_size, "CURVE", 5);
                    engine.out_pos[engine.out_size..].copy_from_slice(b"CURVE");
                }
                engine.out_size += 20;
                // libc::memset (engine.out_pos + engine.out_size, 0, 32);
                engine.out_pos[engine.out_size..].copy_from_slice(&[0; 32]);
                engine.out_size += 32;
                engine.greeting_size = V3_GREETING_SIZE;
            }
        }
    }
}

// zmq::zmtp_engine_t::handshake_fun_t zmq::zmtp_engine_t::select_handshake_fun (
//     bool unversioned_, unsigned char revision_, unsigned char minor_)
pub fn zmtp_select_handshake_fun(
    engine: &mut ZmqEngine,
    unversioned_: bool,
    revision_: u8,
    minor_: u8,
) -> fn(&ZmqOptions, &mut ZmqEngine) -> bool {
    //  Is the peer using ZMTP/1.0 with no revision number?
    if unversioned_ {
        return zmtp_handshake_v1_0_unversioned;
    }
    return match (revision_) {
        ZMTP_1_0 => zmtp_handshake_v1_0,
        ZMTP_2_0 => zmtp_handshake_v2_0,
        ZMTP_3_x => match (minor_) {
            0 => zmtp_handshake_v3_0,
            _ => zmtp_handshake_v3_1,
        },
        _ => zmtp_handshake_v3_1,
    };
}

// bool zmq::zmtp_engine_t::handshake_v1_0_unversioned ()
pub fn zmtp_handshake_v1_0_unversioned(options: &ZmqOptions, engine: &mut ZmqEngine) -> bool {
    //  We send and receive rest of routing id message
    if engine.session().zap_enabled() {
        // reject ZMTP 1.0 connections if ZAP is enabled
        // Error (ProtocolError);
        return false;
    }

    // _encoder = new (std::nothrow) v1_encoder_t (_options.out_batch_size);
    engine.encoder = Some(ZmqEncoder::new(
        options.out_batch_size,
        EncoderType::V1Encoder,
    ));
    // alloc_assert (_encoder);

    // _decoder = new (std::nothrow)v1_decoder_t (_options.in_batch_size, _options.maxmsgsize);
    // alloc_assert (_decoder);
    engine.decoder = Some(ZmqDecoder::new(
        options.in_batch_size as usize,
        DecoderType::V1Decoder,
    ));

    //  We have already sent the message header.
    //  Since there is no way to tell the ENCODER to
    //  skip the message header, we simply throw that
    //  header data away.
    let header_size = if options.routing_id_size + 1 >= u8::MAX {
        10
    } else {
        2
    };
    // unsigned char tmp[10], *bufferp = tmp;
    let mut tmp: [u8; 10] = [0; 10];
    let mut bufferp = &mut tmp;

    //  Prepare the routing id message and load it into ENCODER.
    //  Then consume bytes we have already sent to the peer.
    let mut rc = engine.routing_id_msg.close();
    // zmq_assert (rc == 0);
    rc = engine.routing_id_msg.init_size(options.routing_id_size as usize);
    // zmq_assert (rc == 0);
    // libc::memcpy (engine.routing_id_msg.data_mut(), options.routing_id,
    //               options.routing_id_size);
    engine.routing_id_msg.data_mut().copy_from_slice(&options.routing_id);
    engine.encoder.load_msg(&engine.routing_id_msg);
    let buffer_size = engine.encoder.unwrap().encode(bufferp, header_size);
    // zmq_assert (buffer_size == header_size);

    //  Make sure the DECODER sees the data we have already received.
    engine.in_pos = &mut engine.greeting_recv;
    engine.in_size = engine.greeting_bytes_read as usize;

    //  To allow for interoperability with peers that do not forward
    //  their subscriptions, we inject a phantom subscription message
    //  message into the incoming message stream.
    if options.socket_type == ZMQ_PUB || options.socket_type == ZMQ_XPUB {
        engine.subscription_required = true;
    }

    //  We are sending our routing id now and the next message
    //  will come from the socket.
    engine.next_msg = stream_pull_msg_from_session;

    //  We are expecting routing id message.
    // _process_msg = static_cast<int (stream_engine_base_t::*) (msg_t *)> (
    //   &zmtp_engine_t::process_routing_id_msg);
    engine.process_msg = zmtp_process_routing_id_msg;

    return true;
}

// bool zmq::zmtp_engine_t::handshake_v1_0 ()
pub fn zmtp_handshake_v1_0(options: &ZmqOptions, engine: &mut ZmqEngine) -> bool {
    if engine.session().zap_enabled() {
        // reject ZMTP 1.0 connections if ZAP is enabled
        // Error (ProtocolError);
        return false;
    }

    // _encoder = new (std::nothrow) v1_encoder_t (_options.out_batch_size);
    // alloc_assert (_encoder);
    engine.encoder = Some(ZmqEncoder::new(
        options.out_batch_size,
        EncoderType::V1Encoder,
    ));

    // _decoder = new (std::nothrow)
    //   v1_decoder_t (_options.in_batch_size, _options.maxmsgsize);
    // alloc_assert (_decoder);
    engine.decoder = Some(ZmqDecoder::new(
        options.in_batch_size as usize,
        DecoderType::V1Decoder,
    ));

    return true;
}

// bool zmq::zmtp_engine_t::handshake_v2_0 ()
pub fn zmtp_handshake_v2_0(options: &ZmqOptions, engine: &mut ZmqEngine) -> bool {
    if (engine.session().zap_enabled()) {
        // reject ZMTP 2.0 connections if ZAP is enabled
        // Error (ProtocolError);
        return false;
    }

    // _encoder = new (std::nothrow) v2_encoder_t (_options.out_batch_size);
    // alloc_assert (_encoder);
    engine.encoder = Some(ZmqEncoder::new(
        options.out_batch_size,
        EncoderType::V2Encoder,
    ));

    // _decoder = new (std::nothrow) v2_decoder_t (
    //   _options.in_batch_size, _options.maxmsgsize, _options.zero_copy);
    // alloc_assert (_decoder);
    engine.decoder = Some(ZmqDecoder::new(
        options.in_batch_size as usize,
        DecoderType::V2Decoder,
    ));

    return true;
}

// bool zmq::zmtp_engine_t::handshake_v3_x (const bool downgrade_sub_)
pub fn zmtp_handshake_v3_x(
    options: &ZmqOptions,
    engine: &mut ZmqEngine,
    downgrade_sub_: bool,
) -> bool {
    let cmp_result = engine.greeting_recv[REVISION_POS..].eq(b"NULL\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0");
    if options.mechanism == ZMQ_NULL && cmp_result {
        // _mechanism = new (std::nothrow)
        //   null_mechanism_t (session (), _peer_address, _options);
        // alloc_assert (_mechanism);
        engine.mechanism = Some(ZmqMechanism::new(engine.session()));
    }
    //     else if (_options.mechanism == ZMQ_PLAIN
    //                && memcmp (_greeting_recv + 12,
    //                           "PLAIN\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0", 20)
    //                     == 0) {
    //         if (_options.as_server)
    //             _mechanism = new (std::nothrow)
    //               plain_server_t (session (), _peer_address, _options);
    //         else
    //             _mechanism =
    //               new (std::nothrow) plain_client_t (session (), _options);
    //         alloc_assert (_mechanism);
    //     }
    // #ifdef ZMQ_HAVE_CURVE
    //     else if (_options.mechanism == ZMQ_CURVE
    //              && memcmp (_greeting_recv + 12,
    //                         "CURVE\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0", 20)
    //                   == 0) {
    //         if (_options.as_server)
    //             _mechanism = new (std::nothrow) curve_server_t (
    //               session (), _peer_address, _options, downgrade_sub_);
    //         else
    //             _mechanism = new (std::nothrow)
    //               curve_client_t (session (), _options, downgrade_sub_);
    //         alloc_assert (_mechanism);
    //     }
    // #endif
    // #ifdef HAVE_LIBGSSAPI_KRB5
    //     else if (_options.mechanism == ZMQ_GSSAPI
    //              && memcmp (_greeting_recv + 12,
    //                         "GSSAPI\0\0\0\0\0\0\0\0\0\0\0\0\0\0", 20)
    //                   == 0) {
    //         if (_options.as_server)
    //             _mechanism = new (std::nothrow)
    //               gssapi_server_t (session (), _peer_address, _options);
    //         else
    //             _mechanism =
    //               new (std::nothrow) gssapi_client_t (session (), _options);
    //         alloc_assert (_mechanism);
    //     }
    // #endif
    else {
        engine.socket().event_handshake_failed_protocol(
            engine.session().get_endpoint(),
            ZMQ_PROTOCOL_ERROR_ZMTP_MECHANISM_MISMATCH,
        );
        // Error (ProtocolError);
        return false;
    }
    engine.next_msg = stream_next_handshake_command;
    engine.process_msg = stream_process_handshake_command;

    return true;
}

// bool zmq::zmtp_engine_t::handshake_v3_0 ()
pub fn zmtp_handshake_v3_0(options: &ZmqOptions, engine: &mut ZmqEngine) -> bool {
    // _encoder = new (std::nothrow) v2_encoder_t (_options.out_batch_size);
    // alloc_assert (_encoder);
    engine.encoder = Some(ZmqEncoder::new(
        options.out_batch_size,
        EncoderType::V2Encoder,
    ));

    // _decoder = new (std::nothrow) v2_decoder_t (
    //   _options.in_batch_size, _options.maxmsgsize, _options.zero_copy);
    // alloc_assert (_decoder);
    engine.decoder = Some(ZmqDecoder::new(
        options.in_batch_size as usize,
        DecoderType::V2Decoder,
    ));

    return engine.handshake_v3_x(true);
}

pub fn zmtp_handshake_v3_1(options: &ZmqOptions, engine: &mut ZmqEngine) -> bool {
    // _encoder = new (std::nothrow) v3_1_encoder_t (_options.out_batch_size);
    // alloc_assert (_encoder);
    engine.encoder = Some(ZmqEncoder::new(
        options.out_batch_size,
        EncoderType::V31Encoder,
    ));

    // _decoder = new (std::nothrow) v2_decoder_t (
    //   _options.in_batch_size, _options.maxmsgsize, _options.zero_copy);
    // alloc_assert (_decoder);
    engine.decoder = Some(ZmqDecoder::new(
        options.in_batch_size as usize,
        DecoderType::V2Decoder,
    ));

    return engine.handshake_v3_x(false);
}

// int zmq::zmtp_engine_t::routing_id_msg (msg_t *msg_)
pub fn zmtp_routing_id_msg(
    options: &ZmqOptions,
    engine: &mut ZmqEngine,
    msg: &mut ZmqMsg,
) -> Result<(),ZmqError> {
    msg.init_size(options.routing_id_size as usize)?;
    // errno_assert (rc == 0);
    if options.routing_id_size > 0 {
        // libc::memcpy(msg.data_mut(), options.routing_id, options.routing_id_size);
        msg.data_mut().copy_from_slice(&options.routing_id);
    }
    engine.next_msg = stream_pull_msg_from_session;
    Ok(())
}

// int zmq::zmtp_engine_t::process_routing_id_msg (msg_t *msg_)
pub fn zmtp_process_routing_id_msg(
    options: &ZmqOptions,
    engine: &mut ZmqEngine,
    msg: &mut ZmqMsg,
) -> Result<(), ZmqError> {
    if options.recv_routing_id {
        msg.set_flags(ZMQ_MSG_ROUTING_ID);
        let mut rc = engine.session().push_msg(msg);
        // errno_assert (rc == 0);
    } else {
        msg.close()?;
        // errno_assert (rc == 0);
        msg.init2()?;
        // errno_assert (rc == 0);
    }

    if engine.subscription_required {
        // msg_t subscription;
        let mut subscription: ZmqMsg;

        //  Inject the subscription message, so that also
        //  ZMQ 2.x peers receive published messages.
        let mut rc = subscription.init_size(1);
        // errno_assert (rc == 0);
        // *static_cast<unsigned char *> (subscription.data ()) = 1;
        subscription.data_mut()[0] = 1;
        rc = engine.session().push_msg(&subscription);
        // errno_assert (rc == 0);
    }

    engine.process_msg = stream_push_msg_to_session;

    Ok(())
}

// int zmq::zmtp_engine_t::produce_ping_message (msg_t *msg_)
pub fn zmtp_produce_ping_message(
    options: &ZmqOptions,
    engine: &mut ZmqEngine,
    msg: &mut ZmqMsg,
) -> Result<(), ZmqError> {
    // 16-bit TTL + \4PING == 7
    let ping_ttl_len = PING_CMD_NAME_SIZE + 2;
    // zmq_assert (_mechanism != NULL);

    msg.init_size(ping_ttl_len as usize)?;
    // errno_assert (rc == 0);
    msg.set_flags(ZmqMsg::command);
    // Copy in the command message
    // libc::memcpy (msg_.data_mut(), "\4PING", ZmqMsg::ping_cmd_name_size);
    msg.data_mut().copy_from_slice(b"\x04PING");

    let ttl_val = (options.heartbeat_ttl.to_be());
    // libc::memcpy ((msg_.data_mut()) + ZmqMsg::ping_cmd_name_size,
    //               &ttl_val, size_of::<ttl_val>());
    msg.data_mut()[PING_CMD_NAME_SIZE..].copy_from_slice(&ttl_val.to_be_bytes());

    engine.mechanism.unwrap().encode(msg)?;
    engine.next_msg = stream_pull_and_encode;
    if !engine.has_timeout_timer && engine.heartbeat_timeout > 0 {
        engine.add_timer(engine.heartbeat_timeout, HEARTBEAT_TIMEOUT_TIMER_ID);
        engine.has_timeout_timer = true;
    }
    Ok(())
}

// int zmq::zmtp_engine_t::produce_pong_message (msg_t *msg_)
pub fn zmtp_produce_pong_msg(engine: &mut ZmqEngine, msg_: &mut ZmqMsg) -> Result<(), ZmqError> {
    // zmq_assert (_mechanism != NULL);

    msg_.move_(&mut engine.pong_msg)?;
    // errno_assert (rc == 0);

    engine.mechanism.unwrap().encode(msg_)?;
    engine.next_msg = stream_pull_and_encode;
    Ok(())
}

// int zmq::zmtp_engine_t::process_heartbeat_message (msg_t *msg_)
pub fn zmtp_process_heartbeat_message(
    options: &ZmqOptions,
    engine: &mut ZmqEngine,
    msg_: &mut ZmqMsg,
) -> Result<(),ZmqError> {
    if msg_.is_ping() {
        // 16-bit TTL + \4PING == 7
        let ping_ttl_len = PING_CMD_NAME_SIZE + 2;
        let ping_max_ctx_len = 16;
        let mut remote_heartbeat_ttl = 0;

        // Get the remote heartbeat TTL to setup the timer
        // libc::memcpy (&remote_heartbeat_ttl,
        //               (msg_.data_mut())
        //           + PING_CMD_NAME_SIZE,
        //               ping_ttl_len - PING_CMD_NAME_SIZE);
        msg_.data_mut()[PING_CMD_NAME_SIZE..].copy_from_slice(&remote_heartbeat_ttl.to_be_bytes());

        remote_heartbeat_ttl = (remote_heartbeat_ttl.to_be());
        // The remote heartbeat is in 10ths of a second
        // so we multiply it by 100 to get the timer interval in ms.
        remote_heartbeat_ttl *= 100;

        if !engine.has_ttl_timer && remote_heartbeat_ttl > 0 {
            engine.add_timer(remote_heartbeat_ttl, HEARTBEAT_TTL_TIMER_ID);
            engine.has_ttl_timer = true;
        }

        //  As per ZMTP 3.1 the PING command might contain an up to 16 bytes
        //  context which needs to be PONGed back, so build the pong message
        //  here and store it. Truncate it if it's too long.
        //  Given the engine goes straight to out_event, sequential PINGs will
        //  not be a problem.
        let context_len = min(msg_.size() - ping_ttl_len, ping_max_ctx_len);
        let rc = engine.pong_msg.init_size(ZmqMsg::ping_cmd_name_size + context_len);
        // errno_assert (rc == 0);
        engine.pong_msg.set_flags(ZmqMsg::command);
        // libc::memcpy(
        //     engine.pong_msg.data_mut().as_mut_ptr() as *mut libc::c_void,
        //     "\x04PONG".as_bytes().as_ptr() as *const libc::c_void,
        //     ZmqMsg::ping_cmd_name_size,
        // );
        engine.pong_msg.data_mut()[..PING_CMD_NAME_SIZE].copy_from_slice(b"\x04PONG");
        if context_len > 0 {
            // libc::memcpy(engine.pong_msg.data_mut()) + PING_CMD_NAME_SIZE,
            // (msg_.data_mut()) + ping_ttl_len,
            // context_len);
            engine.pong_msg.data_mut()[PING_CMD_NAME_SIZE..].copy_from_slice(&msg_.data_mut()[ping_ttl_len..]);
        }

        engine.next_msg = zmtp_produce_pong_msg;
        engine.out_event(options);
    }

    Ok(())
}

// int zmq::zmtp_engine_t::process_command_message (msg_t *msg_)
pub fn zmtp_process_command_message(engine: &mut ZmqEngine, msg_: &mut ZmqMsg) -> Result<(),ZmqError> {
    let cmd_name_size = (msg_.data_mut())[0];
    let ping_name_size = PING_CMD_NAME_SIZE - 1;
    let sub_name_size = SUB_CMD_NAME_SIZE - 1;
    let cancel_name_size = CANCEL_CMD_NAME_SIZE - 1;
    //  Malformed command
    if msg_.size() < (cmd_name_size + size_of_val(cmd_name_size)) as usize {
        return Err(EngineError("Malformed command"));
    }

    let cmd_name = (msg_.data()[1..]);
    if cmd_name_size == ping_name_size as u8 && libc::memcmp(
        cmd_name,
        "PING".as_ptr() as *const libc::c_void,
        cmd_name_size as libc::size_t,
    ) == 0 {
        msg_.set_flags(ZMQ_MSG_PING);
    }
    if cmd_name_size == cmd_name_size as u8 && libc::memcmp(
        cmd_name,
        "PONG".as_ptr() as *const libc::c_void,
        cmd_name_size as libc::size_t,
    ) == 0 {
        msg_.set_flags(ZMQ_MSG_PONG);
    }
    if cmd_name_size == sub_name_size as u8 && libc::memcmp(
        cmd_name,
        "SUBSCRIBE".as_ptr() as *const libc::c_void,
        cmd_name_size as libc::size_t,
    ) == 0 {
        msg_.set_flags(ZMQ_MSG_SUBSCRIBE);
    }
    if cmd_name_size == cancel_name_size as u8 && libc::memcmp(
        cmd_name,
        "CANCEL".as_ptr() as *const libc::c_void,
        cmd_name_size as libc::size_t,
    ) == 0 {
        msg_.set_flags(ZMQ_MSG_CANCEL);
    }

    if msg_.is_ping() || msg_.is_pong() {
        return engine.process_heartbeat_message(msg_);
    }

    return Ok(());
}
