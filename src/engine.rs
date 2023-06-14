use std::collections::HashMap;
use std::net::SocketAddr;
use std::os::raw::c_void;
use std::ptr::null_mut;
use anyhow::bail;
use libc::{c_int, EAGAIN, EPROTO, read, write};
use windows::Win32::Networking::WinSock::PF_UNIX;
use crate::address::ZmqAddress;
use crate::context::ZmqContext;
use crate::defines::{handshake_timer_id, heartbeat_ivl_timer_id, heartbeat_timeout_timer_id, heartbeat_ttl_timer_id, NormInstanceHandle, NormObjectHandle, NormSessionHandle, ZMQ_MSG_PROPERTY_PEER_ADDRESS, ZMQ_NOTIFY_CONNECT, ZMQ_NOTIFY_DISCONNECT, ZmqHandle};
use crate::encoder::EncoderBase;
use crate::endpoint::EndpointUriPair;
use crate::fd::ZmqFileDesc;
use crate::io_object::ZmqIoObject;
use crate::ip::get_peer_ip_address;
use crate::mechanism::ZmqMechanism;
use crate::mechanism::ZmqMechanismStatus::ready;
use crate::message::{ZMQ_MSG_COMMAND, ZMQ_MSG_CREDENTIAL, ZmqMessage};
use crate::metadata::ZmqMetadata;
use crate::norm_stream_state::NormRxStreamState;
use crate::session_base::ZmqSessionBase;
use crate::socket::ZmqSocket;

use crate::tcp::{tcp_read, tcp_write};
use crate::thread_context::ZmqThreadContext;
use crate::transport::ZmqTransport;
use crate::udp::{udp_get_endpoint, udp_in_event, udp_init, udp_restart_input, udp_restart_output, udp_terminate};
use crate::utils::copy_bytes;
use crate::v2_decoder::ZmqV2Decoder;

#[derive(Default,Debug,Clone)]
pub struct ZmqEngine<'a> {
    pub plugged: bool,
    pub session: Option<&'a mut ZmqSessionBase>,
    pub io_object: ZmqIoObject,
    pub handle: ZmqHandle,
    pub fd: ZmqFileDesc,
    pub context: &'a mut ZmqContext<'a>,
    pub address: ZmqAddress,
    pub out_address: ZmqAddress,
    pub recv_enabled: bool,
    pub send_enabled: bool,
    pub inpos: &'a mut[u8],
    pub insize: usize,
    pub outpos: &'a mut[u8],
    pub outsize: usize,
    pub encoder: EncoderBase, // zmq_encoder -- Option<ZmqV2Encoder> for norm engine
    pub mechanism: Option<ZmqMechanism>,
    pub next_msg: fn(msg: &mut ZmqMessage) -> i32,
    pub process_msg: fn(msg: &mut ZmqMessage) -> i32,
    pub metadata: Option<ZmqMetadata>,
    pub input_stopped: bool,
    pub output_stopped: bool,
    pub endpoint_uri_pair: EndpointUriPair,
    pub has_handshake_timer: bool,
    pub has_ttl_timer: bool,
    pub has_timeout_timer: bool,
    pub has_heartbeat_timer: bool,
    pub peer_address: String,
    pub handshaking: bool,
    pub tx_msg: ZmqMessage,
    pub io_error: bool,
    pub socket: &'a mut ZmqSocket<'a>,
    pub has_handshake_stage: bool,
    pub empty_endpoint: EndpointUriPair,
    pub norm_instance: NormInstanceHandle,
    pub norm_descriptor_handle: ZmqHandle,
    pub norm_session: NormSessionHandle,
    pub is_sender: bool,
    pub is_receiver: bool,
    pub norm_tx_stream: NormObjectHandle,
    pub tx_first_msg: bool,
    pub tx_more_bit: bool,
    pub output_ready: bool,
    pub norm_tx_ready: bool,
    pub tx_buffer: Vec<u8>,
    pub tx_len: usize,
    pub input_ready: bool,
    pub rx_pending_list: Vec<NormRxStreamState<'a>>,
    pub rx_ready_list: Vec<NormRxStreamState<'a>>,
    pub msg_ready_list: Vec<NormRxStreamState<'a>>,
    pub wrapper_read_fd: ZmqFileDesc,
    pub wrapper_thread_id: u32,
    pub raw_address: Option<SocketAddr>,
    pub out_buffer: Vec<u8>,
}

impl <'a> ZmqEngine <'a> {

    pub fn in_event_internal(&mut self) -> bool {
        // zmq_assert (!_io_error);

        //  If still handshaking, receive and process the greeting message.
        if (self.handshaking) {
            if (self.handshake()) {
                //  Handshaking was successful.
                //  Switch into the normal message flow.
                self.handshaking = false;

                if (self.mechanism.is_none() && self.has_handshake_stage) {
                    self.session.engine_ready();

                    if (self.has_handshake_timer) {
                        self.io._object.cancel_timer(self.handshake_timer_id);
                        self.has_handshake_timer = false;
                    }
                }
            } else {
                return false;
            }
        }

        // zmq_assert (_decoder);

        //  If there has been an I/O error, Stop polling.
        if (self.input_stopped) {
            self.io_object.rm_fd(self.handle);
            self.io_error = true;
            return true; // TODO or return false in this case too?
        }

        //  If there's no data to process in the buffer...
        unsafe {
            if (!self.insize) {
                //  Retrieve the buffer and read as much data as possible.
                //  Note that buffer can be arbitrarily large. However, we assume
                //  the underlying TCP layer has fixed buffer size and thus the
                //  number of bytes read will be always limited.
                let mut bufsize = 0;
                self.decoder.get_buffer(&self.inpos, &bufsize);

                // TODO
                let rc: i32 = read(self.fd as c_int, self.inpos as *mut c_void, bufsize);
                if (rc == -1) {
                    // TODO
                    // if (errno != EAGAIN) {
                    //     // error (connection_error);
                    //     return false;
                    // }
                    return true;
                }

                //  Adjust input size
                self.insize = rc as usize; //static_cast<size_t> (rc);
                // Adjust buffer size to received bytes
                self.decoder.resize_buffer(self.insize);
            }
        }

        let mut rc = 0;
        let mut processed = 0;

        while (self.insize > 0) {
            rc = self.decoder.decode(self.inpos, self.insize, processed);
            // zmq_assert (processed <= _insize);
            self.inpos += processed;
            self.insize -= processed;
            if (rc == 0 || rc == -1) {
                break;
            }
            rc = self.process_msg(self.decoder.msg());
            if (rc == -1) {
                break;
            }
        }

        //  Tear down the connection if we have failed to decode input data
        //  or the session has rejected the message.
        if (rc == -1) {
            // TODO
            // if (errno != EAGAIN) {
            //     // error (protocol_error);
            //     return false;
            // }
            self.input_stopped = true;
            self.io_thread.reset_pollin(self.handle);
        }

        self.session.flush();
        return true;
    }

    //  Unplug the engine from the session.
    // void unplug ();
    pub fn unplug(&mut self) {
        // zmq_assert (_plugged);
        self.plugged = false;

        //  Cancel all timers.
        if (self.has_handshake_timer) {
            self.io_object.cancel_timer(self.handshake_timer_id);
            self.has_handshake_timer = false;
        }

        if (self.has_ttl_timer) {
            self.io_object.cancel_timer(self.heartbeat_ttl_timer_id);
            self.has_ttl_timer = false;
        }

        if (self.has_timeout_timer) {
            self.io_object.cancel_timer(self.heartbeat_timeout_timer_id);
            self.has_timeout_timer = false;
        }

        if (self.has_heartbeat_timer) {
            self.io_object.cancel_timer(self.heartbeat_ivl_timer_id);
            self.has_heartbeat_timer = false;
        }
        //  Cancel all fd subscriptions.
        if (!self.io_error) {
            self.io_object.rm_fd(self.handle);
        }

        //  Disconnect from I/O threads poller object.
        self.io_object.unplug();

        self.session = None;
    }

    pub fn mechanism_ready(&mut self) {
        if (self.options.heartbeat_interval > 0 && !self.has_heartbeat_timer) {
            self.io_object.add_timer(self.options.heartbeat_interval, heartbeat_ivl_timer_id as i32);
            self.has_heartbeat_timer = true;
        }

        if (self.has_handshake_stage) {
            self.session.engine_ready();
        }

        let mut flush_session = false;

        if (self.options.recv_routing_id) {
            let mut routing_id: ZmqMessage = ZmqMessage::default();
            self.mechanism.peer_routing_id(&routing_id);
            let rc: i32 = self.session.push_msg(&routing_id);
            //  && errno == EAGAIN
            if (rc == -1) {
                // If the write is failing at this stage with
                // an EAGAIN the pipe must be being shut down,
                // so we can just bail out of the routing id set.
                return;
            }
            // errno_assert (rc == 0);
            flush_session = true;
        }

        if (self.options.router_notify & ZMQ_NOTIFY_CONNECT) {
            let mut connect_notification = ZmqMessage::default();
            connect_notification.init2();
            let rc: i32 = self.session.push_msg(&connect_notification);
            // && errno == EAGAIN
            if (rc == -1 ) {
                // If the write is failing at this stage with
                // an EAGAIN the pipe must be being shut down,
                // so we can just bail out of the notification.
                return;
            }
            // errno_assert (rc == 0);
            flush_session = true;
        }

        if (flush_session) {
            self.session.flush();
        }

        self.next_msg = self.pull_and_encode;
        self.process_msg = self.write_credential;

        //  Compile metadata.
        // TODO properties_t is actually a t ypedef for metadata_t::dict_t -> hashmap<string,string>
        // let mut properties: HashMap<> = properties_t::default();
        // init_properties(properties);
        //
        // //  Add ZAP properties.
        // let zap_properties = self.mechanism.get_zap_properties();
        // properties.insert(zap_properties.begin(), zap_properties.end());
        //
        // //  Add ZMTP properties.
        // let zmtp_properties = self.mechanism.get_zmtp_properties();
        // properties.insert(zmtp_properties.begin(), zmtp_properties.end());
        //
        // // zmq_assert (_metadata == null_mut());
        // if (!properties.empty()) {
        //     self.metadata = ZmqMetadata::with_properties(properties);
        //     // alloc_assert (_metadata);
        // }

        if (self.has_handshake_timer) {
            self.io_thread.cancel_timer(self.handshake_timer_id);
            self.has_handshake_timer = false;
        }

        self.socket
            .event_handshake_succeeded(&mut self.options, &self.endpoint_uri_pair, 0);
    }

    pub fn pull_and_encode(&mut self, msg: &mut ZmqMessage) -> i32 {
        // zmq_assert (_mechanism != null_mut());

        if (self.session.pull_msg(msg) == -1) {
            return -1;
        }
        if (self.mechanism.unwrap().encode(msg) == -1) {
            return -1;
        }
        return 0;
    }

    //  Indicate if the engine has an handshake stage.
    //  If engine has handshake stage, engine must call session.engine_ready when the handshake is complete.
    // virtual bool has_handshake_stage () = 0;
    pub fn has_handshake_stage(&self) -> bool {
        self._has_handshake_stage
    }

    //  Plug the engine to the session.
    // virtual void Plug (ZmqIoThread *io_thread_, pub struct ZmqSessionBase *session_) = 0;
    pub fn plug(&mut self, io_thread_: &mut ZmqThreadContext, session: &mut ZmqSessionBase) {
        // zmq_assert (!_plugged);
        self.plugged = true;

        //  Connect to session object.
        // zmq_assert (!_session);
        // zmq_assert (session_);
        self.session = Some(*session.clone());
        self.socket = self._session.get_socket();

        //  Connect to I/O threads poller object.
        self.io_object.plug(io_thread_);
        self.handle = self.io_thread.add_fd(self.fd);
        self.io_error = false;

        self.plug_internal();
    }

    //  Terminate and deallocate the engine. Note that 'detached'
    //  events are not fired on termination.
    // virtual void terminate () = 0;
    pub fn terminate(&mut self) {

        match self.address.protocol {
            ZmqTransport::ZmqUdp => udp_terminate(self),
            _ => {
                self.unplug()
            }
        }
    }

    //  This method is called by the session to signalise that more
    //  messages can be written to the pipe.
    //  Returns false if the engine was deleted due to an error.
    //  TODO it is probably better to change the design such that the engine
    //  does not delete itself
    // virtual bool restart_input () = 0;
    pub fn restart_input(&mut self) -> bool {

        match self.address.protocol {
            ZmqTransport::ZmqUdp => udp_restart_input(self),
            _ => {}
        }

        let mut rc = self.process_msg(self.decoder.msg());
        if (rc == -1) {
            // TODO
            // if (errno == EAGAIN) {
            //     self._session.flush();
            // } else {
            //     // error (protocol_error);
            //     return false;
            // }
            // return true;
        }

        while (self.insize > 0) {
            let mut processed = 0;
            rc = self.decoder.decode(self.inpos, self.insize, processed);
            // zmq_assert (processed <= _insize);
            self.inpos += processed;
            self.insize -= processed;
            if (rc == 0 || rc == -1) {
                break;
            }
            rc = (self.process_msg)(self.decoder.msg());
            if (rc == -1) {
                break;
            }
        }

        // if (rc == -1 && errno == EAGAIN)
        if (rc == -1)
        {
            self.session.flush();
        } else if (self.io_error) {
            // error (connection_error);
            return false;
        } else if (rc == -1) {
            // error (protocol_error);
            return false;
        } else {
            self.input_stopped = false;
            self.io_object.set_pollin(self.handle);
            self.session.flush();

            //  Speculative read.
            if (!self.in_event_internal()) {
                return false;
            }
        }

        return true;

    }

    // pub fn get_endpoint(&self) -> EndpointUriPair {
    //
    // }

    //  This method is called by the session to signalise that there
    //  are messages to send available.
    // virtual void restart_output () = 0;
    pub fn restart_output(&mut self) {

        match self.address.protocol {
            ZmqTransport::ZmqUdp => udp_restart_output(self),
            _ => {}
        }

        if (self.io_error) {
            return;
        }

        if (self.output_stopped) {
            self.set_pollout();
            self.output_stopped = false;
        }

        //  Speculative write: The assumption is that at the moment new message
        //  was sent by the user the socket is probably available for writing.
        //  Thus we try to write the data to socket avoiding polling for POLLOUT.
        //  Consequently, the latency should be better in request/reply scenarios.
        self.out_event();
    }

    // virtual void zap_msg_available () = 0;
    pub fn zap_msg_available(&mut self) {
        // zmq_assert (_mechanism != null_mut());

        let rc: i32 = self.mechanism.zap_msg_available();
        if (rc == -1) {
            // error (protocol_error);
            return;
        }
        if (self.input_stopped) {
            if (!self.restart_input()) {
                return;
            }
        }
        if (self.output_stopped) {
            self.restart_output();
        }
    }

    pub fn in_event(&mut self) {

        match self.address.protocol {
            ZmqTransport::ZmqUdp => udp_in_event(self),
            _ => {}
        }

        let res = self.in_event_internal();
    }

    pub fn out_event(&mut self) {
        // zmq_assert (!_io_error);

        //  If write buffer is empty, try to read new data from the encoder.
        if (!self.outsize) {
            //  Even when we Stop polling as soon as there is no
            //  data to send, the poller may invoke out_event one
            //  more time due to 'speculative write' optimisation.
            if (self.encoder == null_mut()) {
                // zmq_assert (_handshaking);
                return;
            }

            // self.outpos = 0;
            // TODO
            // self._outsize = self._encoder.encode (&self._outpos, 0);

            while (self.outsize < (self.options.out_batch_size) as usize) {
                if ((self.next_msg)(&mut self.tx_msg) == -1) {
                    //  ws_engine can cause an engine error and delete it, so
                    //  bail out immediately to avoid use-after-free
                    // TODO
                    // if (errno == ECONNRESET) {
                    //     return;
                    // } else {
                    //     break;
                    // }
                }
                self._encoder.load_msg(&mut self.tx_msg);
                // TODO
                // unsigned char *bufptr = _outpos + _outsize;
                let bufptr = self.outpos + self.outsize;
                let n = self.encoder.encode(
                    &mut Some(bufptr),
                    (self.options.out_batch_size - self.outsize) as usize,
                );
                // zmq_assert (n > 0);
                if (self.outpos == 0) {
                    self.outpos = bufptr;
                }
                self.outsize += n;
            }

            //  If there is no data to send, Stop polling for output.
            if (self.outsize == 0) {
                self.output_stopped = true;
                self.io_thread.reset_pollout();
                return;
            }
        }

        //  If there are any data to write in write buffer, write as much as
        //  possible to the socket. Note that amount of data to write can be
        //  arbitrarily large. However, we assume that underlying TCP layer has
        //  limited transmission buffer and thus the actual number of bytes
        //  written should be reasonably modest.
        // TODO
        let nbytes: i32 = unsafe { write(self.fd as c_int, self._outpos, self._outsize) };

        //  IO error has occurred. We Stop waiting for output events.
        //  The engine is not terminated until we detect input error;
        //  this is necessary to prevent losing incoming messages.
        if (nbytes == -1) {
            self.io_thread.reset_pollout();
            return;
        }

        self.outpos += nbytes;
        self.outsize -= nbytes;

        //  If we are still handshaking and there are no data
        //  to send, Stop polling for output.
        if (self.handshaking) {
            if (self.outsize == 0) {
                self.io_thread.reset_pollout();
            }
        }
    }

    pub fn next_handshake_command(&mut self, msg: &mut ZmqMessage) -> i32 {
        // zmq_assert (_mechanism != null_mut());

        if (self.mechanism.status() == ready) {
            self.mechanism_ready();
            return self.pull_and_encode(msg);
        }
        if (self.mechanism.status() == ZmqMechanism::error) {
            // errno = EPROTO;
            return -1;
        }
        let rc: i32 = self.mechanism.next_handshake_command(msg);

        if (rc == 0) {
            msg.set_flags(ZMQ_MSG_COMMAND);
        }

        return rc;
    }

    pub fn process_handshake_command(&mut self, msg: &mut ZmqMessage) -> i32 {
        // zmq_assert (_mechanism != null_mut());
        let rc: i32 = self.mechanism.process_handshake_command(msg);
        if (rc == 0) {
            if (self.mechanism.status() == ready) {
                self.mechanism_ready();
            }
            // TODO
            // else if (self.mechanism.status() == error) {
            //     // errno = EPROTO;
            //     return -1;
            // }
            if (self.output_stopped) {
                self.restart_output();
            }
        }

        return rc;
    }

    pub fn reset_pollout(&mut self) {
        self.io_object.reset_pollout(self._handle)
    }

    pub fn set_pollout(&mut self) {
        self.io_object.set_pollout(self._handle)
    }

    pub fn set_pollin(&mut self) {
        self.io_object.set_pollin(self._handle)
    }

    pub fn session(&mut self) -> &mut Option<ZmqSessionBase> {
        &mut self._session
    }

    pub fn socket(&mut self) -> &mut ZmqSocket {
        &mut self._socket
    }

    // virtual const EndpointUriPair &get_endpoint () const = 0;
    pub fn get_endpoint(&self) -> &EndpointUriPair {
        match self.address.protocol {
            ZmqTransport::ZmqUdp => udp_get_endpoint(self),
            _ => self.empty_endpoint.clone()
        }
    }

    pub fn get_buffer(&mut self, data: &mut [u8], size: &mut usize) {
        todo!()
    }

    pub fn resize_buffer(&mut self, size: usize) {
        todo!()
    }

    pub fn decode(&mut self, data: &mut[u8], size: usize, processed: &mut usize) {
        todo!()
    }

    pub fn msg(&mut self) -> &mut ZmqMessage {
        todo!()
    }

    pub fn write_credential(&mut self, msg: &mut ZmqMessage) -> anyhow::Result<()> {
        // zmq_assert (_mechanism != null_mut());
        // zmq_assert (_session != null_mut());

        let credential = self._mechanism.get_user_id();
        if (credential.size() > 0) {
            let mut msg = ZmqMessage::default();
            msg.init_size(credential.size())?;
            // zmq_assert (rc == 0);
            copy_bytes(msg.data_mut(), 0, credential.data(), 0, credential.size());
            msg.set_flags(ZMQ_MSG_CREDENTIAL);
            let rc = self._session.push_msg(&msg);
            if (rc == -1) {
                rc = msg.close();
                // errno_assert (rc == 0);
                // return -1;
                bail!("push msg error");
            }
        }
        self._process_msg = self.decode_and_push;
        return self.decode_and_push(msg);
    }

    pub fn process_command_msg(&mut self, msg: &mut ZmqMessage) -> anyhow::Result<()> {
        unimplemented!()
    }

    pub fn decode_and_push(&mut self, msg: &mut ZmqMessage) -> anyhow::Result<()> {
        // zmq_assert (_mechanism != null_mut());

        if (self._mechanism.decode(msg) == -1) {
            bail!("decode error");
        }

        if (self._has_timeout_timer) {
            self._has_timeout_timer = false;
            self.io_thread.cancel_timer(heartbeat_timeout_timer_id);
        }

        if (self._has_ttl_timer) {
            self._has_ttl_timer = false;
            self.io_thread.cancel_timer(heartbeat_ttl_timer_id);
        }

        if msg.flags() & ZMQ_MSG_COMMAND {
            self.process_command_message(msg);
        }

        if (self._metadata) {
            msg.set_metadata(&mut self._metadata.unwrap());
        }
        if (self.session.push_msg(msg) == -1) {
            // TODO
            // if (errno == EAGAIN) {
            //     self.process_msg = self.push_one_then_decode_and_push;
            // }
            // return -1;
            bail!("failed to push msg")
        }
        // return 0;
        Ok(())
    }

    pub fn push_one_then_decode_and_push(&mut self, msg: &mut ZmqMessage) -> i32 {
        let rc: i32 = self._session.push_msg(msg);
        if (rc == 0) {
            self._process_msg = self.decode_and_push;
        }
        return rc;
    }

    pub fn pull_msg_from_session(&mut self, msg: &mut ZmqMessage) -> i32 {
        return self._session.pull_msg(msg);
    }

    pub fn push_msg_to_session(&mut self, msg: &mut ZmqMessage) -> i32 {
        return self._session.push_msg(msg);
    }

    // pub fn error(&mut self, reason_: ZmqErrorReason) {
    //     // zmq_assert (_session);
    //
    //     if (self._options.router_notify & ZMQ_NOTIFY_DISCONNECT) != 0 && !self._handshaking {
    //         // For router sockets with disconnect notification, rollback
    //         // any incomplete message in the pipe, and push the disconnect
    //         // notification message.
    //         self._session.rollback();
    //
    //         let mut disconnect_notification: ZmqMessage = ZmqMessage::default();
    //         disconnect_notification.init2();
    //         self._session.push_msg(&disconnect_notification);
    //     }
    //
    //     // protocol errors have been signaled already at the point where they occurred
    //     if (reason_ != protocol_error
    //         && (self._mechanism == null_mut()
    //         || self._mechanism.status() == ZmqMechanism::handshaking))
    //     {
    //         let err: i32 = errno;
    //         self._socket.event_handshake_failed_no_detail(
    //             &mut self._options,
    //             &self._endpoint_uri_pair,
    //             err,
    //         );
    //         // special case: connecting to non-ZMTP process which immediately drops connection,
    //         // or which never responds with greeting, should be treated as a protocol error
    //         // (i.e. Stop reconnect)
    //         if (((reason_ == connection_error) || (reason_ == timeout_error))
    //             && (self._options.reconnect_stop & ZMQ_RECONNECT_STOP_HANDSHAKE_FAILED) != 0)
    //         {
    //             reason_ = protocol_error;
    //         }
    //     }
    //
    //     self._socket
    //         .event_disconnected(&mut self._options, &self._endpoint_uri_pair, self._s);
    //     self._session.flush();
    //     self._session.engine_error(
    //         !self._handshaking
    //             && (self._mechanism == null_mut()
    //             || self._mechanism.status() != ZmqMechanism::handshaking),
    //         reason_,
    //     );
    //     unplug();
    //     // delete this;
    // }

    pub fn set_handshake_timer(&mut self) {
        // zmq_assert (!_has_handshake_timer);

        if (self._options.handshake_ivl > 0) {
            self.io_thread.add_timer(self._options.handshake_ivl, handshake_timer_id);
            self._has_handshake_timer = true;
        }
    }

    pub fn init_properties(&mut self, properties_: &mut HashMap<String,String>) -> bool {
        if (self._peer_address.empty()) {
            return false;
        }
        properties_.ZMQ_MAP_INSERT_OR_EMPLACE((ZMQ_MSG_PROPERTY_PEER_ADDRESS), self.peer_address.clone());

        //  Private property to support deprecated SRCFD
        // std::ostringstream stream;
        // stream <<  (_s);
        // std::string fd_string = stream.str ();
        let fd_string = format!("{}", self._s);
        properties_.ZMQ_MAP_INSERT_OR_EMPLACE("__fd".to_string(), fd_string);
        return true;
    }

    pub fn timer_event(&mut self, id_: i32) {
        if (id_ == handshake_timer_id) {
            self.has_handshake_timer = false;
            //  handshake timer expired before handshake completed, so engine fail
            // error (timeout_error);
        } else if (id_ == heartbeat_ivl_timer_id) {
            self.next_msg = self.produce_ping_message;
            self.out_event();
            self.io_thread.add_timer(self._options.heartbeat_interval, heartbeat_ivl_timer_id);
        } else if (id_ == heartbeat_ttl_timer_id) {
            self.has_ttl_timer = false;
            // error (timeout_error);
        } else if (id_ == heartbeat_timeout_timer_id) {
            self.has_timeout_timer = false;
            // error (timeout_error);
        } else {
            // There are no other valid timer ids!
            // assert(false);
        }
    }

    pub fn read(&mut self, data: &mut [u8], size: usize) -> anyhow::Result<()> {
        let rc: i32 = tcp_read(self._s, data, size);

        if (rc == 0) {
            // connection closed by peer
            // errno = EPIPE;
            bail!("tcp_read failed")
        }

        Ok(())
    }

    pub fn write(&mut self, data: &mut [u8], size: usize) -> i32 {
        return tcp_write(self._s, data, size);
    }

    pub fn produce_ping_msg(&mut self, msg: &mut ZmqMessage) -> anyhow::Result<()>
    {
        unimplemented!("TODO")
    }

    pub fn init(&mut self, address: &mut ZmqAddress, send: bool, recv: bool) -> anyhow::Result<()>
    {
        match self.address.protocol {
            ZmqTransport::ZmqUdp=> {
                udp_init(self, address, send, recv)
            }
            _ => bail!("unsupported protocol")
        }
    }
}

pub fn get_peer_address(s_: ZmqFileDesc) -> String {
    let mut peer_address: String = String::new();

    let family: i32 = get_peer_ip_address(s_, &peer_address);
    if (family == 0) {
        peer_address.clear();
    }
    // #if defined ZMQ_HAVE_SO_PEERCRED
    else if (cfg!(so_peer_cred) && family == PF_UNIX) {
        // struct ucred cred;
        #[cfg(feature = "so_peer_cred")]
        {
            // let cred: ucred = ucred::new();
            // socklen_t
            // size = mem::size_of::<cred>();
            // if (!getsockopt(s_, SOL_SOCKET, SO_PEERCRED, &cred, &size)) {
            //     std::ostringstream
            //     buf;
            //     buf << ":" << cred.uid << ":" << cred.gid << ":" << cred.pid;
            //     peer_address += buf.str();
            // }
        }
    }
    // #elif defined ZMQ_HAVE_LOCAL_PEERCRED
    else if (cfg!(local_peer_cred) && family == PF_UNIX) {
        #[cfg(feature = "local_peer_cred")]
        {
            // struct xucred
            // cred;
            // socklen_t
            // size = mem::size_of::<cred>();
            // if (!getsockopt(s_, 0, LOCAL_PEERCRED, &cred, &size)
            //     && cred.cr_version == XUCRED_VERSION) {
            //     std::ostringstream
            //     buf;
            //     buf << ":" << cred.cr_uid << ":";
            //     if (cred.cr_ngroups > 0)
            //     buf << cred.cr_groups[0];
            //     buf << ":";
            //     peer_address += buf.str();
            // }
        }
    }
    // #endif
    return peer_address;
}
