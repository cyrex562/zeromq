use std::collections::HashMap;
use libc::{EAGAIN, ECONNRESET};
use crate::defines::{MSG_COMMAND, ZmqFd, ZMQ_NOTIFY_CONNECT, ZMQ_MSG_PROPERTY_PEER_ADDRESS};
use crate::engine::{ZmqEngine, ZmqEngineType};
use crate::engine::raw_engine::raw_plug_internal;
use crate::engine::zmtp_engine::{zmtp_plug_internal, zmtp_produce_ping_message};
use crate::err::ZmqError;
use crate::err::ZmqError::EngineError;
use crate::io::io_thread::ZmqIoThread;
use crate::ip::get_peer_ip_address;
use crate::mechanism::ZmqMechanism;
use crate::metadata::ZmqMetadata;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::session::ZmqSession;
use crate::tcp::{tcp_read, tcp_write};
use crate::utils::get_errno;

pub const HANDSHAKE_TIMER_ID: i32 = 0x40;
pub const HEARTBEAT_IVL_TIMER_ID: i32 = 0x80;
pub const HEARTBEAT_TIMEOUT_TIMER_ID: i32 = 0x81;
pub const HEARTBEAT_TTL_TIMER_ID: i32 = 0x82;

pub fn get_peer_address(s_: ZmqFd) -> String {
    let mut peer_address: String = String::new();
    let family = get_peer_ip_address(s_, &mut peer_address);
    if family == 0 {
        peer_address.clear();
    }

    peer_address
}

// pub struct ZmqStreamEngineBase<'a> {
//     // pub engine: dyn i_engine,
//     pub _decoder: Option<&'a mut ZmqDecoder>,
//     pub _endpoint_uri_pair: ZmqEndpointUriPair,
//     pub _handle: ZmqHandle,
//     pub _handshaking: bool,
//     pub _has_handshake_stage: bool,
//     pub _has_handshake_timer: bool,
//     pub _has_heartbeat_timer: bool,
//     pub _has_timeout_timer: bool,
//     pub _has_ttl_timer: bool,
//     pub _inpos: *mut u8,
//     pub _input_stopped: bool,
//     pub _insize: usize,
//     pub _io_error: bool,
//     pub _mechanism: Option<&'a mut ZmqMechanism>,
//     pub _metadata: Option<&'a mut ZmqMetadata>,
//     pub _options: ZmqOptions,
//     pub _output_stopped: bool,
//     pub _peer_address: String,
//     pub _plugged: bool,
//     pub _s: ZmqFd,
//     pub _session: Option<&'a mut ZmqSession<'a>>,
//     pub _socket: Option<&'a mut ZmqSocket<'a>>,
//     pub _tx_msg: ZmqMsg<'a>,
//     pub io_object: IoObject,
// }

// impl ZmqStreamEngineBase {
//     pub fn new(
//         fd_: ZmqFd,
//         options: &ZmqOptions,
//         endpoint_uri_pair: &ZmqEndpointUriPair,
//         has_handshake_stage: bool,
//     ) -> Self {
//         let mut out = Self {
//             io_object: IoObject::new(),
//             _options: options,
//             _inpos: null_mut(),
//             _insize: 0,
//             _decoder: None,
//             _mechanism: None,
//             _metadata: None,
//             _input_stopped: false,
//             _output_stopped: false,
//             _endpoint_uri_pair: endpoint_uri_pair,
//             _has_handshake_timer: false,
//             _has_ttl_timer: false,
//             _has_timeout_timer: false,
//             _has_heartbeat_timer: false,
//             _peer_address: "".to_string(),
//             _s: fd_,
//             _handle: null_mut(),
//             _plugged: false,
//             _tx_msg: ZmqMsg::new(),
//             _io_error: false,
//             _handshaking: false,
//             _session: None,
//             _socket: None,
//             _has_handshake_stage: false,
//         };
//
//         out._tx_msg.init2();
//
//         out
//     }
// }

pub fn stream_plug(options: &ZmqOptions, engine: &mut ZmqEngine, io_thread_: &mut ZmqIoThread, session_: &mut ZmqSession) -> Result<(), ZmqError> {
    engine.plugged = true;
    engine.session = Some(session_);
    engine.socket = Some(session_.socket());
    engine.io_object.plug(io_thread_);
    engine.handle = engine.add_fd(engine.s);
    engine.io_error = false;

    match engine.engine_type {
        ZmqEngineType::Stream => {
            // stream_plug_internal(engine);
        }
        ZmqEngineType::Udp => {
            // udp_plug(engine);
        }
        ZmqEngineType::Raw => {
            raw_plug_internal(options, engine)?;
        }
        ZmqEngineType::Zmtp => {
            zmtp_plug_internal(options, engine);
        }
    }

    engine.plug_internal()
}

pub fn stream_unplug(engine: &mut ZmqEngine) {
    engine.plugged = false;
    //  Cancel all timers.
    if engine.has_handshake_timer {
        engine.cancel_timer(HANDSHAKE_TIMER_ID);
        engine.has_handshake_timer = false;
    }

    if engine.has_ttl_timer {
        engine.cancel_timer(HEARTBEAT_TTL_TIMER_ID);
        engine.has_ttl_timer = false;
    }

    if engine.has_timeout_timer {
        engine.cancel_timer(HEARTBEAT_TIMEOUT_TIMER_ID);
        engine.has_timeout_timer = false;
    }

    if engine.has_heartbeat_timer {
        engine.cancel_timer(HEARTBEAT_IVL_TIMER_ID);
        engine.has_heartbeat_timer = false;
    }
    //  Cancel all fd subscriptions.
    if !engine.io_error {
        engine.rm_fd(engine.handle);
    }

    //  Disconnect from I/O threads poller object.
    engine.io_object.unplug();

    engine.session = None;
}

pub fn stream_terminate(engine: &mut ZmqEngine) {
    stream_unplug(engine);
}

pub fn stream_in_event(options: &ZmqOptions, engine: &mut ZmqEngine) {
    stream_in_event_internal(options, engine);
}

pub fn stream_in_event_internal(options: &ZmqOptions, engine: &mut ZmqEngine) -> bool {
    //  If still Handshaking, receive and process the greeting message.
    if engine.handshaking {
        if engine.handshake() {
            //  Handshaking was successful.
            //  Switch into the normal message flow.
            engine.handshaking = false;

            if engine.mechanism.is_none() && engine.has_handshake_stage {
                engine.session.engine_ready();

                if engine.has_handshake_timer {
                    engine.io_object.cancel_timer(HANDSHAKE_TIMER_ID);
                    engine.has_handshake_timer = false;
                }
            }
        } else {
            return false;
        }
    }


    // zmq_assert (_decoder);

    //  If there has been an I/O Error, Stop polling.
    if engine.input_stopped {
        engine.rm_fd(engine.handle);
        engine.io_error = true;
        return true; // TODO or return false in this case too?
    }

    //  If there's no data to process in the buffer...
    if !engine.in_size {
        //  Retrieve the buffer and read as much data as possible.
        //  Note that buffer can be arbitrarily large. However, we assume
        //  the underlying TCP layer has fixed buffer size and thus the
        //  number of bytes read will be always limited.
        let mut bufsize = 0usize;
        engine.decoder.get_buffer(engine: &mut ZmqEngine, engine.in_pos, &mut bufsize);

        let mut rc = stream_read(engine, engine.in_pos, bufsize);

        if rc == -1 {
            if get_errno() != EAGAIN {
                // Error (ConnectionError);
                return false;
            }
            return true;
        }

        //  Adjust input size
        engine.in_size = (rc) as usize;
        // Adjust buffer size to received bytes
        engine.decoder.resize_buffer(engine.in_size);
    }

    let mut rc = 0i32;
    let mut processed = 0usize;

    while engine.in_size > 0 {
        rc = engine.decoder.decode(engine.in_pos, engine.in_size, processed);
        // zmq_assert (processed <= _insize);
        engine.in_pos = engine.in_pos.add(processed);
        engine.in_size -= processed;
        if rc == 0 || rc == -1 {
            break;
        }
        if (engine.process_msg)(options, engine, engine.decoder.msg()).is_err() {
            break;
        }
    }

    //  Tear down the connection if we have failed to decode input data
    //  or the session has rejected the message.
    if (rc == -1) {
        if (get_errno() != EAGAIN) {
            // Error (ProtocolError);
            return false;
        }
        engine.input_stopped = true;
        engine.reset_pollin(engine.handle);
    }

    engine.session.flush();
    return true;
}

pub fn stream_out_event(options: &ZmqOptions, engine: &mut ZmqEngine) {
    //  If write buffer is empty, try to read new data from the encoder.
    if (!engine.out_size) {
        //  Even when we Stop polling as soon as there is no
        //  data to send, the poller may invoke out_event one
        //  more time due to 'speculative write' optimisation.
        if ((engine.encoder.is_none())) {
            // zmq_assert (_handshaking);
            return;
        }

        engine.out_pos = &mut [0u8];
        engine.out_size = engine.encoder.unwrap().encode(engine.out_pos, 0);

        while engine.out_size < (options.out_batch_size) {
            if (engine.next_msg)(engine, &mut engine.tx_msg.unwrap()) == -1 {
                //  ws_engine can cause an engine Error and delete it, so
                //  bail out immediately to avoid use-after-free
                if get_errno() == ECONNRESET {
                    return;
                } else {
                    break;
                }
            }
            engine.encoder.load_msg(&engine.tx_msg);
            let mut bufptr = engine.out_pos[engine.out_size..];
            let n = engine.encoder.unwrap().encode(&mut bufptr, options.out_batch_size - engine.out_size);
            // zmq_assert (n > 0);
            if engine.out_pos == &[0u8] {
                engine.out_pos = &mut bufptr;
            }
            engine.out_size += n;
        }

        //  If there is no data to send, Stop polling for output.
        if engine.out_size == 0 {
            engine.output_stopped = true;
            engine.reset_pollout();
            return;
        }
    }

    //  If there are any data to write in write buffer, write as much as
    //  possible to the socket. Note that amount of data to write can be
    //  arbitrarily large. However, we assume that underlying TCP layer has
    //  limited transmission buffer and thus the actual number of bytes
    //  written should be reasonably modest.
    let nbytes = stream_write(engine, engine.out_pos, engine.out_size);

    //  IO Error has occurred. We Stop waiting for output events.
    //  The engine is not Terminated until we detect input Error;
    //  this is necessary to prevent losing incoming messages.
    if nbytes == -1 {
        engine.reset_pollout();
        return;
    }

    engine.out_pos = engine.out_pos[nbytes..];
    engine.out_size -= nbytes;

    //  If we are still Handshaking and there are no data
    //  to send, Stop polling for output.
    if engine.handshaking {
        if engine.out_size == 0 {
            engine.reset_pollout();
        }
    }
}

pub fn stream_restart_output(options: &ZmqOptions, engine: &mut ZmqEngine) {
    if engine.io_error {
        return;
    }

    if engine.output_stopped {
        engine.output_stopped = false;
        engine.set_pollout();
    }

    stream_out_event(options, engine);
}

pub unsafe fn stream_restart_input(options: &ZmqOptions, engine: &mut ZmqEngine) -> bool {
    let mut rc = 0;
    if (engine.process_msg)(options, engine, engine.decoder.msg()).is_err() {
        if get_errno() == EAGAIN {
            engine.session.flush();
        } else {
            // Error (ProtocolError);
            return false;
        }
        return true;
    }

    while engine.in_size > 0 {
        let mut processed = 0usize;
        engine.decoder.unwrap().decode(engine.in_pos, engine.in_size, &mut processed)?;
        // zmq_assert (processed <= _insize);
        engine.in_pos = engine.in_pos.add(processed);
        engine.in_size -= processed;
        if rc == 0 || rc == -1 {
            break;
        }
        (engine.process_msg)(options, engine, engine.decoder.msg())?;
    }

    if rc == -1 && get_errno() == EAGAIN {
        engine.session.flush();
    } else if engine.io_error {
        // Error (ConnectionError);
        return false;
    } else if rc == -1 {
        // Error (ProtocolError);
        return false;
    } else {
        engine.input_stopped = false;
        engine.set_pollin();
        engine.session.flush();

        //  Speculative read.
        if !engine.in_event_internal() {
            return false;
        }
    }

    return true;
}

pub fn stream_next_handshake_command(engine: &mut ZmqEngine, msg_: &mut ZmqMsg) -> Result<(), ZmqError> {
    if engine.mechanism.status() == ZmqMechanism::ready {
        engine.mechanism_ready();
        return engine.pull_and_encode(msg_);
    }
    if engine.mechanism.status() == ZmqMechanism::error {
        // errno = EPROTO;
        return Err(EngineError("failed to pull and encode message"));
    }
    let rc = engine.mechanism.unwrap().next_handshake_command(msg_);

    if rc == 0 {
        msg_.set_flags(MSG_COMMAND);
    }

    Ok(())
}

pub fn stream_process_handshake_command(options: &ZmqOptions, engine: &mut ZmqEngine, msg_: &mut ZmqMsg) -> Result<(), ZmqError> {
    let rc = engine.mechanism.process_handshake_command(msg_);
    if (rc == 0) {
        if engine.mechanism.status() == ZmqMechanism::ready {
            engine.mechanism_ready();
        } else if engine.mechanism.status() == ZmqMechanism::error {
            // errno = EPROTO;
            return Err(EngineError("failed to process handshake command"));
        }
        if engine.output_stopped {
            stream_restart_output(options, engine);
        }
    }

    return rc;
}

pub unsafe fn stream_zap_msg_available(options: &ZmqOptions, engine: &mut ZmqEngine) {
    let rc = engine.mechanism.zap_msg_available();
    if rc == -1 {
        // Error (ProtocolError);
        return;
    }
    if engine.input_stopped {
        if !engine.restart_input() {
            return;
        }
    }
    if engine.output_stopped {
        stream_restart_output(options, engine);
    }
}

// pub unsafe fn get_endpoint(engine: &mut ZmqEngine,) -> &ZmqEndpointUriPair {
//     &engine.endpoint_uri_pair
// }

pub unsafe fn stream_mechanism_ready(options: &ZmqOptions, engine: &mut ZmqEngine) {
    if options.heartbeat_interval > 0 && !engine.has_heartbeat_timer {
        engine.add_timer(options.heartbeat_interval, HEARTBEAT_IVL_TIMER_ID);
        engine.has_heartbeat_timer = true;
    }

    if engine.has_handshake_stage {
        engine.session.engine_ready();
    }

    let mut flush_session = false;

    if options.recv_routing_id {
        let mut routing_id = ZmqMsg::new();
        engine.mechanism.peer_routing_id(&routing_id);
        let rc = engine.session.push_msg(&routing_id);
        if rc == -1 && get_errno() == EAGAIN {
            // If the write is failing at this stage with
            // an EAGAIN the pipe must be being shut down,
            // so we can just bail out of the routing id set.
            return;
        }
        // errno_assert (rc == 0);
        flush_session = true;
    }

    if options.router_notify & ZMQ_NOTIFY_CONNECT {
        let mut connect_notification = ZmqMsg::new();
        connect_notification.init();
        let rc = engine.session.push_msg(&connect_notification);
        if rc == -1 && get_errno() == EAGAIN {
            // If the write is failing at this stage with
            // an EAGAIN the pipe must be being shut down,
            // so we can just bail out of the notification.
            return;
        }
        // errno_assert (rc == 0);
        flush_session = true;
    }

    if (flush_session) {
        engine.session.flush();
    }

    engine.next_msg = stream_pull_and_encode;
    engine.process_msg = stream_write_credential;

    //  Compile metadata.
    let mut properties: HashMap<String, String> = HashMap::new();
    engine.init_properties(properties);

    //  Add ZAP properties.
    let mut zap_properties = engine.mechanism.get_zap_properties();
    properties.insert(zap_properties.begin(), zap_properties.end());

    //  Add ZMTP properties.
    let mut zmtp_properties = engine.mechanism.get_zmtp_properties();
    properties.insert(zmtp_properties.begin(), zmtp_properties.end());

    // zmq_assert (_metadata == NULL);
    if (!properties.empty()) {
        engine.metadata = Some(ZmqMetadata::new(&properties));
        // alloc_assert (_metadata);
    }

    if (engine.has_handshake_timer) {
        engine.cancel_timer(HANDSHAKE_TIMER_ID);
        engine.has_handshake_timer = false;
    }

    engine.socket.event_handshake_succeeded(&engine.endpoint_uri_pair, 0);
}

pub fn stream_write_credential(options: &ZmqOptions, engine: &mut ZmqEngine, msg_: &mut ZmqMsg) -> Result<(), ZmqError> {
    let mut credential = engine.mechanism.get_user_id();
    if credential.size() > 0 {
        let mut msg = ZmqMsg::default();
        msg.init_size(credential.size())?;
        // zmq_assert (rc == 0);
        // TODO
        // libc::memcpy(msg.data_mut(), credential.data(), credential.size());
        msg.set_flags(ZmqMsg::credential);
        if engine.session.push_msg(&msg).is_err() {
            msg.close()?;
            // errno_assert (rc == 0);
            return Err(EngineError("failed to push message"));
        }
    }
    engine.process_msg = stream_decode_and_push;
    return engine.decode_and_push(msg_);
}

pub fn stream_pull_and_encode(
    engine: &mut ZmqEngine,
    msg_: &mut ZmqMsg
) -> Result<(), ZmqError> {
    if engine.session.pull_msg(msg_) == -1 {
        return Err(EngineError("failed to pull message"));
    }
    if engine.mechanism.unwrap().encode(msg_) == -1 {
        return Err(EngineError("failed to encode message"));
    }
    Ok(())
}

pub fn stream_decode_and_push(_options: &ZmqOptions, engine: &mut ZmqEngine, msg_: &mut ZmqMsg) -> Result<(), ZmqError> {
    if engine.mechanism.decode(msg_) == -1 {
        return Err(EngineError("failed to decode message"));
    }

    if engine.has_timeout_timer {
        engine.has_timeout_timer = false;
        engine.cancel_timer(HEARTBEAT_TIMEOUT_TIMER_ID);
    }

    if engine.has_ttl_timer {
        engine.has_ttl_timer = false;
        engine.cancel_timer(HEARTBEAT_TTL_TIMER_ID);
    }

    if msg_.flag_set(MSG_COMMAND) {
        engine.process_command_message(msg_);
    }

    if engine.metadata {
        msg_.set_metadata(&mut engine.metadata.unwrap());
    }
    if engine.session.push_msg(msg_) == -1 {
        if get_errno() == EAGAIN {
            engine.process_msg = stream_push_one_then_decode_and_push;
        }
        return Err(EngineError("failed to push message"));
    }
    return Ok(());
}

pub fn stream_push_one_then_decode_and_push(options: &ZmqOptions, engine: &mut ZmqEngine, msg_: &mut ZmqMsg) -> Result<(), ZmqError> {
    if engine.session.push_msg(msg_) == -1 {
        return Err(EngineError("failed to push msg"));
    }
    engine.process_msg = stream_decode_and_push;
    return stream_decode_and_push(options, engine, msg_);
}

pub fn stream_pull_msg_from_session(engine: &mut ZmqEngine, msg_: &mut ZmqMsg) -> Result<(), ZmqError> {
    engine.session.pull_msg(msg_)
}

pub fn stream_push_msg_to_session(_options: &ZmqOptions, engine: &mut ZmqEngine, msg: &mut ZmqMsg) -> Result<(), ZmqError> {
    engine.session.push_msg(msg)
}

pub fn stream_error(engine: &mut ZmqEngine, reason_: &str) {
    // if (ZMQ_ROUTER_NOTIFY & ZMQ_NOTIFY_DISCONNECT) && !engine.handshaking {
    //     // For router sockets with disconnect notification, rollback
    //     // any incomplete message in the pipe, and push the disconnect
    //     // notification message.
    //     engine.session.rollback();
    //
    //     let mut disconnect_notification = ZmqMsg::new();
    //     disconnect_notification.init();
    //     engine.session.push_msg(&disconnect_notification);
    // }
    //
    // // protocol errors have been signaled already at the point where they occurred
    // if (reason_ != ProtocolError && (engine.mechanism.is_none() || engine.mechanism.status() == ZmqMechanism::handshaking)) {
    //     let mut err = get_errno;
    //     engine.socket.event_handshake_failed_no_detail(engine.endpoint_uri_pair, err);
    //     // special case: connecting to non-ZMTP process which immediately drops connection,
    //     // or which never responds with greeting, should be treated as a protocol Error
    //     // (i.e. Stop reconnect)
    //     if (((reason_ == ConnectionError) || (reason_ == TimeoutError)) && (options.reconnect_stop & ZMQ_RECONNECT_STOP_HANDSHAKE_FAILED)) {
    //         reason_ = ProtocolError;
    //     }
    // }
    //
    // engine.socket.event_disconnected(&engine.endpoint_uri_pair, engine._s);
    // engine.session.flush();
    // engine.session.engine_error(
    //     !engine.handshaking && (engine.mechanism.is_none() || engine.mechanism.status() != ZmqMechanism::handshaking),
    //     reason_);
    // engine.unplug();
    // // delete this;
    todo!()
}

pub unsafe fn stream_set_handshake_timer(options: &ZmqOptions, engine: &mut ZmqEngine) {
    if (options.handshake_ivl > 0) {
        engine.add_timer(options.handshake_ivl, HANDSHAKE_TIMER_ID);
        engine.has_handshake_timer = true;
    }
}

pub unsafe fn stream_init_properties(engine: &mut ZmqEngine, properties_: &mut HashMap<String, String>) -> bool {
    if engine.peer_address.empty() {
        return false;
    }
    properties_.insert(
        ZMQ_MSG_PROPERTY_PEER_ADDRESS.to_string(), engine.peer_address.clone());

    //  Private property to support deprecated SRCFD
    // TODO
    // std::ostringstream stream;
    // stream << static_cast<int> (_s);
    // std::string fd_string = stream.str ();
    // properties_.insert (std::string ("__fd"),
    //                                        ZMQ_MOVE (fd_string));
    return true;
}

pub unsafe fn stream_timer_event(options: &ZmqOptions, engine: &mut ZmqEngine, id_: i32) {
    if id_ == HANDSHAKE_TIMER_ID {
        engine.has_handshake_timer = false;
        //  handshake timer expired before handshake completed, so engine fail
        // Error (TimeoutError);
    } else if id_ == HEARTBEAT_IVL_TIMER_ID {
        engine.next_msg = zmtp_produce_ping_message;
        stream_out_event(options, engine);
        engine.add_timer(options.heartbeat_interval, HEARTBEAT_IVL_TIMER_ID);
    } else if id_ == HEARTBEAT_TTL_TIMER_ID {
        engine.has_ttl_timer = false;
        // Error (TimeoutError);
    } else if id_ == HEARTBEAT_TIMEOUT_TIMER_ID {
        engine.has_timeout_timer = false;
        // Error (TimeoutError);
    } else {}
    // There are no other valid timer ids!
    // assert (false);
}

pub fn stream_read(engine: &mut ZmqEngine , data_: &mut [u8], size_: usize) -> i32 {
    let rc = tcp_read(engine.s, data_, size_);
    if (rc == 0) {
        // connection closed by peer
        // errno = EPIPE;
        return -1;
    }

    return rc;
}

pub fn stream_write(engine: &mut ZmqEngine , data_: &mut [u8], size_: usize) -> i32 {
    tcp_write(engine.s, data_, size_)
}
