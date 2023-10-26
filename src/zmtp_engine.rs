use std::cmp::min;
use std::mem::size_of;
use libc::EAGAIN;
use crate::defines::{ZmqFd, ZMQ_CURVE, ZMQ_GSSAPI, ZMQ_NULL, ZMQ_PLAIN, ZMQ_PROTOCOL_ERROR_ZMTP_MECHANISM_MISMATCH, ZMQ_PUB, ZMQ_XPUB};
use crate::endpoint::ZmqEndpointUriPair;
use crate::msg::{MSG_CANCEL, ZmqMsg, MSG_PING, ping_cmd_name_size, MSG_PONG, MSG_ROUTING_ID, MSG_SUBSCRIBE};
use crate::null_mechanism::ZmqNullMechanism;
use crate::options::ZmqOptions;
use crate::stream_engine_base::{HEARTBEAT_TIMEOUT_TIMER_ID, HEARTBEAT_TTL_TIMER_ID, ZmqStreamEngineBase};
use crate::utils::{get_errno, put_u64};

pub const ZMTP_1_0: i32 = 0;
pub const ZMTP_2_0: i32 = 1;
pub const ZMTP_3_x: i32 = 3;

pub const SIGNATURE_SIZE: usize = 10;
pub const V2_GREETING_SIZE: usize = 12;

pub const V3_GREETING_SIZE: usize = 64;

pub const REVISION_POS: usize = 10;
pub const MINOR_POS: usize = 11;


pub struct ZmtpEngine<'a> {
    pub stream_engine_base: ZmqStreamEngineBase<'a>,
    pub _routing_id_msg: ZmqMsg,
    pub _pong_msg: ZmqMsg,
    pub _greeting_size: usize,
    pub _greeting_recv: [u8; V3_GREETING_SIZE],
    pub _greeting_send: [u8; V3_GREETING_SIZE],
    pub _greeting_bytes_read: u32,
    pub _subscription_required: bool,
    pub _heartbeat_timeout: i32,
}

impl ZmtpEngine {
    pub unsafe fn new(fd_: ZmqFd, options_: &ZmqOptions, endpoint_uri_pair_: &ZmqEndpointUriPair) -> Self
    {
        let mut out = Self {
            stream_engine_base: ZmqStreamEngineBase::new(fd_, options_, endpoint_uri_pair_, true),
            _routing_id_msg: ZmqMsg::default(),
            _pong_msg: ZmqMsg::default(),
            _greeting_size: V2_GREETING_SIZE,
            _greeting_recv: [0; V3_GREETING_SIZE],
            _greeting_send: [0; V3_GREETING_SIZE],
            _greeting_bytes_read: 0,
            _subscription_required: false,
            _heartbeat_timeout: 0,
        };

        out.stream_engine_base._next_msg = &mut out._routing_id_msg;
        out.stream_engine_base._process_msg = &mut out.process_routing_id_msg;
        out._pong_msg.init2();
        out._routing_id_msg.init2();

        if out._options.heartbeat_interval > 0 {
            out._heartbeat_timeout = out._options.heartbeat_timeout;
            if out._heartbeat_timeout == -1 {
                out._heartbeat = out._options.heartbeat_interval;
            }
        }

        out
    }

    pub unsafe fn plug_internal(&mut self)
    {
        // start optional timer, to prevent handshake hanging on no input
        self.set_handshake_timer ();

        //  Send the 'length' and 'flags' fields of the routing id message.
        //  The 'length' field is encoded in the long format.
        self._outpos = self._greeting_send;
        self._outpos[self._outsize] = u8::MAX;
        self._outsize += 1;
        put_u64 (&self._outpos[self._outsize], self._options.routing_id_size + 1);
        self._outsize += 8;
        self._outpos[self._outsize +=1] = 0x7f;

        self.set_pollin ();
        self.set_pollout ();
        //  Flush all the data that may have been already received downstream.
        self.in_event ();
    }

    pub unsafe fn handshake(&mut self) -> bool {
        // zmq_assert (_greeting_bytes_read < _greeting_size);
        //  Receive the greeting.
        let rc = self.receive_greeting ();
        if (rc == -1) {
            return false;
        }
        let unversioned = rc != 0;

        if !(self.select_handshake_fun (unversioned, self._greeting_recv[REVISION_POS],
                                        self._greeting_recv[MINOR_POS])) () {
            return false;
        }

        // Start polling for output if necessary.
        if (self._outsize == 0) {
            self.set_pollout();
        }

        return true;
    }

    pub unsafe fn receive_greeting(&mut self) -> i32 {
        let mut unversioned = false;
        while self._greeting_bytes_read < self._greeting_size as u32 {
            let mut n = self.read (self._greeting_recv + self._greeting_bytes_read,
                                self._greeting_size - self._greeting_bytes_read);
            if (n == -1) {
                if (get_errno() != EAGAIN) {
                    // Error(ConnectionError);
                }
                return -1;
            }

            self._greeting_bytes_read += n;

            //  We have received at least one byte from the peer.
            //  If the first byte is not 0xff, we know that the
            //  peer is using unversioned protocol.
            if (self._greeting_recv[0] != 0xff) {
                unversioned = true;
                break;
            }

            if (self._greeting_bytes_read < SIGNATURE_SIZE) {
                continue;
            }

            //  Inspect the right-most bit of the 10th byte (which coincides
            //  with the 'flags' field if a regular message was sent).
            //  Zero indicates this is a header of a routing id message
            //  (i.e. the peer is using the unversioned protocol).
            if (!(self._greeting_recv[9] & 0x01)) {
                unversioned = true;
                break;
            }

            //  The peer is using versioned protocol.
            self.receive_greeting_versioned ();
        }
        return if unversioned { 1 } else { 0 };
    }

    pub unsafe fn receive_greeting_versioned(&mut self) {
        //  Send the major version number.
        if (self._outpos + self._outsize == self._greeting_send + SIGNATURE_SIZE) {
            if (self._outsize == 0) {
                self.set_pollout();
            }
            self._outpos[self._outsize += 1] = 3; //  Major version number
        }

        if (self._greeting_bytes_read > SIGNATURE_SIZE) {
            if (self._outpos + self._outsize == self._greeting_send + SIGNATURE_SIZE + 1) {
                if (self._outsize == 0) {
                    self.set_pollout();
                }

                //  Use ZMTP/2.0 to talk to older peers.
                if (self._greeting_recv[REVISION_POS] == ZMTP_1_0
                    || self._greeting_recv[REVISION_POS] == ZMTP_2_0) {
                    self._outpos[self._outsize] = self._options.type_ ;
                    self._outsize += 1;
                }
                else {
                    self._outpos[self._outsize] = 1; //  Minor version number
                    self._outsize += 1;
                    libc::memset (self._outpos + self._outsize, 0, 20);

                    // zmq_assert (_options.mechanism == ZMQ_NULL
                    //             || _options.mechanism == ZMQ_PLAIN
                    //             || _options.mechanism == ZMQ_CURVE
                    //             || _options.mechanism == ZMQ_GSSAPI);

                    if (self._options.mechanism == ZMQ_NULL) {
                        libc::memcpy(self._outpos + self._outsize, "NULL", 4);
                    }
                    else if (self._options.mechanism == ZMQ_PLAIN) {
                        libc::memcpy(self._outpos + self._outsize, "PLAIN", 5);
                    }
                    else if (self._options.mechanism == ZMQ_GSSAPI) {
                        libc::memcpy(self._outpos + self._outsize, "GSSAPI", 6);
                    }
                    else if (self._options.mechanism == ZMQ_CURVE) {
                        libc::memcpy(self._outpos + self._outsize, "CURVE", 5);
                    }
                    self._outsize += 20;
                    libc::memset (self._outpos + self._outsize, 0, 32);
                    self._outsize += 32;
                    self._greeting_size = V3_GREETING_SIZE;
                }
            }
        }
    }

    // zmq::zmtp_engine_t::handshake_fun_t zmq::zmtp_engine_t::select_handshake_fun (
    //     bool unversioned_, unsigned char revision_, unsigned char minor_)
    pub fn select_handshake_fun(&mut self, unversioned_: bool, revision_: u8, minor_: u8) -> handshake_fun_t
    {
        //  Is the peer using ZMTP/1.0 with no revision number?
        if (unversioned_) {
            return &ZmtpEngine::handshake_v1_0_unversioned;
        }
        match (revision_) {
            ZMTP_1_0 =>
                return &ZmtpEngine::handshake_v1_0,
            ZMTP_2_0 =>
                return &ZmtpEngine::handshake_v2_0,
            ZMTP_3_x => {
                match (minor_) {
                    0 => return &ZmtpEngine::handshake_v3_0,
                    _ => {
                        return &ZmtpEngine::handshake_v3_1;
                    }
                }
            },
            _ =>
                return &ZmtpEngine::handshake_v3_1
        }
    }

    // bool zmq::zmtp_engine_t::handshake_v1_0_unversioned ()
    pub unsafe fn handshake_v1_0_unversioned(&mut self) -> bool
    {
        //  We send and receive rest of routing id message
        if (self.session ().zap_enabled ()) {
            // reject ZMTP 1.0 connections if ZAP is enabled
            // Error (ProtocolError);
            return false;
        }

        // _encoder = new (std::nothrow) v1_encoder_t (_options.out_batch_size);
        self._encoder = v1_encoder_t::new(_options.out_batch_size);
        // alloc_assert (_encoder);

        // _decoder = new (std::nothrow)v1_decoder_t (_options.in_batch_size, _options.maxmsgsize);
        // alloc_assert (_decoder);
        self._decoder = v1_decoder_t::new(_options.in_batch_size, _options.maxmsgsize);

        //  We have already sent the message header.
        //  Since there is no way to tell the encoder to
        //  skip the message header, we simply throw that
        //  header data away.
        let header_size = if self._options.routing_id_size + 1 >= u8::MAX { 10 } else { 2 };
        // unsigned char tmp[10], *bufferp = tmp;
        let mut tmp: [u8;10] = [0;10];
        let mut bufferp = &mut tmp[0];


        //  Prepare the routing id message and load it into encoder.
        //  Then consume bytes we have already sent to the peer.
        let mut rc = self._routing_id_msg.close ();
        // zmq_assert (rc == 0);
        rc = self._routing_id_msg.init_size (self._options.routing_id_size);
        // zmq_assert (rc == 0);
        libc::memcpy (self._routing_id_msg.data (), self._options.routing_id,
                self._options.routing_id_size);
        self._encoder.load_msg (&self._routing_id_msg);
        let buffer_size = self._encoder.encode (&bufferp, header_size);
        // zmq_assert (buffer_size == header_size);

        //  Make sure the decoder sees the data we have already received.
        self._inpos = self._greeting_recv;
        self._insize = self._greeting_bytes_read;

        //  To allow for interoperability with peers that do not forward
        //  their subscriptions, we inject a phantom subscription message
        //  message into the incoming message stream.
        if (self._options.type_ == ZMQ_PUB || self._options.type_ == ZMQ_XPUB) {
            self._subscription_required = true;
        }

        //  We are sending our routing id now and the next message
        //  will come from the socket.
        self._next_msg = &ZmtpEngine::pull_msg_from_session;

        //  We are expecting routing id message.
        // _process_msg = static_cast<int (stream_engine_base_t::*) (msg_t *)> (
        //   &zmtp_engine_t::process_routing_id_msg);
        self._process_msg = self.process_routing_id_msg;

        return true;
    }

    // bool zmq::zmtp_engine_t::handshake_v1_0 ()
    pub unsafe fn handshake_v1_0(&mut self) -> bool
    {
        if (self.session ().zap_enabled ()) {
            // reject ZMTP 1.0 connections if ZAP is enabled
            // Error (ProtocolError);
            return false;
        }

        // _encoder = new (std::nothrow) v1_encoder_t (_options.out_batch_size);
        // alloc_assert (_encoder);
        self._encoder = v1_encoder_t::new(self._options.out_batch_size);

        // _decoder = new (std::nothrow)
        //   v1_decoder_t (_options.in_batch_size, _options.maxmsgsize);
        // alloc_assert (_decoder);
        self._decoder = v1_decoder_t::new(self._options.in_batch_size, self._options.maxmsgsize);

        return true;
    }

    // bool zmq::zmtp_engine_t::handshake_v2_0 ()
    pub unsafe fn handshake_v2_0(&mut self) -> bool
    {
        if (self.session().zap_enabled ()) {
            // reject ZMTP 2.0 connections if ZAP is enabled
            // Error (ProtocolError);
            return false;
        }

        // _encoder = new (std::nothrow) v2_encoder_t (_options.out_batch_size);
        // alloc_assert (_encoder);
        self._encoder = v2_encoder_t::new(self._options.out_batch_size);

        // _decoder = new (std::nothrow) v2_decoder_t (
        //   _options.in_batch_size, _options.maxmsgsize, _options.zero_copy);
        // alloc_assert (_decoder);
        self._decoder = v2_decoder_t::new(self._options.in_batch_size, self._options.maxmsgsize, self._options.zero_copy);

        return true;
    }

    // bool zmq::zmtp_engine_t::handshake_v3_x (const bool downgrade_sub_)
    pub unsafe fn handshake_v3_x(&mut self, downgrade_sub_: bool) -> bool
    {
        if (self._options.mechanism == ZMQ_NULL
            && libc::memcmp (self._greeting_recv + 12, "NULL\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                       20)
                 == 0) {
            // _mechanism = new (std::nothrow)
            //   null_mechanism_t (session (), _peer_address, _options);
            // alloc_assert (_mechanism);
            self._mechanism = ZmqNullMechanism::new(self.session(), self._peer_address, self._options);
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
            self.socket ().event_handshake_failed_protocol (
              self.session ().get_endpoint (),
              ZMQ_PROTOCOL_ERROR_ZMTP_MECHANISM_MISMATCH);
            // Error (ProtocolError);
            return false;
        }
        self._next_msg = self.next_handshake_command;
        self._process_msg = self.process_handshake_command;

        return true;
    }

    // bool zmq::zmtp_engine_t::handshake_v3_0 ()
    pub unsafe fn handshake_v3_0(&mut self) -> bool
    {
        // _encoder = new (std::nothrow) v2_encoder_t (_options.out_batch_size);
        // alloc_assert (_encoder);
        self._encoder = v2_encoder_t::new(self._options.out_batch_size);

        // _decoder = new (std::nothrow) v2_decoder_t (
        //   _options.in_batch_size, _options.maxmsgsize, _options.zero_copy);
        // alloc_assert (_decoder);
        self._decoder = v2_decoder_t::new(self._options.in_batch_size, self._options.maxmsgsize, self._options.zero_copy);

        return self.handshake_v3_x (true);
    }

    pub unsafe fn handshake_v3_1(&mut self) -> bool {
        // _encoder = new (std::nothrow) v3_1_encoder_t (_options.out_batch_size);
        // alloc_assert (_encoder);
        self._encoder = v3_1_encoder_t::new(self._options.out_batch_size);

        // _decoder = new (std::nothrow) v2_decoder_t (
        //   _options.in_batch_size, _options.maxmsgsize, _options.zero_copy);
        // alloc_assert (_decoder);
        self._decoder = v2_decoder_t::new(self._options.in_batch_size, self._options.maxmsgsize, self._options.zero_copy);

        return self.handshake_v3_x (false);
    }

    // int zmq::zmtp_engine_t::routing_id_msg (msg_t *msg_)
    pub unsafe fn routing_id_msg(&mut self, msg_: &ZmqMsg) -> i32
    {
        let rc = msg_.init_size (self._options.routing_id_size);
        // errno_assert (rc == 0);
        if (self._options.routing_id_size > 0) {
            libc::memcpy(msg_.data(), self._options.routing_id, self._options.routing_id_size);
        }
        self._next_msg = &ZmtpEngine::pull_msg_from_session;
        return 0;
    }

    // int zmq::zmtp_engine_t::process_routing_id_msg (msg_t *msg_)
    pub unsafe fn process_routing_id_msg(&mut self, msg_: &mut ZmqMsg) -> i32
    {
        if (self._options.recv_routing_id) {
            msg_.set_flags (MSG_ROUTING_ID);
            let mut rc = self.session ().push_msg (msg_);
            // errno_assert (rc == 0);
        } else {
            let rc = msg_.close ();
            // errno_assert (rc == 0);
            rc = msg_.init ();
            // errno_assert (rc == 0);
        }

        if (self._subscription_required) {
            // msg_t subscription;
            let mut subscription: ZmqMsg;

            //  Inject the subscription message, so that also
            //  ZMQ 2.x peers receive published messages.
            let mut rc = subscription.init_size (1);
            // errno_assert (rc == 0);
            // *static_cast<unsigned char *> (subscription.data ()) = 1;
            subscription.data()[0] = 1;
            rc = self.session ().push_msg (&subscription);
            // errno_assert (rc == 0);
        }

        self._process_msg = self.push_msg_to_session;

        return 0;
    }

    // int zmq::zmtp_engine_t::produce_ping_message (msg_t *msg_)
    pub unsafe fn produce_ping_message(&mut self, msg_: &mut ZmqMsg) -> i32
    {
        // 16-bit TTL + \4PING == 7
        let ping_ttl_len = ping_cmd_name_size + 2;
        // zmq_assert (_mechanism != NULL);

        let rc = msg_.init_size (ping_ttl_len);
        // errno_assert (rc == 0);
        msg_.set_flags (ZmqMsg::command);
        // Copy in the command message
        libc::memcpy (msg_.data (), "\4PING", ZmqMsg::ping_cmd_name_size);

        let ttl_val = (self._options.heartbeat_ttl.to_be () as u16);
        libc::memcpy ((msg_.data ()) + ZmqMsg::ping_cmd_name_size,
                      &ttl_val, size_of::<ttl_val>());

        rc = self._mechanism.encode (msg_);
        self._next_msg = &ZmtpEngine::pull_and_encode;
        if (!self._has_timeout_timer && self._heartbeat_timeout > 0) {
            self.add_timer (self._heartbeat_timeout, HEARTBEAT_TIMEOUT_TIMER_ID);
            self._has_timeout_timer = true;
        }
        return rc;
    }


    // int zmq::zmtp_engine_t::produce_pong_message (msg_t *msg_)
    pub unsafe fn produce_pong_msg(&mut self, msg_: &mut ZmqMsg) -> i32
    {
        // zmq_assert (_mechanism != NULL);

        let mut rc = msg_.move_(self._pong_msg);
        // errno_assert (rc == 0);

        rc = self._mechanism.encode (msg_);
        self._next_msg = self.pull_and_encode;
        return rc;
    }

    // int zmq::zmtp_engine_t::process_heartbeat_message (msg_t *msg_)
    pub unsafe fn process_heartbeat_message(&mut self, msg_: &mut ZmqMsg) -> i32
    {
        if (msg_.is_ping ()) {
            // 16-bit TTL + \4PING == 7
            let ping_ttl_len = ping_cmd_name_size + 2;
            let ping_max_ctx_len = 16;
            let mut remote_heartbeat_ttl = 0;

            // Get the remote heartbeat TTL to setup the timer
            libc::memcpy (&remote_heartbeat_ttl,
                     (msg_.data ())
                      + ping_cmd_name_size,
                    ping_ttl_len - ping_cmd_name_size);
            remote_heartbeat_ttl = (remote_heartbeat_ttl.to_be());
            // The remote heartbeat is in 10ths of a second
            // so we multiply it by 100 to get the timer interval in ms.
            remote_heartbeat_ttl *= 100;

            if (!self._has_ttl_timer && remote_heartbeat_ttl > 0) {
                self.add_timer (remote_heartbeat_ttl, HEARTBEAT_TTL_TIMER_ID);
                self._has_ttl_timer = true;
            }

            //  As per ZMTP 3.1 the PING command might contain an up to 16 bytes
            //  context which needs to be PONGed back, so build the pong message
            //  here and store it. Truncate it if it's too long.
            //  Given the engine goes straight to out_event, sequential PINGs will
            //  not be a problem.
            let context_len =
              min(msg_.size () - ping_ttl_len, ping_max_ctx_len);
            let rc =
              self._pong_msg.init_size (ZmqMsg::ping_cmd_name_size + context_len);
            // errno_assert (rc == 0);
            self._pong_msg.set_flags (ZmqMsg::command);
            libc::memcpy (self._pong_msg.data (), "\4PONG", ZmqMsg::ping_cmd_name_size);
            if (context_len > 0)
                libc::memcpy (self._pong_msg.data ())
                          + ping_cmd_name_size,
                        (msg_.data ()) + ping_ttl_len,
                        context_len);

            self._next_msg = self.produce_pong_message;
            self.out_event ();
        }

        return 0;
    }

    // int zmq::zmtp_engine_t::process_command_message (msg_t *msg_)
    pub unsafe fn process_command_message(&mut self, msg_: &mut ZmqMsg) -> i32
    {
        let cmd_name_size = (msg_.data ()));
        let ping_name_size = ping_cmd_name_size - 1;
        let sub_name_size = sub_cmd_name_size - 1;
        let cancel_name_size = cancel_cmd_name_size - 1;
        //  Malformed command
        if ((msg_.size () < cmd_name_size + size_of::<cmd_name_size>())) {
            return -1;
        }

        let const cmd_name =
          (msg_.data()) + 1;
        if (cmd_name_size == ping_name_size
            && libc::memcmp (cmd_name, "PING", cmd_name_size) == 0) {
            msg_.set_flags(MSG_PING);
        }
        if (cmd_name_size == ping_name_size
            && libc::memcmp (cmd_name, "PONG", cmd_name_size) == 0) {
            msg_ .set_flags(MSG_PONG);
        }
        if (cmd_name_size == sub_name_size
            && libc::memcmp (cmd_name, "SUBSCRIBE", cmd_name_size) == 0) {
            msg_.set_flags(MSG_SUBSCRIBE);
        }
        if (cmd_name_size == cancel_name_size
            && libc::memcmp (cmd_name, "CANCEL", cmd_name_size) == 0) {
            msg_.set_flags(MSG_CANCEL);
        }

        if (msg_.is_ping () || msg_.is_pong ()){
        return self.process_heartbeat_message(msg_);
    }

        return 0;
    }

}
