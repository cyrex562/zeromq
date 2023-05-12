/*
    Copyright (c) 2007-2016 Contributors as noted in the AUTHORS file

    This file is part of libzmq, the ZeroMQ core engine in C+= 1.

    libzmq is free software; you can redistribute it and/or modify it under
    the terms of the GNU Lesser General Public License (LGPL) as published
    by the Free Software Foundation; either version 3 of the License, or
    (at your option) any later version.

    As a special exception, the Contributors give you permission to link
    this library with independent modules to produce an executable,
    regardless of the license terms of these independent modules, and to
    copy and distribute the resulting executable under terms of your choice,
    provided that you also meet, for each linked independent module, the
    terms and conditions of the license of that module. An independent
    module is a module which is not derived from or based on this library.
    If you modify this library, you must extend this exception to your
    version of the library.

    libzmq is distributed in the hope that it will be useful, but WITHOUT
    ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
    FITNESS FOR A PARTICULAR PURPOSE. See the GNU Lesser General Public
    License for more details.

    You should have received a copy of the GNU Lesser General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

// #include "precompiled.hpp"
// #include "macros.hpp"

// #include <limits.h>
// #include <string.h>

// #ifndef ZMQ_HAVE_WINDOWS
// #include <unistd.h>
// #endif

// #include <new>
// #include <sstream>

// #include "stream_engine_base.hpp"
// #include "io_thread.hpp"
// #include "session_base.hpp"
// #include "v1_encoder.hpp"
// #include "v1_decoder.hpp"
// #include "v2_encoder.hpp"
// #include "v2_decoder.hpp"
// #include "null_mechanism.hpp"
// #include "plain_client.hpp"
// #include "plain_server.hpp"
// #include "gssapi_client.hpp"
// #include "gssapi_server.hpp"
// #include "curve_client.hpp"
// #include "curve_server.hpp"
// #include "raw_decoder.hpp"
// #include "raw_encoder.hpp"
// #include "config.hpp"
// #include "err.hpp"
// #include "ip.hpp"
// #include "tcp.hpp"
// #include "likely.hpp"
// #include "wire.hpp"

use std::mem;
use std::ptr::null_mut;

use bincode::options;
use libc::{size_t, EAGAIN, ECONNRESET, EPIPE, EPROTO};
use windows::Win32::Networking::WinSock::{socklen_t, PF_UNIX, SOL_SOCKET};

use crate::decoder::ZmqDecoderInterface;
use crate::defines::{
    ZmqHandle, ZMQ_MSG_PROPERTY_PEER_ADDRESS, ZMQ_NOTIFY_CONNECT, ZMQ_NOTIFY_DISCONNECT,
    ZMQ_RECONNECT_STOP_HANDSHAKE_FAILED,
};
use crate::encoder::ZmqBaseEncoder;
use crate::endpoint::EndpointUriPair;
use crate::engine_interface::ZmqEngineInterface;
use crate::fd::ZmqFileDesc;
use crate::io_object::ZmqIoObject;
use crate::io_thread::ZmqIoThread;
use crate::mechanism::ZmqMechanismStatus::{error, ready};
use crate::message::{ZmqMessage, ZMQ_MSG_COMMAND, ZMQ_MSG_CREDENTIAL};
use crate::metadata::ZmqMetadata;
use crate::options::ZmqOptions;
use crate::socket_base::ZmqSocketBase;
use crate::utils::copy_bytes;
use crate::zmtp_engine::{ZmqMechanism, ZmqSessionBase};

// enum
// {
pub const heartbeat_ivl_timer_id: u8 = 0x80;
pub const heartbeat_timeout_timer_id: u8 = 0x81;
pub const heartbeat_ttl_timer_id: u8 = 0x82;
// }

// enum
// {
pub const handshake_timer_id: u8 = 0x40;
// };

// : public io_object_t, public ZmqIEngine
#[derive(Default, Debug, Clone)]
pub struct ZmqStreamEngineBase {
    pub io_object: ZmqIoObject,
    pub engine_interface: ZmqEngineInterface,
    // const ZmqOptions _options;
    pub _options: ZmqOptions,
    // unsigned char *_inpos;
    pub _inpos: usize,
    pub _insize: usize,
    // ZmqDecoderInterface *_decoder;
    //     unsigned char *_outpos;
    pub _outpos: usize,
    // _outsize: usize;
    pub _outsize: usize,
    // ZmqBaseEncoder *_encoder;
    pub _encoder: ZmqBaseEncoder,
    // ZmqMechanism *_mechanism;
    pub _mechanism: ZmqMechanism,
    // int (ZmqStreamEngineBase::*_next_msg) (msg: &mut ZmqMessage);
    pub _next_msg: fn(msg: &mut ZmqMessage) -> i32,
    // int (ZmqStreamEngineBase::*_process_msg) (msg: &mut ZmqMessage);
    pub _process_msg: fn(msg: &mut ZmqMessage) -> i32,
    //  Metadata to be attached to received messages. May be NULL.
    // ZmqMetadata *_metadata;
    pub _metadata: Option<ZmqMetadata>,
    //  True iff the engine couldn't consume the last decoded message.
    pub _input_stopped: bool,
    //  True iff the engine doesn't have any message to encode.
    pub _output_stopped: bool,
    //  Representation of the connected endpoints.
    // const endpoint_uri_ZmqPair _endpoint_uri_pair;
    pub _endpoint_uri_pair: EndpointUriPair,
    //  ID of the handshake timer
    //  True is linger timer is running.
    pub _has_handshake_timer: bool,
    //  Heartbeat stuff
    pub _has_ttl_timer: bool,
    pub _has_timeout_timer: bool,
    pub _has_heartbeat_timer: bool,
    pub _peer_address: String,
    //  Underlying socket.
    // ZmqFileDesc _s;
    pub _s: ZmqFileDesc,
    // handle_t _handle;
    pub _handle: ZmqHandle,
    pub _plugged: bool,
    //  When true, we are still trying to determine whether
    //  the peer is using versioned protocol, and if so, which
    //  version.  When false, normal message flow has started.
    pub _handshaking: bool,
    // ZmqMessage _tx_msg;
    pub _tx_msg: ZmqMessage,
    pub _io_error: bool,
    //  The session this engine is attached to.
    // ZmqSessionBase *_session;
    pub _session: Option<ZmqSessionBase>,
    //  Socket
    // ZmqSocketBase *_socket;
    pub _socket: ZmqSocketBase,
    //  Indicate if engine has an handshake stage, if it does, engine must call session.engine_ready
    //  when handshake is completed.
    pub _has_handshake_stage: bool,
    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqStreamEngineBase)
}

impl ZmqDecoderInterface for ZmqStreamEngineBase {}

impl ZmqStreamEngineBase {
    // bool in_event_internal ();

    pub fn in_event_internal(&mut self) -> bool {
        // zmq_assert (!_io_error);

        //  If still handshaking, receive and process the greeting message.
        if (self._handshaking) {
            if (self.handshake()) {
                //  Handshaking was successful.
                //  Switch into the normal message flow.
                self._handshaking = false;

                if (self._mechanism == null_mut() && self._has_handshake_stage) {
                    self._session.engine_ready();

                    if (self._has_handshake_timer) {
                        cancel_timer(handshake_timer_id);
                        self._has_handshake_timer = false;
                    }
                }
            } else {
                return false;
            }
        }

        // zmq_assert (_decoder);

        //  If there has been an I/O error, stop polling.
        if (self._input_stopped) {
            rm_fd(self._handle);
            self._io_error = true;
            return true; // TODO or return false in this case too?
        }

        //  If there's no data to process in the buffer...
        if (!self._insize) {
            //  Retrieve the buffer and read as much data as possible.
            //  Note that buffer can be arbitrarily large. However, we assume
            //  the underlying TCP layer has fixed buffer size and thus the
            //  number of bytes read will be always limited.
            let mut bufsize = 0;
            self._decoder.get_buffer(&self._inpos, &bufsize);

            // TODO
            // let rc: i32 = read (self._inpos, bufsize);

            if (rc == -1) {
                if (errno != EAGAIN) {
                    // error (connection_error);
                    return false;
                }
                return true;
            }

            //  Adjust input size
            self._insize = rc; //static_cast<size_t> (rc);
            // Adjust buffer size to received bytes
            self._decoder.resize_buffer(self._insize);
        }

        let mut rc = 0;
        let mut processed = 0;

        while (self._insize > 0) {
            rc = self._decoder.decode(self._inpos, self._insize, processed);
            // zmq_assert (processed <= _insize);
            self._inpos += processed;
            self._insize -= processed;
            if (rc == 0 || rc == -1) {
                break;
            }
            rc = self._process_msg(self._decoder.msg());
            if (rc == -1) {
                break;
            }
        }

        //  Tear down the connection if we have failed to decode input data
        //  or the session has rejected the message.
        if (rc == -1) {
            if (errno != EAGAIN) {
                // error (protocol_error);
                return false;
            }
            self._input_stopped = true;
            reset_pollin(_handle);
        }

        self._session.flush();
        return true;
    }

    //  Unplug the engine from the session.
    // void unplug ();
    pub fn unplug(&mut self) {
        // zmq_assert (_plugged);
        self._plugged = false;

        //  Cancel all timers.
        if (self._has_handshake_timer) {
            cancel_timer(handshake_timer_id);
            self._has_handshake_timer = false;
        }

        if (self._has_ttl_timer) {
            cancel_timer(heartbeat_ttl_timer_id);
            self._has_ttl_timer = false;
        }

        if (self._has_timeout_timer) {
            cancel_timer(heartbeat_timeout_timer_id);
            self._has_timeout_timer = false;
        }

        if (self._has_heartbeat_timer) {
            cancel_timer(heartbeat_ivl_timer_id);
            self._has_heartbeat_timer = false;
        }
        //  Cancel all fd subscriptions.
        if (!self._io_error) {
            rm_fd(self._handle);
        }

        //  Disconnect from I/O threads poller object.
        self.io_object.unplug();

        self._session = None;
    }

    // int write_credential (msg: &mut ZmqMessage);

    // void mechanism_ready ();

    pub fn mechanism_ready(&mut self) {
        if (self._options.heartbeat_interval > 0 && !self._has_heartbeat_timer) {
            add_timer(self._options.heartbeat_interval, heartbeat_ivl_timer_id);
            self._has_heartbeat_timer = true;
        }

        if (self._has_handshake_stage) {
            self._session.engine_ready();
        }

        let mut flush_session = false;

        if (self._options.recv_routing_id) {
            let mut routing_id: ZmqMessage = ZmqMessage::default();
            self._mechanism.peer_routing_id(&routing_id);
            let rc: i32 = self._session.push_msg(&routing_id);
            if (rc == -1 && errno == EAGAIN) {
                // If the write is failing at this stage with
                // an EAGAIN the pipe must be being shut down,
                // so we can just bail out of the routing id set.
                return;
            }
            // errno_assert (rc == 0);
            flush_session = true;
        }

        if (self._options.router_notify & ZMQ_NOTIFY_CONNECT) {
            let mut connect_notification = ZmqMessage::default();
            connect_notification.init2();
            let rc: i32 = self._session.push_msg(&connect_notification);
            if (rc == -1 && errno == EAGAIN) {
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

        self._next_msg = self.pull_and_encode;
        self._process_msg = self.write_credential;

        //  Compile metadata.
        let mut properties: properties_t = properties_t::default();
        init_properties(properties);

        //  Add ZAP properties.
        let zap_properties = self._mechanism.get_zap_properties();
        properties.insert(zap_properties.begin(), zap_properties.end());

        //  Add ZMTP properties.
        let zmtp_properties = self._mechanism.get_zmtp_properties();
        properties.insert(zmtp_properties.begin(), zmtp_properties.end());

        // zmq_assert (_metadata == null_mut());
        if (!properties.empty()) {
            self._metadata = ZmqMetadata::with_properties(properties);
            // alloc_assert (_metadata);
        }

        if (self._has_handshake_timer) {
            cancel_timer(handshake_timer_id);
            self._has_handshake_timer = false;
        }

        self._socket.event_handshake_succeeded(&mut self._options, &self._endpoint_uri_pair, 0);
    }

    pub fn pull_and_encode(&mut self, msg: &mut ZmqMessage) -> i32 {
        // zmq_assert (_mechanism != null_mut());

        if (self._session.pull_msg(msg) == -1) {
            return -1;
        }
        if (self._mechanism.encode(msg) == -1) {
            return -1;
        }
        return 0;
    }

    // ZmqStreamEngineBase (fd: ZmqFileDesc,
    // options: &ZmqOptions,
    // const endpoint_uri_ZmqPair &endpoint_uri_pair_,
    // has_handshake_stage_: bool);
    // ZmqStreamEngineBase::ZmqStreamEngineBase (
    // fd: ZmqFileDesc,
    // options: &ZmqOptions,
    // const endpoint_uri_ZmqPair &endpoint_uri_pair_,
    // has_handshake_stage_: bool) :
    // _options (options_),
    // _inpos (null_mut()),
    // _insize (0),
    // _decoder (null_mut()),
    // _outpos (null_mut()),
    // _outsize (0),
    // _encoder (null_mut()),
    // _mechanism (null_mut()),
    // _next_msg (null_mut()),
    // _process_msg (null_mut()),
    // _metadata (null_mut()),
    // _input_stopped (false),
    // _output_stopped (false),
    // _endpoint_uri_pair (endpoint_uri_pair_),
    // _has_handshake_timer (false),
    // _has_ttl_timer (false),
    // _has_timeout_timer (false),
    // _has_heartbeat_timer (false),
    // _peer_address (get_peer_address (fd)),
    // _s (fd),
    // _handle (static_cast<handle_t> (null_mut())),
    // _plugged (false),
    // _handshaking (true),
    // _io_error (false),
    // self._session (null_mut()),
    // _socket (null_mut()),
    // _has_handshake_stage (has_handshake_stage_)
    // {
    // let rc: i32 = _tx_msg.init ();
    // errno_assert (rc == 0);
    //
    // //  Put the socket into non-blocking mode.
    // unblock_socket (_s);
    // }
    pub fn new(
        fd: ZmqFileDesc,
        options: &mut ZmqOptions,
        endpoint_uri_pair: &EndpointUriPair,
        has_handshake_stage: bool,
    ) -> Self {
        let mut out = Self {
            _options: options.clone(),
            _endpoint_uri_pair: endpoint_uri_pair.clone(),
            _peer_address: get_peer_address(fd),
            _s: fd,
            _has_handshake_stage: has_handshake_stage,
            ..Default::default()
        };
        out._tx_msg.init2();
        unblock_socket(out._s);
        out
    }

    // ~ZmqStreamEngineBase () ;

    //  ZmqIEngine interface implementation.
    pub fn has_handshake_stage(&self) -> bool {
        self._has_handshake_stage
    }

    // void plug (ZmqIoThread *io_thread_, ZmqSessionBase *session_) ;
    pub fn plug(&mut self, io_thread_: &mut ZmqIoThread, session: &mut ZmqSessionBase) {
        // zmq_assert (!_plugged);
        self._plugged = true;

        //  Connect to session object.
        // zmq_assert (!_session);
        // zmq_assert (session_);
        self._session = session_;
        self._socket = self._session.get_socket();

        //  Connect to I/O threads poller object.
        self.io_object.plug(io_thread_);
        self._handle = add_fd(self._s);
        self._io_error = false;

        self.plug_internal();
    }

    // void terminate () ;
    pub fn terminate(&mut self) {
        self.unplug();
        // delete this;
    }

    // bool restart_input () ;

    pub fn restart_input(&mut self) -> bool {
        // zmq_assert (_input_stopped);
        // zmq_assert (_session != null_mut());
        // zmq_assert (_decoder != null_mut());

        let mut rc = (self._process_msg)(self._decoder.msg());
        if (rc == -1) {
            if (errno == EAGAIN) {
                self._session.flush();
            } else {
                // error (protocol_error);
                return false;
            }
            return true;
        }

        while (self._insize > 0) {
            let mut processed = 0;
            rc = self._decoder.decode(self._inpos, self._insize, processed);
            // zmq_assert (processed <= _insize);
            self._inpos += processed;
            self._insize -= processed;
            if (rc == 0 || rc == -1) {
                break;
            }
            rc = (self._process_msg)(self._decoder.msg());
            if (rc == -1) {
                break;
            }
        }

        if (rc == -1 && errno == EAGAIN) {
            self._session.flush();
        } else if (self._io_error) {
            // error (connection_error);
            return false;
        } else if (rc == -1) {
            // error (protocol_error);
            return false;
        } else {
            self._input_stopped = false;
            set_pollin();
            self._session.flush();

            //  Speculative read.
            if (!in_event_internal()) {
                return false;
            }
        }

        return true;
    }

    // void restart_output () ;
    pub fn restart_output(&mut self) {
        if (self._io_error) {
            return;
        }

        if (self._output_stopped) {
            self.set_pollout();
            self._output_stopped = false;
        }

        //  Speculative write: The assumption is that at the moment new message
        //  was sent by the user the socket is probably available for writing.
        //  Thus we try to write the data to socket avoiding polling for POLLOUT.
        //  Consequently, the latency should be better in request/reply scenarios.
        self.out_event();
    }

    // void zap_msg_available () ;
    pub fn zap_msg_available(&mut self) {
        // zmq_assert (_mechanism != null_mut());

        let rc: i32 = self._mechanism.zap_msg_available();
        if (rc == -1) {
            // error (protocol_error);
            return;
        }
        if (_input_stopped) {
            if (!self.restart_input()) {
                return;
            }
        }
        if (_output_stopped) {
            self.restart_output();
        }
    }

    // const endpoint_uri_ZmqPair &get_endpoint () const ;
    pub fn get_endpoint(&self) -> &EndpointUriPair {
        &self._endpoint_uri_pair
    }

    //  i_poll_events interface implementation.
    // void in_event () ;
    pub fn in_event(&mut self) {
        // ignore errors
        let res = self.in_event_internal();
        LIBZMQ_UNUSED(res);
    }

    // void out_event () ;

    pub fn out_event(&mut self) {
        // zmq_assert (!_io_error);

        //  If write buffer is empty, try to read new data from the encoder.
        if (!self._outsize) {
            //  Even when we stop polling as soon as there is no
            //  data to send, the poller may invoke out_event one
            //  more time due to 'speculative write' optimisation.
            if (self._encoder == null_mut()) {
                // zmq_assert (_handshaking);
                return;
            }

            self._outpos = 0;
            // TODO
            // self._outsize = self._encoder.encode (&self._outpos, 0);

            while (self._outsize < (self._options.out_batch_size) as usize) {
                if ((self._next_msg)(&mut self._tx_msg) == -1) {
                    //  ws_engine can cause an engine error and delete it, so
                    //  bail out immediately to avoid use-after-free
                    if (errno == ECONNRESET) {
                        return;
                    } else {
                        break;
                    }
                }
                self._encoder.load_msg(&mut self._tx_msg);
                // TODO
                // unsigned char *bufptr = _outpos + _outsize;
                let n = self._encoder.encode(
                    &mut Some(bufptr),
                    (self._options.out_batch_size - self._outsize) as usize,
                );
                // zmq_assert (n > 0);
                if (self._outpos == 0) {
                    self._outpos = bufptr;
                }
                self._outsize += n;
            }

            //  If there is no data to send, stop polling for output.
            if (self._outsize == 0) {
                self._output_stopped = true;
                reset_pollout();
                return;
            }
        }

        //  If there are any data to write in write buffer, write as much as
        //  possible to the socket. Note that amount of data to write can be
        //  arbitrarily large. However, we assume that underlying TCP layer has
        //  limited transmission buffer and thus the actual number of bytes
        //  written should be reasonably modest.
        // TODO
        // let nbytes: i32 = write (self._outpos, self._outsize);

        //  IO error has occurred. We stop waiting for output events.
        //  The engine is not terminated until we detect input error;
        //  this is necessary to prevent losing incoming messages.
        if (nbytes == -1) {
            reset_pollout();
            return;
        }

        self._outpos += nbytes;
        self._outsize -= nbytes;

        //  If we are still handshaking and there are no data
        //  to send, stop polling for output.
        if (self._handshaking) {
            if (self._outsize == 0) {
                reset_pollout();
            }
        }
    }

    // void timer_event (id_: i32) ;

    // typedef ZmqMetadata::dict_t properties_t;

    // bool init_properties (properties_t &properties_);

    //  Function to handle network disconnections.

    // virtual void error (ZmqErrorReason reason_);

    // int next_handshake_command (msg: &mut ZmqMessage);
    pub fn next_handshake_command(&mut self, msg: &mut ZmqMessage) -> i32 {
        // zmq_assert (_mechanism != null_mut());

        if (self._mechanism.status() == ready) {
            mechanism_ready();
            return pull_and_encode(msg);
        }
        if (self._mechanism.status() == ZmqMechanism::error) {
            errno = EPROTO;
            return -1;
        }
        let rc: i32 = self._mechanism.next_handshake_command(msg);

        if (rc == 0) {
            msg.set_flags(ZMQ_MSG_COMMAND);
        }

        return rc;
    }

    // int process_handshake_command (msg: &mut ZmqMessage);
    pub fn process_handshake_command(&mut self, msg: &mut ZmqMessage) -> i32 {
        // zmq_assert (_mechanism != null_mut());
        let rc: i32 = self._mechanism.process_handshake_command(msg);
        if (rc == 0) {
            if (self._mechanism.status() == ready) {
                mechanism_ready();
            } else if (self._mechanism.status() == error) {
                errno = EPROTO;
                return -1;
            }
            if (self._output_stopped) {
                self.restart_output();
            }
        }

        return rc;
    }

    // int pull_msg_from_session (msg: &mut ZmqMessage);

    // int push_ZmqMessageo_session (msg: &mut ZmqMessage);

    // int pull_and_encode (msg: &mut ZmqMessage);

    // virtual int decode_and_push (msg: &mut ZmqMessage);

    // int push_one_then_decode_and_push (msg: &mut ZmqMessage);

    // void set_handshake_timer ();

    // virtual bool handshake () { return true; };

    // virtual void plug_internal (){};

    // virtual int process_command_message (msg: &mut ZmqMessage)
    // {
    // LIBZMQ_UNUSED (msg);
    // return -1;
    // };

    // virtual int produce_ping_message (msg: &mut ZmqMessage)
    // {
    // LIBZMQ_UNUSED (msg);
    // return -1;
    // };

    // virtual int process_heartbeat_message (msg: &mut ZmqMessage)
    // {
    // LIBZMQ_UNUSED (msg);
    // return -1;
    // };

    // virtual int produce_pong_message (msg: &mut ZmqMessage)
    // {
    // LIBZMQ_UNUSED (msg);
    // return -1;
    // };

    // virtual int read (data: &mut [u8], size: usize);

    // virtual int write (const data: &mut [u8], size: usize);

    // void reset_pollout () { io_object_t::reset_pollout (_handle); }
    pub fn reset_pollout(&mut self) {
        self.io_object.reset_pollout(self._handle)
    }

    // void set_pollout () { io_object_t::set_pollout (_handle); }
    pub fn set_pollout(&mut self) {
        self.io_object.set_pollout(self._handle)
    }

    // void set_pollin () { io_object_t::set_pollin (_handle); }
    pub fn set_pollin(&mut self) {
        self.io_object.set_pollin(self._handle)
    }

    // ZmqSessionBase *session () { return self._session; }
    pub fn session(&mut self) -> &mut Option<ZmqSessionBase> {
        &mut self._session
    }

    // ZmqSocketBase *socket () { return _socket; }
    pub fn socket(&mut self) -> &mut ZmqSocketBase {
        &mut self._socket
    }

    pub fn write_credential(&mut self, msg: &mut ZmqMessage) -> i32 {
        // zmq_assert (_mechanism != null_mut());
        // zmq_assert (_session != null_mut());

        let credential = self._mechanism.get_user_id();
        if (credential.size() > 0) {
            let mut msg = ZmqMessage::default();
            rc = msg.init_size(credential.size());
            // zmq_assert (rc == 0);
            copy_bytes(msg.data_mut(), 0, credential.data(), 0, credential.size());
            msg.set_flags(ZMQ_MSG_CREDENTIAL);
            rc = self._session.push_msg(&msg);
            if (rc == -1) {
                rc = msg.close();
                // errno_assert (rc == 0);
                return -1;
            }
        }
        self._process_msg = self.decode_and_push;
        return decode_and_push(msg);
    }

    pub fn decode_and_push(&mut self, msg: &mut ZmqMessage) -> i32 {
        // zmq_assert (_mechanism != null_mut());

        if (self._mechanism.decode(msg) == -1) {
            return -1;
        }

        if (self._has_timeout_timer) {
            self._has_timeout_timer = false;
            cancel_timer(heartbeat_timeout_timer_id);
        }

        if (self._has_ttl_timer) {
            self._has_ttl_timer = false;
            cancel_timer(heartbeat_ttl_timer_id);
        }

        if msg.flags() & ZMQ_MSG_COMMAND {
            process_command_message(msg);
        }

        if (self._metadata) {
            msg.set_metadata(&mut self._metadata.unwrap());
        }
        if (_session.push_msg(msg) == -1) {
            if (errno == EAGAIN) {
                self._process_msg = push_one_then_decode_and_push;
            }
            return -1;
        }
        return 0;
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

    pub fn error(&mut self, reason_: ZmqErrorReason) {
        // zmq_assert (_session);

        if (self._options.router_notify & ZMQ_NOTIFY_DISCONNECT) != 0 && !self._handshaking {
            // For router sockets with disconnect notification, rollback
            // any incomplete message in the pipe, and push the disconnect
            // notification message.
            self._session.rollback();

            let mut disconnect_notification: ZmqMessage = ZmqMessage::default();
            disconnect_notification.init2();
            self._session.push_msg(&disconnect_notification);
        }

        // protocol errors have been signaled already at the point where they occurred
        if (reason_ != protocol_error && (self._mechanism == null_mut() || self._mechanism.status() == ZmqMechanism::handshaking)) {
            let err: i32 = errno;
            self._socket.event_handshake_failed_no_detail(
                &mut self._options,
                &self._endpoint_uri_pair,
                err,
            );
            // special case: connecting to non-ZMTP process which immediately drops connection,
            // or which never responds with greeting, should be treated as a protocol error
            // (i.e. stop reconnect)
            if (((reason_ == connection_error) || (reason_ == timeout_error)) && (self._options.reconnect_stop & ZMQ_RECONNECT_STOP_HANDSHAKE_FAILED) != 0) {
                reason_ = protocol_error;
            }
        }

        self._socket.event_disconnected(&mut self._options, &self._endpoint_uri_pair, self._s);
        self._session.flush();
        self._session.engine_error(
            !self._handshaking && (self._mechanism == null_mut() || self._mechanism.status() != ZmqMechanism::handshaking),
            reason_,
        );
        unplug();
        // delete this;
    }

    pub fn set_handshake_timer(&mut self) {
        // zmq_assert (!_has_handshake_timer);

        if (self._options.handshake_ivl > 0) {
            add_timer(self._options.handshake_ivl, handshake_timer_id);
            self._has_handshake_timer = true;
        }
    }

    pub fn init_properties(&mut self, properties_: &mut properties_t) -> bool {
        if (self._peer_address.empty()) {
            return false;
        }
        properties_.ZMQ_MAP_INSERT_OR_EMPLACE((ZMQ_MSG_PROPERTY_PEER_ADDRESS), _peer_address);

        //  Private property to support deprecated SRCFD
        // std::ostringstream stream;
        // stream <<  (_s);
        // std::string fd_string = stream.str ();
        fd_string = format!("{}", self._s);
        properties_.ZMQ_MAP_INSERT_OR_EMPLACE(std::string("__fd"), ZMQ_MOVE(fd_string));
        return true;
    }

    pub fn timer_event(&mut self, id_: i32) {
        if (id_ == handshake_timer_id) {
            _has_handshake_timer = false;
            //  handshake timer expired before handshake completed, so engine fail
            // error (timeout_error);
        } else if (id_ == heartbeat_ivl_timer_id) {
            _next_msg = &ZmqStreamEngineBase::produce_ping_message;
            out_event();
            add_timer(self._options.heartbeat_interval, heartbeat_ivl_timer_id);
        } else if (id_ == heartbeat_ttl_timer_id) {
            _has_ttl_timer = false;
            // error (timeout_error);
        } else if (id_ == heartbeat_timeout_timer_id) {
            _has_timeout_timer = false;
            // error (timeout_error);
        } else {
            // There are no other valid timer ids!
            assert(false);
        }
    }

    pub fn read(&mut self, data: &mut [u8], size: usize) -> i32 {
        let rc: i32 = tcp_read(self._s, data, size);

        if (rc == 0) {
            // connection closed by peer
            errno = EPIPE;
            return -1;
        }

        return rc;
    }

    pub fn write(&mut self, data: &mut [u8], size: usize) -> i32 {
        return tcp_write(self._s, data, size);
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

// ZmqStreamEngineBase::~ZmqStreamEngineBase ()
// {
//     zmq_assert (!_plugged);
//
//     if (_s != retired_fd) {
// // #ifdef ZMQ_HAVE_WINDOWS
//         let rc: i32 = closesocket (_s);
//         wsa_assert (rc != SOCKET_ERROR);
// // #else
//         int rc = close (_s);
// // #if defined(__FreeBSD_kernel__) || defined(__FreeBSD__)
//         // FreeBSD may return ECONNRESET on close() under load but this is not
//         // an error.
//         if (rc == -1 && errno == ECONNRESET)
//             rc = 0;
// // #endif
//         errno_assert (rc == 0);
// // #endif
//         _s = retired_fd;
//     }
//
//     let rc: i32 = _tx_msg.close ();
//     errno_assert (rc == 0);
//
//     //  Drop reference to metadata and destroy it if we are
//     //  the only user.
//     if (_metadata != null_mut()) {
//         if (_metadata.drop_ref ()) {
//             LIBZMQ_DELETE (_metadata);
//         }
//     }
//
//     LIBZMQ_DELETE (_encoder);
//     LIBZMQ_DELETE (_decoder);
//     LIBZMQ_DELETE (_mechanism);
// }
