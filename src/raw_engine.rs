use std::collections::{HashMap};
use crate::engine::ZmqEngine;
use crate::err::ZmqError;
use crate::metadata::ZmqMetadata;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::raw_decoder::ZmqRawDecoder;
use crate::raw_encoder::ZmqRawEncoder;
use crate::stream_engine::stream_pull_msg_from_session;

// pub struct ZmqRawEngine {
//     pub stream_engine_base: ZmqRawEngine,
// }

// impl ZmqRawEngine {
//     pub fn new(fd_: ZmqFd, options_: &ZmqOptions, endpoint_uri_pair_: &ZmqEndpointUriPair) -> Self {
//         Self {
//             stream_engine_base: ZmqRawEngine::new(fd_, options_, endpoint_uri_pair_, false),
//         }
//     }

pub fn raw_plug_internal(options: &ZmqOptions, engine: &mut ZmqEngine) {
    // no Handshaking for raw sock, instantiate raw encoder and decoders
    // _encoder = new (std::nothrow) raw_encoder_t (_options.out_batch_size);
    let mut _encoder = ZmqRawEncoder::new(options.out_batch_size);
    // alloc_assert (_encoder);

    // _decoder = new (std::nothrow) raw_decoder_t (_options.in_batch_size);
    let mut _decoder = ZmqRawDecoder::new(options.in_batch_size as usize);
    // alloc_assert (_decoder);

    engine.next_msg = stream_pull_msg_from_session;
    engine.process_msg = raw_push_raw_msg_to_session;

    // properties_t properties;
    let mut properties: HashMap<String, String> = HashMap::new();
    if (engine.init_properties(properties)) {
        //  Compile metadata.
        // zmq_assert (_metadata == NULL);
        engine.metadata = Some(ZmqMetadata::new(&mut properties));
        // alloc_assert (_metadata);
    }

    if (options.raw_notify) {
        //  For raw sockets, send an initial 0-length message to the
        // application so that it knows a peer has connected.
        // msg_t connector;
        let mut connector: ZmqMsg = ZmqMsg::default();
        connector.init2();
        raw_push_raw_msg_to_session(engine, &mut connector);
        connector.close();
        engine.session().flush();
    }

    engine.set_pollin();
    engine.set_pollout();
    //  Flush all the data that may have been already received downstream.
    engine.in_event();
}

pub fn raw_handshake(engine: &mut ZmqEngine) -> bool {
    true
}

pub fn raw_error(options: &ZmqOptions, engine: &mut ZmqEngine, reason_: ErrorReason) {
    if options.raw_socket && options.raw_notify {
        //  For raw sockets, send a final 0-length message to the application
        //  so that it knows the peer has been disconnected.
        // msg_t terminator;
        let mut terminator = ZmqMsg::new();
        terminator.init();
        engine.push_raw_msg_to_session(&terminator);
        terminator.close();
    }
    engine.error(reason_);
}

pub fn raw_push_raw_msg_to_session(engine: &mut ZmqEngine, msg_: &mut ZmqMsg) -> Result<(), ZmqError> {
    if engine.metadata.is_some() && engine.metadata.unwrap() != *msg_.metadata() {
        msg_.set_metadata(&mut engine.metadata.unwrap());
    }
    return engine.push_msg_to_session(msg_);
}
// }
