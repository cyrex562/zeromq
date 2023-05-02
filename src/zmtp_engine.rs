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
// #include "macros.hpp"

// #include <limits.h>
// #include <string.h>

// #ifndef ZMQ_HAVE_WINDOWS
// #include <unistd.h>
// #endif

// #include <new>
// #include <sstream>

// #include "zmtp_engine.hpp"
// #include "io_thread.hpp"
// #include "session_base.hpp"
// #include "v1_encoder.hpp"
// #include "v1_decoder.hpp"
// #include "v2_encoder.hpp"
// #include "v2_decoder.hpp"
// #include "v3_1_encoder.hpp"
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
// #include "likely.hpp"
// #include "wire.hpp"

use std::mem;
use libc::{EAGAIN, memcmp, memcpy, memset};
use crate::curve_server::curve_server_t;
use crate::endpoint::EndpointUriPair;
use crate::fd::ZmqFileDesc;
use crate::gssapi_client::gssapi_client_t;
use crate::gssapi_server::gssapi_server_t;
use crate::message::{ZMQ_MSG_CANCEL, ZMQ_MSG_COMMAND, ZMQ_MSG_PING, ZMQ_MSG_PONG, ZMQ_MSG_ROUTING_ID, ZMQ_MSG_SUBSCRIBE, ZmqMessage};
use crate::options::ZmqOptions;
use crate::plain_client::plain_client_t;
use crate::plain_server::plain_server_t;
use crate::stream_engine_base::stream_engine_base_t;
use crate::utils::{cmp_bytes, copy_bytes, set_bytes};
use crate::v1_decoder::v1_decoder_t;
use crate::v1_encoder::v1_encoder_t;
use crate::v2_decoder::v2_decoder_t;
use crate::v2_encoder::v2_encoder_t;
use crate::v3_1_encoder::v3_1_encoder_t;
use crate::zmq_hdr::{ZMQ_CURVE, ZMQ_GSSAPI, ZMQ_NULL, ZMQ_PLAIN, ZMQ_PROTOCOL_ERROR_ZMTP_MECHANISM_MISMATCH, ZMQ_PUB, ZMQ_XPUB};
use crate::zmtp_engine::ZmtpRevisions::ZMTP_2_0;

//  Protocol revisions
#[repr(C)]
enum ZmtpRevisions
{
    ZMTP_1_0 = 0,
    ZMTP_2_0 = 1,
    ZMTP_3_x = 3
}

pub struct ZmqThread;
pub struct ZmqSessionBase;
pub struct ZmqMechanism;

// static const size_t signature_size = 10;
const signature_size: usize = 10;

//  Size of ZMTP/1.0 and ZMTP/2.0 greeting message
// static const size_t v2_greeting_size = 12;
const v2_greeting_size: usize = 12;

//  Size of ZMTP/3.0 greeting message
// static const size_t v3_greeting_size = 64;
const v3_greeting_size: usize = 64;

//  Position of the revision and minor fields in the greeting.
const revision_pos: usize = 10;
const minor_pos: usize = 11;

//  This engine handles any socket with SOCK_STREAM semantics,
//  e.g. TCP socket or an UNIX domain socket.
#[derive(Default, Debug, Clone)]
pub struct ZmqZmtpEngine
{
    // : public stream_engine_base_t
    pub stream_engine_base: ZmqStreamEngineBase,
// public:
//     ZmqZmtpEngine (fd: ZmqFileDesc,
//                    options: &ZmqOptions,
//                    const endpoint_uri_pair_t &endpoint_uri_pair_);
//     ~ZmqZmtpEngine ();
//
//   protected:
//     //  Detects the protocol used by the peer.
//     bool handshake ();
//
//     void plug_internal ();
//
//     int process_command_message (msg: &mut ZmqMessage);
//     int produce_ping_message (msg: &mut ZmqMessage);
//     int process_heartbeat_message (msg: &mut ZmqMessage);
//     int produce_pong_message (msg: &mut ZmqMessage);

    // private:
    //  Receive the greeting from the peer.
    // int receive_greeting ();
    // void receive_greeting_versioned ();
    //
    // typedef bool (ZmqZmtpEngine::*handshake_fun_t) ();
    // static handshake_fun_t select_handshake_fun (unversioned: bool,
    //                                              unsigned char revision,
    //                                              unsigned char minor);
    //
    // bool handshake_v1_0_unversioned ();
    // bool handshake_v1_0 ();
    // bool handshake_v2_0 ();
    // bool handshake_v3_x (downgrade_sub: bool);
    // bool handshake_v3_0 ();
    // bool handshake_v3_1 ();
    // int routing_id_msg (msg: &mut ZmqMessage);
    // int process_routing_id_msg (msg: &mut ZmqMessage);
    // ZmqMessage _routing_id_msg;
    pub _routing_id_msg: ZmqMessage,
    //  Need to store PING payload for PONG
    // ZmqMessage _pong_msg;
    pub _pong_msg: ZmqMessage,
    //  Expected greeting size.
    pub _greeting_size: usize,
    //  Greeting received from, and sent to peer
    // unsigned char _greeting_recv[v3_greeting_size];
    pub _greeting_recv: [u8; v3_greeting_size],
    // pub _greeting_recv_major: ZmtpRevisions,
    // pub _greeting_recv_minor: u8,
    // unsigned char _greeting_send[v3_greeting_size];
    pub _greeting_send: [u8; v3_greeting_size],
    // pub _greeting_send_major: ZmtpRevisions,
    // pub _greeting_send_minor: u8,
    //  Size of greeting received so far
    // unsigned int _greeting_bytes_read;
    pub _greeting_bytes_read: usize,
    //  Indicates whether the engine is to inject a phantom
    //  subscription message into the incoming stream.
    //  Needed to support old peers.
    pub _subscription_required: bool,
    pub _heartbeat_timeout: i32,
    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqZmtpEngine)
}

impl ZmqZmtpEngine {
    // ZmqZmtpEngine::ZmqZmtpEngine (
    //   fd: ZmqFileDesc,
    //   options: &ZmqOptions,
    //   const endpoint_uri_pair_t &endpoint_uri_pair_) :
    //     stream_engine_base_t (fd, options_, endpoint_uri_pair_, true),
    //     _greeting_size (v2_greeting_size),
    //     _greeting_bytes_read (0),
    //     _subscription_required (false),
    //     _heartbeat_timeout (0)
    pub fn new(fd: ZmqFileDesc, options: &ZmqOptions, endpoint_uri_pair: EndpointUriPair) -> Self
    {
        let mut out = Self {..Default::default()};

        // TODO
        // out._next_msg = static_cast<int (stream_engine_base_t::*) (ZmqMessage *)> (
        //     &ZmqZmtpEngine::routing_id_msg);

        // TODO
        // _process_msg = static_cast<int (stream_engine_base_t::*) (ZmqMessage *)> (
        //     &ZmqZmtpEngine::process_routing_id_msg);
        // let _process_msg = &mut out._process_routing_id_msg;

        let mut rc = out._pong_msg.init2();
        // errno_assert (rc == 0);

        rc = out._routing_id_msg.init2();
        // errno_assert (rc == 0);

        if (options.heartbeat_interval > 0) {
            out._heartbeat_timeout = options.heartbeat_timeout;
            if (out._heartbeat_timeout == -1) {
                out._heartbeat_timeout = options.heartbeat_interval;
            }
        }

        out
    }


    // ZmqZmtpEngine::~ZmqZmtpEngine ()
    // {
    //     let rc: i32 = _routing_id_msg.close ();
    //     errno_assert (rc == 0);
    // }

    pub fn plug_internal (&mut self)
    {
        // start optional timer, to prevent handshake hanging on no input
        self.set_handshake_timer ();

        //  Send the 'length' and 'flags' fields of the routing id message.
        //  The 'length' field is encoded in the long format.
        self._outpos = self._greeting_send.clone();
        self._outpos[self._outsize+= 1] = UCHAR_MAX;
        put_uint64(&self._outpos[self._outsize], self._options.routing_id_size + 1);
        self._outsize += 8;
        self._outpos[_outsize+= 1] = 0x7f;

        set_pollin();
        set_pollout();
        //  Flush all the data that may have been already received downstream.
        in_event();
    }

    pub fn handshake (&mut self) -> bool
    {
        // zmq_assert (_greeting_bytes_read < _greeting_size);
        //  Receive the greeting.
        let rc: i32 = receive_greeting ();
        if (rc == -1) {
            return false;
        }
        let unversioned = rc != 0;

        if (!(self.select_handshake_fun (unversioned, self._greeting_recv[revision_pos] as ZmtpRevisions,
                                 self._greeting_recv[minor_pos])) ()) {
            return false;
        }

        // Start polling for output if necessary.
        if (self._outsize == 0) {
            set_pollout();
        }

        return true;
    }


    pub fn receive_greeting (&mut self) -> i32
    {
        let mut unversioned = false;
        while (self._greeting_bytes_read < self._greeting_size) {
            // TODO
            // let n: i32 = read (self._greeting_recv + self._greeting_bytes_read,
            //                    self._greeting_size - self._greeting_bytes_read);
            if (n == -1) {
                if (errno != EAGAIN) {
                    // error(connection_error);
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

            if (self._greeting_bytes_read < signature_size) {
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
            receive_greeting_versioned ();
        }
        return if unversioned { 1 } else { 0 };
    }



    pub fn receive_greeting_versioned (&mut self)
    {
        //  Send the major version number.
        if (self._outpos + self._outsize == self._greeting_send + signature_size) {
            if (self._outsize == 0) {
                set_pollout();
            }
            self._outpos[self._outsize+= 1] = 3; //  Major version number
        }

        if self._greeting_bytes_read > signature_size {
            if (self._outpos + self._outsize == self._greeting_send + signature_size + 1) {
                if (self._outsize == 0) {
                    set_pollout();
                }

                //  Use ZMTP/2.0 to talk to older peers.
                if (self._greeting_recv[revision_pos] == ZmtpRevisions::ZMTP_1_0 as u8
                    || self._greeting_recv[revision_pos] == ZMTP_2_0 as u8) {
                    self._outpos[self._outsize += 1] = self._options.type_;
                }
                else {
                    self._outpos[self._outsize+= 1] = 1; //  Minor version number
                    // memset (_outpos + _outsize, 0, 20);
                    zero_bytes(self._outpos, self._outsize, 20);

                    zmq_assert (self._options.mechanism == ZMQ_NULL
                        || self._options.mechanism == ZMQ_PLAIN
                        || self._options.mechanism == ZMQ_CURVE
                        || self._options.mechanism == ZMQ_GSSAPI);

                    if (self._options.mechanism == ZMQ_NULL) {
                        copy_bytes(self._outpos, self._outsize, b"NULL", 0, 4);
                    }
                    else if (self._options.mechanism == ZMQ_PLAIN) {
                        copy_bytes(self._outpos, self._outsize, b"PLAIN",0, 5);
                    }
                    else if (self._options.mechanism == ZMQ_GSSAPI) {
                        copy_bytes(self._outpos, self._outsize, b"GSSAPI",0, 6);
                    }
                    else if (self._options.mechanism == ZMQ_CURVE) {
                        copy_bytes(self._outpos, self._outsize, b"CURVE", 0,5);
                    }
                    self._outsize += 20;
                    set_bytes(self._outpos, self._outsize, 0, 32);
                    self._outsize += 32;
                    self._greeting_size = v3_greeting_size;
                }
            }
        }
    }

    pub fn select_handshake_fun (&mut self,
                                 unversioned_: bool, revision_: ZmtpRevisions, minor_: u8) -> handshake_fun_t
    {
        //  Is the peer using ZMTP/1.0 with no revision number?
        if (unversioned_) {
            return handshake_v1_0_unversioned;
        }
        match revision_ {
            ZmtpRevisions::ZMTP_1_0 => handshake_v1_0,
            ZmtpRevisions::ZMTP_2_0=> handshake_v2_0,
            ZmtpRevisions::ZMTP_3_x => {
                match minor_
                {
                    0 => handshake_v3_0,
                    _ => handshake_v3_1,
                }
            }
            _ => handshake_v3_1,
        }
    }



    pub fn handshake_v1_0_unversioned (&mut self) -> bool
    {
        //  We send and receive rest of routing id message
        if (session ().zap_enabled ()) {
            // reject ZMTP 1.0 connections if ZAP is enabled
            // error (protocol_error);
            return false;
        }

        self._encoder = v1_encoder_t::new(self._options.out_batch_size);
        // alloc_assert (_encoder);

        self._decoder = v1_decoder_t::new(self._options.in_batch_size, self._options.maxmsgsize);
        // alloc_assert (_decoder);

        //  We have already sent the message header.
        //  Since there is no way to tell the encoder to
        //  skip the message header, we simply throw that
        //  header data away.
        let header_size =
        if self._options.routing_id_size + 1 >= u8::MAX { 10 } else { 2 };
        // unsigned char tmp[10], *bufferp = tmp;
        let mut tmp: [u8;10] = [0;10];
        let bufferp = &mut tmp;

        //  Prepare the routing id message and load it into encoder.
        //  Then consume bytes we have already sent to the peer.
        let mut rc = self._routing_id_msg.close ();
        // zmq_assert (rc == 0);
        let rc2 = self._routing_id_msg.init_size (self._options.routing_id_size);
        // zmq_assert (rc == 0);
        copy_bytes(self._routing_id_msg.data_mut(), 0,self._options.routing_id,
                0, self._options.routing_id_size);
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
        self._next_msg = &self.pull_msg_from_session;

        //  We are expecting routing id message.
        self._process_msg = self.process_routing_id_msg;

        return true;
    }

    pub fn handshake_v1_0(&mut self) -> bool
    {
        if (session().zap_enabled()) {
            // reject ZMTP 1.0 connections if ZAP is enabled
            // error (protocol_error);
            return false;
        }

        self._encoder = v1_encoder_t::new(self._options.out_batch_size);
        // alloc_assert (self._encoder);

        self._decoder = v1_decoder_t::new(self._options.in_batch_size, self._options.maxmsgsize);
        // alloc_assert (self._decoder);

        return true;
    }

    pub fn handshake_v2_0 (&mut self) -> bool
    {
        if (session().zap_enabled ()) {
            // reject ZMTP 2.0 connections if ZAP is enabled
            // error (protocol_error);
            return false;
        }

        self._encoder = v2_encoder_t::new(self._options.out_batch_size);
        // alloc_assert (self._encoder);

        self._decoder = v2_decoder_t::new(
            self._options.in_batch_size, self._options.maxmsgsize, self._options.zero_copy);
        // alloc_assert (self._decoder);

        return true;
    }


    pub fn handshake_v3_x (&mut self, downgrade_sub_: bool) -> bool
    {
        if self._options.mechanism == ZMQ_NULL
            && cmp_bytes (&self._greeting_recv, 12, b"NULL\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                       0,20) == 0 {
            self._mechanism = ZmqMechanism::new(session(), self._peer_address, self._options);
            // alloc_assert (self._mechanism);
        } else if (self._options.mechanism == ZMQ_PLAIN
            && cmp_bytes (&self._greeting_recv, 12,
                       b"PLAIN\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",0, 20)
            == 0) {
            if (self._options.as_server) {
                self._mechanism = plain_server_t::new(session(), self._peer_address, self._options);
            }
            else {
                self._mechanism = plain_client_t::new(session(), self._options);
            }
            // alloc_assert (self._mechanism);
        }
// #ifdef ZMQ_HAVE_CURVE
        else if (self._options.mechanism == ZMQ_CURVE
            && cmp_bytes (&self._greeting_recv, 12,
                       b"CURVE\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0", 0,20)
            == 0) {
            if (self._options.as_server) {
                self._mechanism =  curve_server_t::new(
                    session(), self._peer_address, self._options, downgrade_sub_);
            }
            else {
                self._mechanism =  curve_client_t::new(session(), self._options, downgrade_sub_);
            }
            // alloc_assert (self._mechanism);
        }
// #endif
// #ifdef HAVE_LIBGSSAPI_KRB5
        else if (self._options.mechanism == ZMQ_GSSAPI
            && cmp_bytes (&self._greeting_recv, 12,
                       b"GSSAPI\0\0\0\0\0\0\0\0\0\0\0\0\0\0", 0,20)
            == 0) {
            if (self._options.as_server) {
                self._mechanism = gssapi_server_t::new(session(), self._peer_address, self._options);
            }
            else {
                self._mechanism = gssapi_client_t::new(session(), self._options);
            }
            // alloc_assert (self._mechanism);
        }
// #endif
        else {
            // TODO
            // socket().event_handshake_failed_protocol (
            //     session ().get_endpoint (),
            //     ZMQ_PROTOCOL_ERROR_ZMTP_MECHANISM_MISMATCH);
            // error (protocol_error);
            return false;
        }
        self._next_msg = &ZmqZmtpEngine::next_handshake_command;
        self._process_msg = &ZmqZmtpEngine::process_handshake_command;

        return true;
    }

    pub fn handshake_v3_0 (&mut self) -> bool
    {
        self._encoder = v2_encoder_t::new(self._options.out_batch_size);
        // alloc_assert (self._encoder);

        self._decoder = v2_decoder_t::new(
            self._options.in_batch_size, self._options.maxmsgsize, self._options.zero_copy);
        // alloc_assert (self._decoder);

        return self.handshake_v3_x (true);
    }

    pub fn handshake_v3_1 (&mut self) -> bool
    {
        self._encoder = v3_1_encoder_t::new(self._options.out_batch_size);
        // alloc_assert (self._encoder);

        self._decoder = v2_decoder_t::new(
            self._options.in_batch_size, self._options.maxmsgsize, self._options.zero_copy);
        // alloc_assert (self._decoder);

        return self.handshake_v3_x (false);
    }

    pub fn routing_id_msg (&mut self, msg: &mut ZmqMessage) -> i32
    {
        let rc: i32 = msg.init_size (self._options.routing_id_size);
        // errno_assert (rc == 0);
        if (self._options.routing_id_size > 0) {
            copy_bytes(msg.data_mut(), 0, self._options.routing_id, 0,self._options.routing_id_size);
        }
        self._next_msg = &ZmqZmtpEngine::pull_msg_from_session;
        return 0;
    }


    pub fn process_routing_id_msg (&mut self, msg: &mut ZmqMessage) -> i32
    {
        if (self._options.recv_routing_id) {
            msg.set_flags (ZMQ_MSG_ROUTING_ID);
            let rc: i32 = session ().push_msg (msg);
            errno_assert (rc == 0);
        } else {
            let rc = msg.close ();
            // errno_assert (rc == 0);
            let rc2 = msg.init2();
            // errno_assert (rc == 0);
        }

        if (self._subscription_required) {
            let mut subscription = ZmqMessage::new();

            //  Inject the subscription message, so that also
            //  ZMQ 2.x peers receive published messages.
            let rc = subscription.init_size (1);
            // errno_assert (rc == 0);
            subscription.data_mut()[0] = 1;
            rc = session ().push_msg (&subscription);
            // errno_assert (rc == 0);
        }

        self._process_msg = push_ZmqMessageo_session;

        return 0;
    }

    pub fn produce_ping_message (&mut self, msg: &mut ZmqMessage) -> i32
    {
        // 16-bit TTL + \4PING == 7
        let ping_ttl_len = ZmqMessage::PING_CMD_NAME_SIZE + 2;
        // zmq_assert (self._mechanism != null_mut());

        let mut rc = msg.init_size (ping_ttl_len);
        // errno_assert (rc == 0);
        msg.set_flags (ZMQ_MSG_COMMAND);
        // Copy in the command message
        copy_bytes (msg.data_mut(), 0, b"\4PING", 0,ZmqMessage::PING_CMD_NAME_SIZE);

        let ttl_val = i32::to_be(self._options.heartbeat_ttl);
        copy_bytes(msg.data_mut(), ZmqMessage::PING_CMD_NAME_SIZE,
                &ttl_val.to_le_bytes(), 0, mem::size_of_val(&ttl_val));

        rc = self._mechanism.encode (msg);
        self._next_msg = &ZmqZmtpEngine::pull_and_encode;
        if (!_has_timeout_timer && self._heartbeat_timeout > 0) {
            add_timer (self._heartbeat_timeout, heartbeat_timeout_timer_id);
            self._has_timeout_timer = true;
        }
        return rc;
    }


    pub fn produce_pong_message (&mut self, msg: &mut ZmqMessage) -> i32
    {
        // zmq_assert (self._mechanism != null_mut());
        // TODO
        // let mut rc = msg.move (self._pong_msg);
        // errno_assert (rc == 0);

        rc = self._mechanism.encode (msg);
        self._next_msg = &ZmqZmtpEngine::pull_and_encode;
        return rc;
    }


    pub fn process_heartbeat_message (&mut self, msg: &mut ZmqMessage) -> i32
    {
        if (msg.is_ping ()) {
            // 16-bit TTL + \4PING == 7
            let ping_ttl_len = ZmqMessage::PING_CMD_NAME_SIZE + 2;
            let ping_max_ctx_len = 16;
            let mut remote_heartbeat_ttl = 0u16;

            // Get the remote heartbeat TTL to setup the timer
            // copy_bytes(&mut remote_heartbeat_ttl.,
            //         static_cast<uint8_t *> (msg.data ())
            //             + ZmqMessage::PING_CMD_NAME_SIZE,
            //         ping_ttl_len - ZmqMessage::PING_CMD_NAME_SIZE);
            remote_heartbeat_ttl = u16::from_ne_bytes(msg.data()[ZmqMessage::PING_CMD_NAME_SIZE..ZmqMessage::PING_CMD_NAME_SIZE+ping_ttl_len]);

            remote_heartbeat_ttl = (remote_heartbeat_ttl.to_be());
            // The remote heartbeat is in 10ths of a second
            // so we multiply it by 100 to get the timer interval in ms.
            remote_heartbeat_ttl *= 100;

            if (!_has_ttl_timer && remote_heartbeat_ttl > 0) {
                add_timer (remote_heartbeat_ttl, heartbeat_ttl_timer_id);
                self._has_ttl_timer = true;
            }

            //  As per ZMTP 3.1 the PING command might contain an up to 16 bytes
            //  context which needs to be PONGed back, so build the pong message
            //  here and store it. Truncate it if it's too long.
            //  Given the engine goes straight to out_event, sequential PINGs will
            //  not be a problem.
            let context_len =
                usize::min(msg.size () - ping_ttl_len, ping_max_ctx_len);
            let rc: i32 =
                self._pong_msg.init_size (ZmqMessage::PING_CMD_NAME_SIZE + context_len);
            // errno_assert (rc == 0);
            self._pong_msg.set_flags (ZMQ_MSG_COMMAND);
            copy_bytes(self._pong_msg.data_mut(), 0,b"\x04PONG", 0,ZmqMessage::PING_CMD_NAME_SIZE);
            if (context_len > 0) {
                copy_bytes(self._pong_msg.data_mut(),
                    ZmqMessage::PING_CMD_NAME_SIZE,
                       msg.data(), ping_ttl_len,
                       context_len);
            }

            self._next_msg = self.produce_pong_message;
            out_event ();
        }

        return 0;
    }

    pub fn process_command_message (&mut self, msg: &mut ZmqMessage) -> i32
    {
        let cmd_name_size = msg.data ()[0];
        let ping_name_size = ZmqMessage::PING_CMD_NAME_SIZE - 1;
        let sub_name_size = ZmqMessage::SUB_CMD_NAME_SIZE - 1;
        let cancel_name_size = ZmqMessage::CANCEL_CMD_NAME_SIZE - 1;
        //  Malformed command
        if msg.size () < (cmd_name_size + mem::size_of_val(&cmd_name_size)) as usize {
            return -1;
        }

        let cmd_name = msg.data ()[1..];
        if cmd_name_size == ping_name_size
            && cmp_bytes(&cmd_name, 0, b"PING", 0, cmd_name_size as usize) == 0 {
            msg.set_flags(ZMQ_MSG_PING);
        }
        if (cmd_name_size == ping_name_size
            && cmp_bytes(&cmd_name, 0, b"PONG", 0, cmd_name_size as usize) == 0) {
            msg.set_flags(ZMQ_MSG_PONG);
        }
        if (cmd_name_size == sub_name_size
            && cmp_bytes (&cmd_name, 0, b"SUBSCRIBE", 0, cmd_name_size as usize) == 0) {
            msg.set_flags(ZMQ_MSG_SUBSCRIBE);
        }
        if (cmd_name_size == cancel_name_size
            && cmp_bytes (&cmd_name, 0, b"CANCEL", 0, cmd_name_size as usize) == 0) {
            msg.set_flags(ZMQ_MSG_CANCEL);
        }

        if (msg.is_ping () || msg.is_pong ()) {
            return process_heartbeat_message(msg);
        }

        return 0;
    }


}







