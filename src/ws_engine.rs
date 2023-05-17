/*
Copyright (c) 2007-2019 Contributors as noted in the AUTHORS file

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

// #ifdef ZMQ_USE_NSS
// #include <secoid.h>
// #include <sechash.h>
// #define SHA_DIGEST_LENGTH 20
// #elif defined ZMQ_USE_BUILTIN_SHA1
// #include "../external/sha1/sha1.h"
// #elif defined ZMQ_USE_GNUTLS
// #define SHA_DIGEST_LENGTH 20
// #include <gnutls/gnutls.h>
// #include <gnutls/crypto.h>
// #endif

// #if !defined ZMQ_HAVE_WINDOWS
// #include <sys/types.h>
// #include <unistd.h>
// #include <sys/socket.h>
// #include <netinet/in.h>
// #include <arpa/inet.h>
// #ifdef ZMQ_HAVE_VXWORKS
// #include <sockLib.h>
// #endif
// #endif

// #include <cstring>

// #include "compat.hpp"
// #include "tcp.hpp"
// #include "ws_engine.hpp"
// #include "session_base.hpp"
// #include "err.hpp"
// #include "ip.hpp"
// #include "random.hpp"
// #include "ws_decoder.hpp"
// #include "ws_encoder.hpp"
// #include "null_mechanism.hpp"
// #include "plain_server.hpp"
// #include "plain_client.hpp"

// #ifdef ZMQ_HAVE_CURVE
// #include "curve_client.hpp"
// #include "curve_server.hpp"
// #endif

//  OSX uses a different name for this socket option
// #ifndef IPV6_ADD_MEMBERSHIP
// #define IPV6_ADD_MEMBERSHIP IPV6_JOIN_GROUP
// #endif

// #ifdef __APPLE__
// #include <TargetConditionals.h>
// #endif


use std::io::Read;
use std::ptr::{hash, null_mut};

use bincode::options;
use libc::{EAGAIN, ECONNRESET, memcpy, memset, strcmp};

use crate::curve_server::curve_server_t;
use crate::defines::{ZMQ_CURVE, ZMQ_NULL, ZMQ_PLAIN, ZMQ_PROTOCOL_ERROR_WS_UNSPECIFIED};
use crate::endpoint::EndpointUriPair;
use crate::fd::ZmqFileDesc;
use crate::mechanism::ZmqMechanism;
use crate::mechanism::ZmqMechanismStatus::error;
use crate::message::{ZMQ_MSG_COMMAND, ZMQ_MSG_PING, ZMQ_MSG_PONG, ZMQ_MSG_ROUTING_ID, ZmqMessage};
use crate::null_mechanism::ZmqNullMechanism;
use crate::options::ZmqOptions;
use crate::plain_client::PlainClient;
use crate::plain_server::PlainServer;
use crate::stream_engine_base::{heartbeat_ivl_timer_id, heartbeat_timeout_timer_id, ZmqStreamEngineBase};
use crate::utils::{copy_bytes, set_bytes};
use crate::ws_address::WsAddress;
use crate::ws_decoder::ws_decoder_t;
use crate::ws_encoder::ws_encoder_t;
use crate::ws_engine::ws_client_handshake_state::{client_handshake_complete, client_handshake_end_line_cr, client_handshake_error, client_handshake_initial, client_header_field_begin_name, client_header_field_colon, client_header_field_cr, client_header_field_name, client_header_field_value, client_header_field_value_trailing_space, response_line_cr, response_line_H, response_line_HT, response_line_HTT, response_line_HTTP, response_line_HTTP_slash, response_line_HTTP_slash_1, response_line_HTTP_slash_1_dot, response_line_HTTP_slash_1_dot_1, response_line_HTTP_slash_1_dot_1_space, response_line_p, response_line_pr, response_line_pro, response_line_prot, response_line_proto, response_line_protoc, response_line_protoco, response_line_protocol, response_line_protocols, response_line_s, response_line_status_1, response_line_status_10, response_line_status_101, response_line_status_101_space, response_line_sw, response_line_swi, response_line_swit, response_line_switc, response_line_switch, response_line_switchi, response_line_switchin, response_line_switching, response_line_switching_space};
use crate::ws_engine::ws_server_handshake_state::{handshake_complete, handshake_end_line_cr, handshake_error, handshake_initial, header_field_begin_name, header_field_colon, header_field_cr, header_field_name, header_field_value, header_field_value_trailing_space, request_line_cr, request_line_G, request_line_GE, request_line_GET, request_line_GET_space, request_line_H, request_line_HT, request_line_HTT, request_line_HTTP, request_line_HTTP_slash, request_line_HTTP_slash_1, request_line_HTTP_slash_1_dot, request_line_HTTP_slash_1_dot_1, request_line_resource, request_line_resource_space};

enum ws_server_handshake_state {
    handshake_initial = 0,
    request_line_G,
    request_line_GE,
    request_line_GET,
    request_line_GET_space,
    request_line_resource,
    request_line_resource_space,
    request_line_H,
    request_line_HT,
    request_line_HTT,
    request_line_HTTP,
    request_line_HTTP_slash,
    request_line_HTTP_slash_1,
    request_line_HTTP_slash_1_dot,
    request_line_HTTP_slash_1_dot_1,
    request_line_cr,
    header_field_begin_name,
    header_field_name,
    header_field_colon,
    header_field_value_trailing_space,
    header_field_value,
    header_field_cr,
    handshake_end_line_cr,
    handshake_complete,

    handshake_error = -1,
}


enum ws_client_handshake_state {
    client_handshake_initial = 0,
    response_line_H,
    response_line_HT,
    response_line_HTT,
    response_line_HTTP,
    response_line_HTTP_slash,
    response_line_HTTP_slash_1,
    response_line_HTTP_slash_1_dot,
    response_line_HTTP_slash_1_dot_1,
    response_line_HTTP_slash_1_dot_1_space,
    response_line_status_1,
    response_line_status_10,
    response_line_status_101,
    response_line_status_101_space,
    response_line_s,
    response_line_sw,
    response_line_swi,
    response_line_swit,
    response_line_switc,
    response_line_switch,
    response_line_switchi,
    response_line_switchin,
    response_line_switching,
    response_line_switching_space,
    response_line_p,
    response_line_pr,
    response_line_pro,
    response_line_prot,
    response_line_proto,
    response_line_protoc,
    response_line_protoco,
    response_line_protocol,
    response_line_protocols,
    response_line_cr,
    client_header_field_begin_name,
    client_header_field_name,
    client_header_field_colon,
    client_header_field_value_trailing_space,
    client_header_field_value,
    client_header_field_cr,
    client_handshake_end_line_cr,
    client_handshake_complete,
    client_handshake_error = -1,
}

#[derive(Default, Debug, Clone)]
pub struct ZmqWsEngine {
    pub stream_engine_base: ZmqStreamEngineBase,
    pub _client: bool,
    pub address: WsAddress,
    pub _client_handshake_state: ws_client_handshake_state,
    pub _server_handshake_state: ws_server_handshake_state,
    pub _read_buffer: [u8; WS_BUFFER_SIZE],
    pub _write_buffer: Vec<u8>,
    //[u8;WS_BUFFER_SIZE],
    pub _header_name: [u8; MAX_HEADER_NAME_LENGTH + 1],
    pub _header_name_position: i32,
    pub _header_value: [u8; MAX_HEADER_VALUE_LENGTH + 1],
    pub _header_value_position: i32,
    pub _header_upgrade_websocket: bool,
    pub _header_connection_upgrade: bool,
    pub _websocket_protocol: [u8; 256],
    pub _websocket_key: [u8; MAX_HEADER_VALUE_LENGTH + 1],
    pub _websocket_accept: [u8; MAX_HEADER_VALUE_LENGTH + 1],
    pub _heartbeat_timeout: i32,
    pub _close_msg: ZmqMessage,
}

impl ZmqWsEngine {
    // ZmqWsEngine (fd: ZmqFileDesc,
    //              options: &ZmqOptions,
    //              const endpoint_uri_ZmqPair &endpoint_uri_pair_,
    //              const WsAddress &address_,
    //              client_: bool);
    pub fn new(fd: ZmqFileDesc,
               options: &mut ZmqOptions,
               endpoint_uri_pair_: &EndpointUriPair,
               address_: &mut WsAddress,
               client_: bool) -> Self {
        // ZmqStreamEngineBase (fd, options_, endpoint_uri_pair_, true),
        //     _client (client_),
        //     address (address_),
        //     _client_handshake_state (client_handshake_initial),
        //     self._server_handshake_state (handshake_initial),
        //     _header_name_position (0),
        //     _header_value_position (0),
        //     _header_upgrade_websocket (false),
        //     _header_connection_upgrade (false),
        //     _heartbeat_timeout (0)
        let mut out = Self {
            stream_engine_base: ZmqStreamEngineBase::new(fd, options, endpoint_uri_pair_, true),
            _client: client_,
            address: address_.clone(),
            _client_handshake_state: ws_client_handshake_state::client_handshake_initial,
            _server_handshake_state: ws_server_handshake_state::handshake_initial,
            _header_name_position: 0,
            _header_value_position: 0,
            _header_upgrade_websocket: false,
            _header_connection_upgrade: false,
            _heartbeat_timeout: 0,
            ..Default::default()
        };
        set_bytes(&mut out._websocket_key, 0, 0, MAX_HEADER_VALUE_LENGTH + 1);
        set_bytes(&mut out._websocket_accept, 0, 0, MAX_HEADER_VALUE_LENGTH + 1);
        set_bytes(&mut out._websocket_protocol, 0, 0, 256);

        out._next_msg = &ZmqWsEngine::next_handshake_command;
        out._process_msg = &ZmqWsEngine::process_handshake_command;
        out._close_msg.init2();

        if (out._options.heartbeat_interval > 0) {
            out._heartbeat_timeout = out._options.heartbeat_timeout;
            if (out._heartbeat_timeout == -1) {
                out._heartbeat_timeout = out._options.heartbeat_interval;
            }
        }
        out
    }

    // ~ZmqWsEngine ();

    // int decode_and_push (msg: &mut ZmqMessage);

    // int process_command_message (msg: &mut ZmqMessage);

    // int produce_pong_message (msg: &mut ZmqMessage);

    // int produce_ping_message (msg: &mut ZmqMessage);

    // bool handshake ();

    // void plug_internal ();

    // void start_ws_handshake ();

    // int routing_id_msg (msg: &mut ZmqMessage);

    // int process_routing_id_msg (msg: &mut ZmqMessage);

    // int produce_close_message (msg: &mut ZmqMessage);

    // int produce_no_msg_after_close (msg: &mut ZmqMessage);

    // int close_connection_after_close (msg: &mut ZmqMessage);

    // bool select_protocol (protocol: &str);

    // bool client_handshake ();

    // bool server_handshake ();

    pub fn plug_internal(&mut self) {
        start_ws_handshake();
        set_pollin();
        in_event();
    }

    pub fn routing_id_msg(&mut self, msg: &mut ZmqMessage) -> i32 {
        let rc: i32 = msg.init_size(self._options.routing_id_size);
        // errno_assert (rc == 0);
        if (self._options.routing_id_size > 0) {
            copy_bytes(msg.data_mut(), 0, self._options.routing_id, 0, self._options.routing_id_size);
        }
        self._next_msg = &ZmqWsEngine::pull_msg_from_session;

        return 0;
    }


    pub fn start_ws_handshake(&mut self) {
        if (self._client) {
            let mut protocol: &str = "";
            if (self._options.mechanism == ZMQ_NULL) {
                protocol = "ZWS2.0/NULL,ZWS2.0";
            } else if (self._options.mechanism == ZMQ_PLAIN) {
                protocol = "ZWS2.0/PLAIN";
            }
// #ifdef ZMQ_HAVE_CURVE
            else if (self._options.mechanism == ZMQ_CURVE) {
                protocol = "ZWS2.0/CURVE";
            }
// #endif
            else {
                // Avoid uninitialized variable error breaking UWP build
                protocol = "";
                assert(false);
            }

            let mut nonce: [u8; 16] = [0; 16];
            let p = (nonce);

            // The nonce doesn't have to be secure one, it is just use to avoid proxy cache
            *p = generate_random();
            *(p + 1) = generate_random();
            *(p + 2) = generate_random();
            *(p + 3) = generate_random();

            let mut size = encode_base64(nonce, 16, self._websocket_key, MAX_HEADER_VALUE_LENGTH);
            // assert (size > 0);

            self._write_buffer = format!(
          "GET {} HTTP/1.1\r\n" \
          "Host: {}\r\n" \
          "Upgrade: websocket\r\n" \
          "Connection: Upgrade\r\n" \  
          "Sec-WebSocket-Key: {}\r\n" \
          "Sec-WebSocket-Protocol: {}\r\n" \
          "Sec-WebSocket-Version: 13\r\n\r\n",
          address.path(), address.host(), self._websocket_key, protocol).into_bytes();
            // assert (size > 0 && size < WS_BUFFER_SIZE);
            // TODO:
            // self._outpos = self._write_buffer;
            self._outsize = size;
            set_pollout();
        }
    }

    pub fn process_routing_id_msg(&mut self, msg: &mut ZmqMessage) -> i32 {
        if (self._options.recv_routing_id) {
            msg.set_flags(ZMQ_MSG_ROUTING_ID);
            let rc: i32 = session().push_msg(msg);
            // errno_assert (rc == 0);
        } else {
            let mut rc = msg.close();
            // errno_assert (rc == 0);
            rc = msg.init2();
            // errno_assert (rc == 0);
        }

        self._process_msg = push_msg_to_session;

        return 0;
    }


    pub fn select_protocol(&mut self, options: &mut ZmqOptions, protocol_: &str) -> bool {
        if (self._options.mechanism == ZMQ_NULL && "ZWS2.0" == protocol_) {
            self._next_msg = (&ZmqWsEngine::routing_id_msg);
            self._process_msg = (&ZmqWsEngine::process_routing_id_msg);

            // No mechanism in place, enabling heartbeat
            if (self._options.heartbeat_interval > 0 && !_has_heartbeat_timer) {
                add_timer(self._options.heartbeat_interval, heartbeat_ivl_timer_id);
                self._has_heartbeat_timer = true;
            }

            return true;
        }

        if (self._options.mechanism.is_none() && ("ZWS2.0/NULL" == protocol_)) {
            self._mechanism = ZmqNullMechanism::new(session(), self._peer_address, self._options);
            // alloc_assert (_mechanism);
            return true;
        } else if (self._options.mechanism == ZMQ_PLAIN && ("ZWS2.0/PLAIN" == protocol_)) {
            if (self._options.as_server) {
                self._mechanism = PlainServer::new(session(), self._peer_address, self._options);
            } else {
                self._mechanism = PlainClient::new(session(), self._options);
            }
            // alloc_assert (_mechanism);
            return true;
        }
// #ifdef ZMQ_HAVE_CURVE
        else if (self._options.mechanism == ZMQ_CURVE && ("ZWS2.0/CURVE" == protocol_)) {
            if (self._options.as_server) {
                self._mechanism = curve_server_t(session(), self._peer_address, self._options, false);
            } else {
                self._mechanism = curve_client_t(session(), self._options, false);
            }
            // alloc_assert (_mechanism);
            return true;
        }
// #endif

        return false;
    }


    pub fn handshake(&mut self) -> bool {
        let mut complete = false;

        if (self._client) {
            complete = self.client_handshake();
        } else {
            complete = self.server_handshake();
        }

        if (complete) {
            self._encoder = ws_encoder_t::new(self._options.out_batch_size, self._client);
            // alloc_assert (_encoder);

            self._decoder = ws_decoder_t::new(self._options.in_batch_size, self._options.maxmsgsize,
                                              self._options.zero_copy, !self._client);
            // alloc_assert (_decoder);

            self.socket().event_handshake_succeeded(self._endpoint_uri_pair, 0);

            set_pollout();
        }

        return complete;
    }


    pub fn server_handshake(&mut self) -> bool {
        let nbytes = self.read(&mut self._read_buffer, WS_BUFFER_SIZE)?;
        if (nbytes == -1) {
            if (errno != EAGAIN) {}
            // error (ZmqIEngine::connection_error);
            return false;
        }

        self._inpos = self._read_buffer;
        self._insize = nbytes;

        while (self._insize > 0) {
            let c = (*self._inpos);

            match (self._server_handshake_state) {
                handshake_initial => {
                    if (c == 'G') {
                        self._server_handshake_state = request_line_G;
                    } else {
                        self._server_handshake_state = handshake_error;
                    }
                    //
                }
                request_line_G => {
                    if (c == 'E') {
                        self._server_handshake_state = request_line_GE;
                    } else {
                        self._server_handshake_state = handshake_error;
                    }
                }

                request_line_GE => {
                    if (c == 'T') {
                        self._server_handshake_state = request_line_GET;
                    } else {
                        self._server_handshake_state = handshake_error;
                    }
                }

                request_line_GET => {
                    if (c == ' ') {
                        self._server_handshake_state = request_line_GET_space;
                    } else {
                        self._server_handshake_state = handshake_error;
                    }
                }
                //
                request_line_GET_space => {
                    if (c == '\r' || c == '\n') {
                        self._server_handshake_state = handshake_error;
                    }
                    // TODO: instead of check what is not allowed check what is allowed
                    if (c != ' ') {
                        self._server_handshake_state = request_line_resource;
                    } else {
                        self._server_handshake_state = request_line_GET_space;
                    }
                }
                //
                request_line_resource => {
                    if (c == '\r' || c == '\n') {
                        self._server_handshake_state = handshake_error;
                    } else if (c == ' ') {
                        self._server_handshake_state = request_line_resource_space;
                    } else {
                        self._server_handshake_state = request_line_resource;
                    }
                }
                //
                request_line_resource_space => {
                    if (c == 'H') {
                        self._server_handshake_state = request_line_H;
                    } else {
                        self._server_handshake_state = handshake_error;
                    }
                }

                request_line_H => {
                    if (c == 'T') {
                        self._server_handshake_state = request_line_HT;
                    } else {
                        self._server_handshake_state = handshake_error;
                    }
                }
                //
                request_line_HT => {
                    if (c == 'T') {
                        self._server_handshake_state = request_line_HTT;
                    } else {
                        self._server_handshake_state = handshake_error;
                    }
                }

                request_line_HTT => {
                    if (c == 'P') {
                        self._server_handshake_state = request_line_HTTP;
                    } else {
                        self._server_handshake_state = handshake_error;
                    }
                }

                request_line_HTTP => {
                    if (c == '/') {
                        self._server_handshake_state = request_line_HTTP_slash;
                    } else {
                        self._server_handshake_state = handshake_error;
                    }
                }

                request_line_HTTP_slash => {
                    if (c == '1') {
                        self._server_handshake_state = request_line_HTTP_slash_1;
                    } else {
                        self._server_handshake_state = handshake_error;
                    }
                }

                request_line_HTTP_slash_1 => {
                    if (c == '.') {
                        self._server_handshake_state = request_line_HTTP_slash_1_dot;
                    } else {
                        self._server_handshake_state = handshake_error;
                    }
                }

                request_line_HTTP_slash_1_dot => {
                    if (c == '1') {
                        self._server_handshake_state = request_line_HTTP_slash_1_dot_1;
                    } else {
                        self._server_handshake_state = handshake_error;
                    }
                }

                request_line_HTTP_slash_1_dot_1 => {
                    if (c == '\r') {
                        self._server_handshake_state = request_line_cr;
                    } else {
                        self._server_handshake_state = handshake_error;
                    }
                }

                request_line_cr => {
                    if (c == '\n') {
                        self._server_handshake_state = header_field_begin_name;
                    } else {
                        self._server_handshake_state = handshake_error;
                    }
                }

                header_field_begin_name => {
                    match c {
                        '\r' => self._server_handshake_state = handshake_end_line_cr,
                        '\n' => self._server_handshake_state = handshake_error,

                        _ => {
                            self._header_name[0] = c;
                            self._header_name_position = 1;
                            self._server_handshake_state = header_field_name;
                        }
                    }
                }

                header_field_name => {
                    if (c == '\r' || c == '\n') {
                        self._server_handshake_state = handshake_error;
                    } else if (c == ':') {
                        self._header_name[_header_name_position] = 0;
                        self._server_handshake_state = header_field_colon;
                    } else if (self._header_name_position + 1 > MAX_HEADER_NAME_LENGTH) {
                        self._server_handshake_state = handshake_error;
                    } else {
                        self._header_name[_header_name_position] = c;
                        self._header_name_position += 1;
                        self._server_handshake_state = header_field_name;
                    }
                }

                header_field_colon | header_field_value_trailing_space => {
                    if (c == '\n') {
                        self._server_handshake_state = handshake_error;
                    } else if (c == '\r') {
                        self._server_handshake_state = header_field_cr;
                    } else if (c == ' ') {
                        self._server_handshake_state = header_field_value_trailing_space;
                    } else {
                        self._header_value[0] = c;
                        self._header_value_position = 1;
                        self._server_handshake_state = header_field_value;
                    }
                }

                header_field_value => {
                    if (c == '\n') {
                        self._server_handshake_state = handshake_error;
                    } else if (c == '\r') {
                        self._header_value[_header_value_position] = 0;

                        if (("upgrade" == self._header_name)) {
                            self._header_upgrade_websocket = ("websocket" == self._header_value);
                        } else if (("connection" == self._header_name)) {
                            char * rest = null_mut();
                            char * element = strtok_r(self._header_value, ",", &rest);
                            while (element != null_mut()) {
                                while (*element == ' ') {
                                    element += 1;
                                }
                                if (("upgrade" == element)) {
                                    self._header_connection_upgrade = true;
                                }
                                element = strtok_r(null_mut(), ",", &rest);
                            }
                        } else if (("Sec-WebSocket-Key" == self._header_name)) {
                            strcpy_s(self._websocket_key, self._header_value);
                        } else if (("Sec-WebSocket-Protocol" == self._header_name)) {
                            // Currently only the ZWS2.0 is supported
                            // Sec-WebSocket-Protocol can appear multiple times or be a comma separated list
                            // if _websocket_protocol is already set we skip the check
                            if (self._websocket_protocol[0] == 0) {
                                char * rest = null_mut();
                                char * p = strtok_r(self._header_value, ",", &rest);
                                while (p != null_mut()) {
                                    if (*p == ' ') {
                                        p += 1;
                                    }

                                    if (select_protocol(p)) {
                                        strcpy_s(self._websocket_protocol, p);
                                    }

                                    p = strtok_r(null_mut(), ",", &rest);
                                }
                            }
                        }

                        self._server_handshake_state = header_field_cr;
                    } else if (self._header_value_position + 1 > MAX_HEADER_VALUE_LENGTH) {
                        self._server_handshake_state = handshake_error;
                    } else {
                        self._header_value[_header_value_position] = c;
                        self._header_value_position += 1;
                        self._server_handshake_state = header_field_value;
                    }
                }
                header_field_cr => {
                    if (c == '\n') {
                        self._server_handshake_state = header_field_begin_name;
                    } else {
                        self._server_handshake_state = handshake_error;
                    }
                }

                handshake_end_line_cr => {
                    if (c == '\n') {
                        if (self._header_connection_upgrade && self._header_upgrade_websocket && self._websocket_protocol[0] != 0 && self._websocket_key[0] != 0) {
                            self._server_handshake_state = handshake_complete;

                            let hash: [u8; SHA_DIGEST_LENGTH] = [0; SHA_DIGEST_LENGTH];
                            compute_accept_key(&self._websocket_key, &hash);

                            let accept_key_len: i32 = encode_base64(
                                hash, SHA_DIGEST_LENGTH, self._websocket_accept,
                                MAX_HEADER_VALUE_LENGTH);
                            // assert (accept_key_len > 0);
                            self._websocket_accept[accept_key_len] = 0;

                            self._write_buffer = format!(                                                        "HTTP/1.1 101 Switching Protocols\r\n" \
                                                        "Upgrade: websocket\r\n" \
                                                        "Connection: Upgrade\r\n" \
                                                        "Sec-WebSocket-Accept: %s\r\n" \
                                                        "Sec-WebSocket-Protocol: %s\r\n" \
                                                        "\r\n",
                                                        self._websocket_accept, self._websocket_protocol).into_bytes();
                            // assert(written >= 0 && written < WS_BUFFER_SIZE);
                            // TODO
                            // self._outpos = self._write_buffer;
                            self._outsize = written;

                            self._inpos += 1;
                            self._insize -= 1;

                            return true;
                        }
                        self._server_handshake_state = handshake_error;
                    } else {
                        self._server_handshake_state = handshake_error;
                    }
                }

                _ => {}
                // assert (false);
            }

            self._inpos += 1;
            self._insize -= 1;

            if (self._server_handshake_state == handshake_error) {
                // TODO: send bad request

                self.socket().event_handshake_failed_protocol(
                    self._endpoint_uri_pair, ZMQ_PROTOCOL_ERROR_WS_UNSPECIFIED);

                // error (ZmqIEngine::protocol_error);
                return false;
            }
        }
        return false;
    }


    pub fn client_handshake(&mut self) -> bool {
        let nbytes = self.read(&mut self._read_buffer, WS_BUFFER_SIZE).unwrap();
        if (nbytes == -1) {
            if (errno != EAGAIN) {}
            // error (ZmqIEngine::connection_error);
            return false;
        }

        self._inpos = self._read_buffer;
        self._insize = nbytes;

        while (self._insize > 0) {
            let c = (*_inpos);

            match (self._client_handshake_state) {
                client_handshake_initial => {
                    if (c == 'H') {
                        self._client_handshake_state = response_line_H;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }
                response_line_H => {
                    if (c == 'T') {
                        self._client_handshake_state = response_line_HT;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_HT => {
                    if (c == 'T') {
                        self._client_handshake_state = response_line_HTT;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_HTT => {
                    if (c == 'P') {
                        self._client_handshake_state = response_line_HTTP;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_HTTP => {
                    if (c == '/') {
                        self._client_handshake_state = response_line_HTTP_slash;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_HTTP_slash => {
                    if (c == '1') {
                        self._client_handshake_state = response_line_HTTP_slash_1;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_HTTP_slash_1 => {
                    if (c == '.') {
                        self._client_handshake_state = response_line_HTTP_slash_1_dot;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_HTTP_slash_1_dot => {
                    if (c == '1') {
                        self._client_handshake_state = response_line_HTTP_slash_1_dot_1;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_HTTP_slash_1_dot_1 => {
                    if (c == ' ') {
                        self._client_handshake_state = response_line_HTTP_slash_1_dot_1_space;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_HTTP_slash_1_dot_1_space => {
                    if (c == ' ') {
                        self._client_handshake_state = response_line_HTTP_slash_1_dot_1_space;
                    } else if (c == '1') {
                        self._client_handshake_state = response_line_status_1;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_status_1 => {
                    if (c == '0') {
                        self._client_handshake_state = response_line_status_10;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_status_10 => {
                    if (c == '1') {
                        self._client_handshake_state = response_line_status_101;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_status_101 => {
                    if (c == ' ') {
                        self._client_handshake_state = response_line_status_101_space;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_status_101_space => {
                    if (c == ' ') {
                        self._client_handshake_state = response_line_status_101_space;
                    } else if (c == 'S') {
                        self._client_handshake_state = response_line_s;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_s => {
                    if (c == 'w') {
                        self._client_handshake_state = response_line_sw;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_sw => {
                    if (c == 'i') {
                        self._client_handshake_state = response_line_swi;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_swi => {
                    if (c == 't') {
                        self._client_handshake_state = response_line_swit;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_swit => {
                    if (c == 'c') {
                        self._client_handshake_state = response_line_switc;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_switc => {
                    if (c == 'h') {
                        self._client_handshake_state = response_line_switch;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_switch => {
                    if (c == 'i') {
                        self._client_handshake_state = response_line_switchi;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_switchi => {
                    if (c == 'n') {
                        self._client_handshake_state = response_line_switchin;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_switchin => {
                    if (c == 'g') {
                        self._client_handshake_state = response_line_switching;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_switching => {
                    if (c == ' ') {
                        self._client_handshake_state = response_line_switching_space;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_switching_space => {
                    if (c == 'P') {
                        self._client_handshake_state = response_line_p;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_p => {
                    if (c == 'r') {
                        self._client_handshake_state = response_line_pr;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_pr => {
                    if (c == 'o') {
                        self._client_handshake_state = response_line_pro;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_pro => {
                    if (c == 't') {
                        self._client_handshake_state = response_line_prot;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_prot => {
                    if (c == 'o') {
                        self._client_handshake_state = response_line_proto;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_proto => {
                    if (c == 'c') {
                        self._client_handshake_state = response_line_protoc;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_protoc => {
                    if (c == 'o') {
                        self._client_handshake_state = response_line_protoco;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_protoco => {
                    if (c == 'l') {
                        self._client_handshake_state = response_line_protocol;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_protocol => {
                    if (c == 's') {
                        self._client_handshake_state = response_line_protocols;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_protocols => {
                    if (c == '\r') {
                        self._client_handshake_state = response_line_cr;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                response_line_cr => {
                    if (c == '\n') {
                        self._client_handshake_state = client_header_field_begin_name;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                client_header_field_begin_name => {
                    match (c) {
                        '\r' => self._client_handshake_state = client_handshake_end_line_cr,
                        '\n' => self._client_handshake_state = client_handshake_error,
                        _ => {
                            self._header_name[0] = c;
                            self._header_name_position = 1;
                            self._client_handshake_state = client_header_field_name;
                        }
                    }
                }

                client_header_field_name => {
                    if (c == '\r' || c == '\n') {
                        self._client_handshake_state = client_handshake_error;
                    } else if (c == ':') {
                        self._header_name[_header_name_position] = 0;
                        self._client_handshake_state = client_header_field_colon;
                    } else if (self._header_name_position + 1 > MAX_HEADER_NAME_LENGTH) {
                        self._client_handshake_state = client_handshake_error;
                    } else {
                        self._header_name[_header_name_position] = c;
                        self._header_name_position += 1;
                        self._client_handshake_state = client_header_field_name;
                    }
                }

                client_header_field_colon | client_header_field_value_trailing_space => {
                    if (c == '\n') {
                        self._client_handshake_state = client_handshake_error;
                    } else if (c == '\r') {
                        self._client_handshake_state = client_header_field_cr;
                    } else if (c == ' ') {
                        self._client_handshake_state = client_header_field_value_trailing_space;
                    } else {
                        self._header_value[0] = c;
                        self._header_value_position = 1;
                        self._client_handshake_state = client_header_field_value;
                    }
                }

                client_header_field_value => {
                    if (c == '\n') {
                        self._client_handshake_state = client_handshake_error;
                    } else if (c == '\r') {
                        self._header_value[_header_value_position] = 0;

                        if (("upgrade" == self._header_name)) {
                            self._header_upgrade_websocket = ("websocket" == self._header_value);
                        } else if (("connection" == self._header_name)) {
                            self._header_connection_upgrade = ("upgrade" == self._header_value);
                        } else if (("Sec-WebSocket-Accept" == self._header_name)) {
                            self._websocket_accept = self._header_value;
                        } else if (("Sec-WebSocket-Protocol" == self._header_name)) {
                            if (self._mechanism) {
                                self._client_handshake_state = client_handshake_error;
                            }
                            if (select_protocol(self._header_value)) {
                                strcpy_s(self._websocket_protocol, self._header_value);
                            }
                        }
                        self._client_handshake_state = client_header_field_cr;
                    } else if (self._header_value_position + 1 > MAX_HEADER_VALUE_LENGTH) {
                        self._client_handshake_state = client_handshake_error;
                    } else {
                        self._header_value[_header_value_position] = c;
                        self._header_value_position += 1;
                        self._client_handshake_state = client_header_field_value;
                    }
                }

                client_header_field_cr => {
                    if (c == '\n') {
                        self._client_handshake_state = client_header_field_begin_name;
                    } else {
                        self._client_handshake_state = client_handshake_error;
                    }
                }

                client_handshake_end_line_cr => {
                    if (c == '\n') {
                        if (self._header_connection_upgrade && self._header_upgrade_websocket && self._websocket_protocol[0] != 0 && self._websocket_accept[0] != 0) {
                            self._client_handshake_state = client_handshake_complete;

                            // TODO: validate accept key

                            self._inpos += 1;
                            self._insize -= 1;

                            return true;
                        }
                        self._client_handshake_state = client_handshake_error;
                    } else { self._client_handshake_state = client_handshake_error; }
                }
                _ => {
                    // assert(false);
                }
            }

            self._inpos += 1;
            self._insize -= 1;

            if (self._client_handshake_state == client_handshake_error) {
                self.socket().event_handshake_failed_protocol(
                    self._endpoint_uri_pair, ZMQ_PROTOCOL_ERROR_WS_UNSPECIFIED);

                // error (ZmqIEngine::protocol_error);
                return false;
            }
        }

        return false;
    }

    pub fn decode_and_push(&mut self, msg: &mut ZmqMessage) -> i32 {
        // zmq_assert (_mechanism != null_mut());

        //  with WS engine, ping and pong commands are control messages and should not go through any mechanism
        if (msg.is_ping() || msg.is_pong() || msg.is_close_cmd()) {
            if (process_command_message(msg) == -1) {
                return -1;
            }
        } else if (self._mechanism.decode(msg) == -1) {
            return -1;
        }

        if (self._has_timeout_timer) {
            self._has_timeout_timer = false;
            cancel_timer(heartbeat_timeout_timer_id);
        }

        if (msg.flags() & ZMQ_MSG_COMMAND != 0 && !msg.is_ping() && !msg.is_pong() && !msg.is_close_cmd()) {
            process_command_message(msg);
        }

        if (self._metadata) {
            msg.set_metadata(self._metadata);
        }
        if (session().push_msg(msg) == -1) {
            if (errno == EAGAIN) {
                self._process_msg = &ZmqWsEngine::push_one_then_decode_and_push;
            }
            return -1;
        }
        return 0;
    }

    pub fn produce_close_message(&mut self, msg: &mut ZmqMessage) -> i32 {
        // let rc = msg.move (self._close_msg);
        // errno_assert (rc == 0);
        self._close_msg = msg.clone();

        self._next_msg = (
            &ZmqWsEngine::produce_no_msg_after_close);

        return rc;
    }

    pub fn produce_no_msg_after_close(&mut self, msg: &mut ZmqMessage) -> i32 {
        // LIBZMQ_UNUSED (msg);
        self._next_msg = (
            &ZmqWsEngine::close_connection_after_close);

        errno = EAGAIN;
        return -1;
    }

    pub fn close_connection_after_close(&mut self, msg: &mut ZmqMessage) -> i32 {
        // LIBZMQ_UNUSED (msg);
        // error (connection_error);
        errno = ECONNRESET;
        return -1;
    }

    pub fn produce_ping_message(&mut self, msg: &mut ZmqMessage) -> i32 {
        msg.init2();
        // errno_assert (rc == 0);
        msg.set_flags(ZMQ_MSG_COMMAND | ZMQ_MSG_PING);

        self._next_msg = &ZmqWsEngine::pull_and_encode;
        if (!_has_timeout_timer && self._heartbeat_timeout > 0) {
            add_timer(self._heartbeat_timeout, heartbeat_timeout_timer_id);
            self._has_timeout_timer = true;
        }

        // return rc;
        0
    }

    pub fn produce_pong_message(&mut self, msg: &mut ZmqMessage) -> i32 {
        msg.init2();
        // errno_assert (rc == 0);
        msg.set_flags(ZMQ_MSG_COMMAND | ZMQ_MSG_PONG);

        self._next_msg = &ZmqWsEngine::pull_and_encode;
        return 0;
    }

    pub fn process_command_message(&mut self, msg: &mut ZmqMessage) -> i32 {
        if (msg.is_ping()) {
            self._next_msg = (
                &ZmqWsEngine::produce_pong_message);
            out_event();
        } else if (msg.is_close_cmd()) {
            // let rc = self._close_msg.copy (*msg);
            self._close_msg = msg.clone();
            // errno_assert (rc == 0);
            self._next_msg = (
                &ZmqWsEngine::produce_close_message);
            out_event();
        }

        return 0;
    }


    pub fn encode_base64(in_: &mut [u8], in_len_: i32, out_: &str, out_len_: i32) -> i32 {
        let base64enc_tab: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

        let mut io = 0;
        let mut v = 0;
        let mut rem = 0;

        // for (int ii = 0; ii < in_len_; ii+= 1)
        for ii in 0..in_len_ {
            let ch = in_[ii];
            v = (v << 8) | ch;
            rem += 8;
            while (rem >= 6) {
                rem -= 6;
                if (io >= out_len_) {
                    return -1;
                } /* truncation is failure */
                out_[io += 1] = base64enc_tab[(v >> rem) & 63];
            }
        }
        if (rem) {
            v <<= (6 - rem);
            if (io >= out_len_) {
                return -1;
            } /* truncation is failure */
            out_[io += 1] = base64enc_tab[v & 63];
        }
        while (io & 3) {
            if (io >= out_len_) {
                return -1;
            } /* truncation is failure */
            out_[io += 1] = '=';
        }
        if (io >= out_len_) {
            return -1;
        } /* no room for null terminator */
        out_[io] = 0;
        return io;
    }
} // impl ws_engine

// static int
// encode_base64 (const in_: &mut [u8], in_len_: i32, char *out_, out_len_: i32);
//
// static void compute_accept_key (char *key_,
//                                 unsigned char hash_[SHA_DIGEST_LENGTH]);


// ZmqWsEngine::~ZmqWsEngine ()
// {
//     _close_msg.close ();
// }


pub fn compute_accept_key(key_: &[u8], hash_: &[u8]) {
    let magic_string: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    // TODO
// // #ifdef ZMQ_USE_NSS
//     let mut len = 0u32;
//     let hash_type = HASH_GetHashTypeByOidTag (SEC_OID_SHA1);
//     HASHContext *ctx = HASH_Create (hash_type);
//     assert (ctx);
//
//     HASH_Begin (ctx);
//     HASH_Update (ctx,  key_, (unsigned int) strlen (key_));
//     HASH_Update (ctx,  magic_string,
//                  (unsigned int) strlen (magic_string));
//     HASH_End (ctx, hash_, &len, SHA_DIGEST_LENGTH);
//     HASH_Destroy (ctx);
// #elif defined ZMQ_USE_BUILTIN_SHA1
//     sha1_ctxt ctx;
//     SHA1_Init (&ctx);
//     SHA1_Update (&ctx,  key_, strlen (key_));
//     SHA1_Update (&ctx,  magic_string, strlen (magic_string));
//
//     SHA1_Final (hash_, &ctx);
// #elif defined ZMQ_USE_GNUTLS
//     gnutls_hash_hd_t hd;
//     gnutls_hash_init (&hd, GNUTLS_DIG_SHA1);
//     gnutls_hash (hd, key_, strlen (key_));
//     gnutls_hash (hd, magic_string, strlen (magic_string));
//     gnutls_hash_deinit (hd, hash_);
// // #else
// #error "No sha1 implementation set"
// #endif
}
