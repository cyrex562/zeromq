use crate::defines::fd_t;
use crate::endpoint::endpoint_uri_pair_t;
use crate::i_engine::error_reason_t;
use crate::metadata::metadata_t;
use crate::msg::msg_t;
use crate::options::options_t;
use crate::raw_decoder::raw_decoder_t;
use crate::raw_encoder::raw_encoder_t;

pub struct raw_engine_t {
    pub stream_engine_base: raw_engine_t,
}

impl raw_engine_t {
    pub fn new(fd_: fd_t, options_: &options_t, endpoint_uri_pair_: &endpoint_uri_pair_t) -> Self {
        Self {
            stream_engine_base: raw_engine_t::new(fd_, options_, endpoint_uri_pair_, false),
        }
    }

    pub unsafe fn plug_internal(&mut self)
    {
        // no handshaking for raw sock, instantiate raw encoder and decoders
        // _encoder = new (std::nothrow) raw_encoder_t (_options.out_batch_size);
        let mut _encoder = raw_encoder_t::new(self._options.out_batch_size);
        // alloc_assert (_encoder);

        // _decoder = new (std::nothrow) raw_decoder_t (_options.in_batch_size);
        let mut _decoder = raw_decoder_t::new(self._options.in_batch_size);
        // alloc_assert (_decoder);

        self._next_msg = self.pull_msg_from_session;
        self._process_msg = self.push_raw_msg_to_session;

        // properties_t properties;
        let mut properties = properties_t::new();
        if (self.stream_engine_base.init_properties (properties)) {
            //  Compile metadata.
            // zmq_assert (_metadata == NULL);
            self.stream_engine_base._metadata = metadata_t::new(properties);
            // alloc_assert (_metadata);
        }

        if (self._options.raw_notify) {
            //  For raw sockets, send an initial 0-length message to the
            // application so that it knows a peer has connected.
            // msg_t connector;
            let mut connector: msg_t = msg_t::default();
            connector.init2 ();
            self.push_raw_msg_to_session (&connector);
            connector.close ();
            self.stream_engine_base.session ().flush ();
        }

        self.stream_engine_base.set_pollin ();
        self.stream_engine_base.set_pollout ();
        //  Flush all the data that may have been already received downstream.
        self.stream_engine_base.in_event ();
    }

    pub unsafe fn handshake(&mut self) -> bool {
        true
    }

    pub unsafe fn error(&mut self, reason_: error_reason_t) {
        if (self._options.raw_socket && self._options.raw_notify) {
            //  For raw sockets, send a final 0-length message to the application
            //  so that it knows the peer has been disconnected.
            // msg_t terminator;
            let mut terminator = msg_t::new();
            terminator.init ();
            self.push_raw_msg_to_session (&terminator);
            terminator.close ();
        }
        self.stream_engine_base.error (reason_);
    }

    pub unsafe fn push_raw_msg_to_session(&mut self, msg_: &mut msg_t) -> i32 {
        if (self._metadata && self._metadata != msg_.metadata ()){
            msg_.set_metadata(self._metadata);
        }
        return self.stream_engine_base.push_msg_to_session (msg_);
    }
}