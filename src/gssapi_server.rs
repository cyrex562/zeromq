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

// #ifdef HAVE_LIBGSSAPI_KRB5

// #include <string.h>
// #include <string>

// #include "msg.hpp"
// #include "session_base.hpp"
// #include "err.hpp"
// #include "gssapi_server.hpp"
// #include "wire.hpp"

use std::ptr::null_mut;

use libc::{EAGAIN, EPROTO};

use crate::defines::ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND;
use crate::gssapi_mechanism_base::ZmqGssApiMechanismBase;
use crate::mechanism::ZmqMechanismStatus;
use crate::message::ZmqMessage;
use crate::options::ZmqOptions;
use crate::session_base::ZmqSessionBase;
use crate::zap_client::ZmqZapClient;

enum GssApiServerState {
    send_next_token,
    recv_next_token,
    expect_zap_reply,
    send_ready,
    recv_ready,
    connected,
}

// #include <gssapi/gssapi.h>
#[derive(Default, Debug, Clone)]
pub struct ZmqGssApiServer<'a> {
    //  : public ZmqGssApiMechanismBase,
    pub mechanism_base: ZmqGssApiMechanismBase,
    // public ZmqZapClient
    pub zap_client: ZmqZapClient,
    // ZmqSessionBase * const session;
    pub session: &'a mut ZmqSessionBase,
    //  const peer_address: String;
    //  Current FSM self.state
    pub state: GssApiServerState,
    //  True iff server considers the client authenticated pub self.state: GssApiServerState,
    pub security_context_established: bool,
    //  The underlying mechanism type (ignored)
    // gss_OID doid;
    pub doid: gss_OID,
}

impl ZmqGssApiServer {
    // ZmqGssApiServer (ZmqSessionBase *session_, const std::string &peer_address,
    // options: &ZmqOptions);

    pub fn new(session: &mut ZmqSessionBase, peer_address: &str, options: &mut ZmqOptions) -> Self {
        // ZmqMechanismBase (session_, options_),
        //     ZmqGssApiMechanismBase (session_, options_),
        //     ZmqZapClient (session_, peer_address_, options_),
        //     session (session_),
        //     peer_address (peer_address_),
        //     self.state (recv_next_token),
        //     self.security_context_established (false)
        let mut out = Self {
            mechanism_base: ZmqGssApiMechanismBase::new(session, options),
            zap_client: ZmqZapClient(session, peer_address, options),
            session: session,
            state: GssApiServerState::recv_next_token,
            security_context_established: false,
            ..Default::default()
        };
        out.mechanism_base.maj_stat = GSS_S_CONTINUE_NEEDED;
        if (!options_.gss_principal.empty()) {
            let principal_size = options_.gss_principal.len();
            out.mechanism_base.principal_name = String::with_capacity(principal_size + 1);
            // assert (principal_name);
            // memcpy (principal_name, options_.gss_principal,
            // principal_size + 1);
            principal_name = options.gss_principal.clone();
            let name_type = convert_nametype(options_.gss_principal_nt);
            if (acquire_credentials(&out.mechanism_base.principal_name, &cred, name_type) != 0) {
                out.mechanism_base.maj_stat = GSS_S_FAILURE;
            }
        }
        out
    }
    // ~ZmqGssApiServer () ;

    // mechanism implementation
    // int next_handshake_command (msg: &mut ZmqMessage) ;
    pub fn next_handshake_command(&mut self, msg: &mut ZmqMessage) -> i32 {
        if (self.state == GssApiServerState::send_ready) {
            let rc = produce_ready(msg);
            if (rc == 0) {
                self.state = GssApiServerState::recv_ready;
            }

            return rc;
        }

        if (self.state != GssApiServerState::send_next_token) {
            errno = EAGAIN;
            return -1;
        }

        if (produce_next_token(msg) < 0) {
            return -1;
        }

        if (maj_stat != GSS_S_CONTINUE_NEEDED && maj_stat != GSS_S_COMPLETE) {
            return -1;
        }

        if (maj_stat == GSS_S_COMPLETE) {
            self.security_context_established = true;
        }

        self.state = GssApiServerState::recv_next_token;

        return 0;
    }

    // int process_handshake_command (msg: &mut ZmqMessage) ;

    pub fn process_handshake_command(&mut self, msg: &mut ZmqMessage) -> i32 {
        if (self.state == GssApiServerState::recv_ready) {
            let rc = process_ready(msg);
            if (rc == 0) {
                self.state = GssApiServerState::connected;
            }

            return rc;
        }

        if (self.state != GssApiServerState::recv_next_token) {
            session.get_socket().event_handshake_failed_protocol(
                session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND,
            );
            errno = EPROTO;
            return -1;
        }

        if (self.security_context_established) {
            //  Use ZAP protocol (RFC 27) to authenticate the user.
            //  Note that rc will be -1 only if ZAP is not set up, but if it was
            //  requested and it does not work properly the program will abort.
            let mut expecting_zap_reply = false;
            let mut rc = session.zap_connect();
            if (rc == 0) {
                send_zap_request();
                rc = receive_and_process_zap_reply();
                if (rc != 0) {
                    if (rc == -1) {
                        return -1;
                    }
                    expecting_zap_reply = true;
                }
            }
            self.state = if expecting_zap_reply {
                GssApiServerState::expect_zap_reply
            } else {
                GssApiServerState::send_ready
            };
            return 0;
        }

        if (process_next_token(msg) < 0) {
            return -1;
        }

        accept_context();
        self.state = GssApiServerState::send_next_token;

        // errno_assert (msg.close () == 0);
        // errno_assert (msg.init () == 0);

        return 0;
    }

    // int encode (msg: &mut ZmqMessage) ;
    pub fn encode(&mut self, msg: &mut ZmqMessage) -> i32 {
        // zmq_assert (self.state == connected);

        if (self.do_encryption) {
            return self.mechanism_base.encode_message(msg);
        }

        return 0;
    }

    // int decode (msg: &mut ZmqMessage) ;
    pub fn decode(&mut self, msg: &mut ZmqMessage) -> i32 {
        // zmq_assert (self.state == connected);

        if (do_encryption) {
            return decode_message(msg);
        }

        return 0;
    }

    // status_t status () const ;
    pub fn status(&mut self) -> ZmqMechanismStatus {
        return if self.state == GssApiServerState::connected {
            ZmqMechanismStatus::ready
        } else {
            ZmqMechanismStatus::handshaking
        };
    }

    // void send_zap_request ();
    pub fn send_zap_request(&mut self) {
        let principal: gss_buffer_desc = ();
        gss_display_name(&min_stat, target_name, &principal, null_mut());
        self.zap_client
            .send_zap_request("GSSAPI", 6, (principal.value), principal.length);

        gss_release_buffer(&min_stat, &principal);
    }

    // int zap_msg_available () ;
    pub fn zap_msg_available(&mut self) -> i32 {
        if (self.state != GssApiServerState::expect_zap_reply) {
            errno = EFSM;
            return -1;
        }
        let rc: i32 = receive_and_process_zap_reply();
        if (rc == 0) {
            self.state = GssApiServerState::send_ready;
        }
        return if rc == -1 { -1 } else { 0 };
    }

    // int produce_next_token (msg: &mut ZmqMessage);
    pub fn produce_next_token(&mut self, msg: &mut ZmqMessage) -> i32 {
        if (send_tok.length != 0) {
            // Client expects another token
            if (self
                .mechanism_base
                .produce_initiate(msg, send_tok.value, send_tok.length)
                < 0)
            {
                return -1;
            }
            gss_release_buffer(&min_stat, &send_tok);
        }

        if (maj_stat != GSS_S_COMPLETE && maj_stat != GSS_S_CONTINUE_NEEDED) {
            gss_release_name(&min_stat, &target_name);
            if (context != GSS_C_NO_CONTEXT) {
                gss_delete_sec_context(&min_stat, &context, GSS_C_NO_BUFFER);
            }
            return -1;
        }

        return 0;
    }

    // int process_next_token (msg: &mut ZmqMessage);
    pub fn process_next_token(&mut self, msg: &mut ZmqMessage) -> i32 {
        if (maj_stat == GSS_S_CONTINUE_NEEDED) {
            if (process_initiate(msg, &recv_tok.value, recv_tok.length) < 0) {
                if (target_name != GSS_C_NO_NAME) {
                    gss_release_name(&min_stat, &target_name);
                }
                return -1;
            }
        }

        return 0;
    }

    // void accept_context ();
    pub fn accept_context(&mut self) {
        maj_stat = gss_accept_sec_context(
            &init_sec_min_stat,
            &context,
            cred,
            &recv_tok,
            GSS_C_NO_CHANNEL_BINDINGS,
            &target_name,
            &doid,
            &send_tok,
            &ret_flags,
            null_mut(),
            null_mut(),
        );

        if (recv_tok.value) {
            // free(recv_tok.value);
            recv_tok.value = null_mut();
        }
    }
}
