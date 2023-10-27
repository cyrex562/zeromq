use std::ptr::null_mut;

use libc::{EAGAIN, ECONNRESET};

use crate::defines::{MSG_COMMAND, ZMQ_RECONNECT_STOP_HANDSHAKE_FAILED, ZmqFd, ZmqHandle};
use crate::endpoint::ZmqEndpointUriPair;
use crate::i_decoder::IDecoder;
use crate::i_engine::{ErrorReason, IEngine};
use crate::i_engine::ErrorReason::{ConnectionError, ProtocolError, TimeoutError};
use crate::io_object::IoObject;
use crate::io_thread::ZmqIoThread;
use crate::mechanism::ZmqMechanism;
use crate::metadata::ZmqMetadata;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use crate::session_base::ZmqSessionBase;
use crate::socket_base::ZmqSocketBase;
use crate::utils::get_errno;

pub const HANDSHAKE_TIMER_ID: i32 = 0x40;
pub const HEARTBEAT_IVL_TIMER_ID: i32 = 0x80;
pub const HEARTBEAT_TIMEOUT_TIMER_ID: i32 = 0x81;
pub const HEARTBEAT_TTL_TIMER_ID: i32 = 0x82;

pub fn get_peer_address(s_: ZmqFd) -> String {
    let mut peer_address: String = String::new();
    let family = zmq::get_peer_ip_address(s_, &mut peer_address);
    if family == 0 {
        peer_address.clear();
    }

    peer_address
}

pub struct ZmqStreamEngineBase<'a> {
    pub io_object: IoObject,
    // pub engine: dyn i_engine,
    pub _options: ZmqOptions,
    pub _inpos: *mut u8,
    pub _insize: usize,
    pub _decoder: Option<&'a mut dyn IDecoder>,
    pub _mechanism: Option<&'a mut ZmqMechanism>,
    pub _metadata: Option<&'a mut ZmqMetadata>,
    pub _input_stopped: bool,
    pub _output_stopped: bool,
    pub _endpoint_uri_pair: ZmqEndpointUriPair,
    pub _has_handshake_timer: bool,
    pub _has_ttl_timer: bool,
    pub _has_timeout_timer: bool,
    pub _has_heartbeat_timer: bool,
    pub _peer_address: String,
    pub _s: ZmqFd,
    pub _handle: ZmqHandle,
    pub _plugged: bool,
    pub _tx_msg: ZmqMsg,
    pub _io_error: bool,
    pub _handshaking: bool,
    pub _session: Option<&'a mut ZmqSessionBase<'a>>,
    pub _socket: Option<&'a mut ZmqSocketBase<'a>>,
    pub _has_handshake_stage: bool,
}

impl ZmqStreamEngineBase {
    pub fn new(
        fd_: ZmqFd,
        options: &ZmqOptions,
        endpoint_uri_pair: &ZmqEndpointUriPair,
        has_handshake_stage: bool,
    ) -> Self {
        let mut out = Self {
            io_object: IoObject::new(),
            _options: options,
            _inpos: null_mut(),
            _insize: 0,
            _decoder: None,
            _mechanism: None,
            _metadata: None,
            _input_stopped: false,
            _output_stopped: false,
            _endpoint_uri_pair: endpoint_uri_pair,
            _has_handshake_timer: false,
            _has_ttl_timer: false,
            _has_timeout_timer: false,
            _has_heartbeat_timer: false,
            _peer_address: "".to_string(),
            _s: fd_,
            _handle: null_mut(),
            _plugged: false,
            _tx_msg: ZmqMsg::new(),
            _io_error: false,
            _handshaking: false,
            _session: None,
            _socket: None,
            _has_handshake_stage: false,
        };

        out._tx_msg.init2();

        out
    }

    pub unsafe fn plug(&mut self, io_thread_: &mut ZmqIoThread, session_: &mut ZmqSessionBase) {
        self._plugged = true;
        self._session = Some(session_);
        self._socket = Some(session_.socket());
        self.io_object.plug(io_thread_);
        self._handle = self.add_fd(self._s);
        self._io_error = false;
        self.plug_internal()
    }

    pub unsafe fn unplug(&mut self) {
        self._plugged = false;
        //  Cancel all timers.
        if (self._has_handshake_timer) {
            self.cancel_timer(HANDSHAKE_TIMER_ID);
            self._has_handshake_timer = false;
        }

        if (self._has_ttl_timer) {
            self.cancel_timer(HEARTBEAT_TTL_TIMER_ID);
            self._has_ttl_timer = false;
        }

        if (self._has_timeout_timer) {
            self.cancel_timer(HEARTBEAT_TIMEOUT_TIMER_ID);
            self._has_timeout_timer = false;
        }

        if (self._has_heartbeat_timer) {
            self.cancel_timer(HEARTBEAT_IVL_TIMER_ID);
            self._has_heartbeat_timer = false;
        }
        //  Cancel all fd subscriptions.
        if (!self._io_error) {
            self.rm_fd(self._handle);
        }

        //  Disconnect from I/O threads poller object.
        self.io_object.unplug();

        self._session = None;
    }

    pub unsafe fn terminate(&mut self) {
        self.unplug();
    }

    pub unsafe fn in_event(&mut self) {
        self.in_event_internal()
    }

    pub unsafe fn in_event_internal(&mut self) -> bool {
        //  If still Handshaking, receive and process the greeting message.
        if ((self._handshaking)) {
            if (self.handshake()) {
                //  Handshaking was successful.
                //  Switch into the normal message flow.
                self._handshaking = false;

                if (self._mechanism.is_none() && self._has_handshake_stage) {
                    self._session.engine_ready();

                    if (self._has_handshake_timer) {
                        self.cancel_timer(HANDSHAKE_TIMER_ID);
                        self._has_handshake_timer = false;
                    }
                }
            } else {
                return false;
            }
        }


        // zmq_assert (_decoder);

        //  If there has been an I/O Error, Stop polling.
        if (self._input_stopped) {
            self.rm_fd(self._handle);
            self._io_error = true;
            return true; // TODO or return false in this case too?
        }

        //  If there's no data to process in the buffer...
        if (!self._insize) {
            //  Retrieve the buffer and read as much data as possible.
            //  Note that buffer can be arbitrarily large. However, we assume
            //  the underlying TCP layer has fixed buffer size and thus the
            //  number of bytes read will be always limited.
            let mut bufsize = 0usize;
            self._decoder.get_buffer(&mut self._inpos, &mut bufsize);

            let mut rc = self.read(self._inpos, bufsize);

            if (rc == -1) {
                if (get_errno() != EAGAIN) {
                    // Error (ConnectionError);
                    return false;
                }
                return true;
            }

            //  Adjust input size
            self._insize = (rc);
            // Adjust buffer size to received bytes
            self._decoder.resize_buffer(self._insize);
        }

        let mut rc = 0i32;
        let mut processed = 0usize;

        while (self._insize > 0) {
            rc = self._decoder.decode(self._inpos, self._insize, processed);
            // zmq_assert (processed <= _insize);
            self._inpos = self._inpos.add(processed);
            self._insize -= processed;
            if (rc == 0 || rc == -1) {
                break;
            }
            rc = (self._process_msg)(self._decoder.msg());
            if (rc == -1) {
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
            self._input_stopped = true;
            self.reset_pollin(self._handle);
        }

        self._session.flush();
        return true;
    }

    pub unsafe fn out_event(&mut self) {
        //  If write buffer is empty, try to read new data from the encoder.
        if (!self._outsize) {
            //  Even when we Stop polling as soon as there is no
            //  data to send, the poller may invoke out_event one
            //  more time due to 'speculative write' optimisation.
            if ((self._encoder.is_none())) {
                // zmq_assert (_handshaking);
                return;
            }

            self._outpos = null_mut();
            self._outsize = self._encoder.encode(&self._outpos, 0);

            while (self._outsize < (self._options.out_batch_size)) {
                if ((self._next_msg)(&self._tx_msg) == -1) {
                    //  ws_engine can cause an engine Error and delete it, so
                    //  bail out immediately to avoid use-after-free
                    if (get_errno() == ECONNRESET) {
                        return;
                    } else {
                        break;
                    }
                }
                self._encoder.load_msg(&self._tx_msg);
                let bufptr = self._outpos + self._outsize;
                let n = self._encoder.encode(&bufptr, self._options.out_batch_size - self._outsize);
                // zmq_assert (n > 0);
                if (self._outpos == null_mut()) {
                    self._outpos = bufptr;
                }
                self._outsize += n;
            }

            //  If there is no data to send, Stop polling for output.
            if (self._outsize == 0) {
                self._output_stopped = true;
                self.reset_pollout();
                return;
            }
        }

        //  If there are any data to write in write buffer, write as much as
        //  possible to the socket. Note that amount of data to write can be
        //  arbitrarily large. However, we assume that underlying TCP layer has
        //  limited transmission buffer and thus the actual number of bytes
        //  written should be reasonably modest.
        let nbytes = self.write(self._outpos, self._outsize);

        //  IO Error has occurred. We Stop waiting for output events.
        //  The engine is not Terminated until we detect input Error;
        //  this is necessary to prevent losing incoming messages.
        if (nbytes == -1) {
            self.reset_pollout();
            return;
        }

        self._outpos += nbytes;
        self._outsize -= nbytes;

        //  If we are still Handshaking and there are no data
        //  to send, Stop polling for output.
        if ((self._handshaking)) {
            if (self._outsize == 0) {
                self.reset_pollout();
            }
        }
    }

    pub unsafe fn restart_output(&mut self) {
        if self._io_error {
            return;
        }

        if self._output_stopped {
            self._output_stopped = false;
            self.set_pollout();
        }

        self.out_event();
    }

    pub unsafe fn restart_input(&mut self) -> bool {
        let rc = (self._process_msg)(self._decoder.msg());
        if (rc == -1) {
            if (get_errno() == EAGAIN) {
                self._session.flush();
            } else {
                // Error (ProtocolError);
                return false;
            }
            return true;
        }

        while (self._insize > 0) {
            let mut processed = 0usize;
            rc = self._decoder.decode(self._inpos, self._insize, processed);
            // zmq_assert (processed <= _insize);
            self._inpos = self._inpos.add(processed);
            self._insize -= processed;
            if (rc == 0 || rc == -1) {
                break;
            }
            rc = (self._process_msg)(self._decoder.msg());
            if (rc == -1) {
                break;
            }
        }

        if (rc == -1 && get_errno() == EAGAIN) {
            self._session.flush();
        } else if (self._io_error) {
            // Error (ConnectionError);
            return false;
        } else if (rc == -1) {
            // Error (ProtocolError);
            return false;
        } else {
            self._input_stopped = false;
            self.set_pollin();
            self._session.flush();

            //  Speculative read.
            if (!self.in_event_internal()) {
                return false;
            }
        }

        return true;
    }

    pub unsafe fn next_handshake_command(&mut self msg_: &mut ZmqMsg) -> i32 {
        if (self._mechanism.status() == ZmqMechanism::ready) {
            self.mechanism_ready();
            return self.pull_and_encode(msg_);
        }
        if (self._mechanism.status() == ZmqMechanism::error) {
            // errno = EPROTO;
            return -1;
        }
        let rc = self._mechanism.next_handshake_command(msg_);

        if (rc == 0) {
            msg_.set_flags(MSG_COMMAND);
        }

        return rc;
    }

    pub unsafe fn process_handshake_command(&mut self, msg_: &mut ZmqMsg) -> i32 {
        let rc = self._mechanism.process_handshake_command(msg_);
        if (rc == 0) {
            if (self._mechanism.status() == ZmqMechanism::ready) {
                self.mechanism_ready();
            } else if (self._mechanism.status() == ZmqMechanism::error) {
                // errno = EPROTO;
                return -1;
            }
            if (self._output_stopped) {
                self.restart_output();
            }
        }

        return rc;
    }

    pub unsafe fn zap_msg_available(&mut self) {
        let rc = self._mechanism.zap_msg_available();
        if (rc == -1) {
            // Error (ProtocolError);
            return;
        }
        if (self._input_stopped) {
            if (!self.restart_input()) {
                return;
            }
        }
        if (self._output_stopped) {
            self.restart_output();
        }
    }

    pub unsafe fn get_endpoint(&mut self) -> &ZmqEndpointUriPair {
        &self._endpoint_uri_pair
    }

    pub unsafe fn mechanism_ready(&mut self) {
        if (self._options.heartbeat_interval > 0 && !self._has_heartbeat_timer) {
            self.add_timer(self._options.heartbeat_interval, HEARTBEAT_IVL_TIMER_ID);
            self._has_heartbeat_timer = true;
        }

        if (self._has_handshake_stage) {
            self._session.engine_ready();
        }

        let mut flush_session = false;

        if (self._options.recv_routing_id) {
            let mut routing_id = ZmqMsg::new();
            self._mechanism.peer_routing_id(&routing_id);
            let rc = self._session.push_msg(&routing_id);
            if (rc == -1 && get_errno() == EAGAIN) {
                // If the write is failing at this stage with
                // an EAGAIN the pipe must be being shut down,
                // so we can just bail out of the routing id set.
                return;
            }
            // errno_assert (rc == 0);
            flush_session = true;
        }

        if (self._options.router_notify & ZMQ_NOTIFY_CONNECT) {
            let mut connect_notification = ZmqMsg::new();
            connect_notification.init();
            let rc = self._session.push_msg(&connect_notification);
            if (rc == -1 && get_errno() == EAGAIN) {
                // If the write is failing at this stage with
                // an EAGAIN the pipe must be being shut down,
                // so we can just bail out of the notification.
                return;
            }
            // errno_assert (rc == 0);
            flush_session = true;
        }

        if (flush_session) {
            self._session.flush();
        }

        self._next_msg = &ZmqStreamEngineBase::pull_and_encode;
        self._process_msg = &ZmqStreamEngineBase::write_credential;

        //  Compile metadata.
        let mut properties = properties_t::default();
        self.init_properties(properties);

        //  Add ZAP properties.
        let mut zap_properties = self._mechanism.get_zap_properties();
        properties.insert(zap_properties.begin(), zap_properties.end());

        //  Add ZMTP properties.
        let mut zmtp_properties = self._mechanism.get_zmtp_properties();
        properties.insert(zmtp_properties.begin(), zmtp_properties.end());

        // zmq_assert (_metadata == NULL);
        if (!properties.empty()) {
            self._metadata = ZmqMetadata::new(properties);
            // alloc_assert (_metadata);
        }

        if (self._has_handshake_timer) {
            self.cancel_timer(HANDSHAKE_TIMER_ID);
            self._has_handshake_timer = false;
        }

        self._socket.event_handshake_succeeded(self._endpoint_uri_pair, 0);
    }

    pub unsafe fn write_credential(&mut self, msg_: &mut ZmqMsg) -> i32 {
        let mut credential = self._mechanism.get_user_id();
        if (credential.size() > 0) {
            let mut msg = ZmqMsg::default();
            let rc = msg.init_size(credential.size());
            // zmq_assert (rc == 0);
            libc::memcpy(msg.data_mut(), credential.data(), credential.size());
            msg.set_flags(ZmqMsg::credential);
            rc = self._session.push_msg(&msg);
            if (rc == -1) {
                rc = msg.close();
                // errno_assert (rc == 0);
                return -1;
            }
        }
        self._process_msg = &ZmqStreamEngineBase::decode_and_push;
        return self.decode_and_push(msg_);
    }

    pub unsafe fn pull_and_decode(&mut self, msg_: &ZmqMsg) -> i32 {
        if (self._session.pull_msg(msg_) == -1) {
            return -1;
        }
        if (self._mechanism.encode(msg_) == -1) {
            return -1;
        }
        return 0;
    }

    pub unsafe fn decode_and_push(&mut self, msg_: &mut ZmqMsg) -> i32 {
        if (self._mechanism.decode(msg_) == -1) {
            return -1;
        }

        if (self._has_timeout_timer) {
            self._has_timeout_timer = false;
            self.cancel_timer(HEARTBEAT_TIMEOUT_TIMER_ID);
        }

        if (self._has_ttl_timer) {
            self._has_ttl_timer = false;
            self.cancel_timer(HEARTBEAT_TTL_TIMER_ID);
        }

        if msg_.flag_set(MSG_COMMAND) {
            self.process_command_message(msg_);
        }

        if (self._metadata) {
            msg_.set_metadata(self._metadata);
        }
        if (self._session.push_msg(msg_) == -1) {
            if (get_errno() == EAGAIN) {
                self._process_msg = &ZmqStreamEngineBase::push_one_then_decode_and_push;
            }
            return -1;
        }
        return 0;
    }

    pub unsafe fn push_one_then_decode_and_push(&mut self, msg_: &mut ZmqMsg) -> i32 {
        if (self._session.push_msg(msg_) == -1) {
            return -1;
        }
        self._process_msg = &ZmqStreamEngineBase::decode_and_push;
        return self.decode_and_push(msg_);
    }

    pub unsafe fn pull_msg_from_session(&mut self, msg_: &ZmqMsg) -> i32 {
        self._session.pull_msg(msg_)
    }

    pub unsafe fn push_msg_to_session(&mut self, msg_: &ZmqMsg) -> i32 {
        self._session.push_msg(msg_)
    }

    pub unsafe fn error(&mut self, reason_: ErrorReason) {
        if ((self._options.router_notify & ZMQ_NOTIFY_DISCONNECT) && !self._handshaking) {
            // For router sockets with disconnect notification, rollback
            // any incomplete message in the pipe, and push the disconnect
            // notification message.
            self._session.rollback();

            let mut disconnect_notification = ZmqMsg::new();
            disconnect_notification.init();
            self._session.push_msg(&disconnect_notification);
        }

        // protocol errors have been signaled already at the point where they occurred
        if (reason_ != ProtocolError && (self._mechanism.is_none() || self._mechanism.status() == ZmqMechanism::handshaking)) {
            let mut err = get_errno;
            self._socket.event_handshake_failed_no_detail(self._endpoint_uri_pair, err);
            // special case: connecting to non-ZMTP process which immediately drops connection,
            // or which never responds with greeting, should be treated as a protocol Error
            // (i.e. Stop reconnect)
            if (((reason_ == ConnectionError) || (reason_ == TimeoutError)) && (self._options.reconnect_stop & ZMQ_RECONNECT_STOP_HANDSHAKE_FAILED)) {
                reason_ = ProtocolError;
            }
        }

        self._socket.event_disconnected(&self._endpoint_uri_pair, self._s);
        self._session.flush();
        self._session.engine_error(
            !self._handshaking && (self._mechanism.is_none() || self._mechanism.status() != ZmqMechanism::handshaking),
            reason_);
        self.unplug();
        // delete this;
    }

    pub unsafe fn set_handshake_timer(&mut self) {
        if (self._options.handshake_ivl > 0) {
            self.add_timer(self._options.handshake_ivl, HANDSHAKE_TIMER_ID);
            self._has_handshake_timer = true;
        }
    }

    pub unsafe fn init_properties(&mut self, properties_: &mut properties_t) -> bool {
        if (self._peer_address.empty()) {
            return false;
        }
        properties_.insert(
            ZMQ_MSG_PROPERTY_PEER_ADDRESS, self._peer_address);

        //  Private property to support deprecated SRCFD
        // TODO
        // std::ostringstream stream;
        // stream << static_cast<int> (_s);
        // std::string fd_string = stream.str ();
        // properties_.insert (std::string ("__fd"),
        //                                        ZMQ_MOVE (fd_string));
        return true;
    }

    pub unsafe fn timer_event(&mut self, id_: i32) {
        if (id_ == HANDSHAKE_TIMER_ID) {
            self._has_handshake_timer = false;
            //  handshake timer expired before handshake completed, so engine fail
            // Error (TimeoutError);
        } else if (id_ == HEARTBEAT_IVL_TIMER_ID) {
            self._next_msg = &ZmqStreamEngineBase::produce_ping_message;
            self.out_event();
            self.add_timer(self._options.heartbeat_interval, HEARTBEAT_IVL_TIMER_ID);
        } else if (id_ == HEARTBEAT_TTL_TIMER_ID) {
            self._has_ttl_timer = false;
            // Error (TimeoutError);
        } else if (id_ == HEARTBEAT_TIMEOUT_TIMER_ID) {
            self._has_timeout_timer = false;
            // Error (TimeoutError);
        } else {}
        // There are no other valid timer ids!
        // assert (false);
    }
    
    pub unsafe fn read(&mut self, data_: &mut [u8], size_: usize) -> i32 {
        let rc = zmq::tcp_read(self._s, data_, size_);
        if (rc == 0) {
            // connection closed by peer
            // errno = EPIPE;
            return -1;
        }
    
        return rc;
    }
    
    pub unsafe fn write(&mut self, data_: &mut [u8] size_: usize) -> i32 {
        zmq::tcp_write(self._s, data_, size_)
    }
}
