// /*
//     Copyright (c) 2007-2016 Contributors as noted in the AUTHORS file

//     This file is part of libzmq, the ZeroMQ core engine in C+= 1.

//     libzmq is free software; you can redistribute it and/or modify it under
//     the terms of the GNU Lesser General Public License (LGPL) as published
//     by the Free Software Foundation; either version 3 of the License, or
//     (at your option) any later version.

//     As a special exception, the Contributors give you permission to link
//     this library with independent modules to produce an executable,
//     regardless of the license terms of these independent modules, and to
//     copy and distribute the resulting executable under terms of your choice,
//     provided that you also meet, for each linked independent module, the
//     terms and conditions of the license of that module. An independent
//     module is a module which is not derived from or based on this library.
//     If you modify this library, you must extend this exception to your
//     version of the library.

//     libzmq is distributed in the hope that it will be useful, but WITHOUT
//     ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
//     FITNESS FOR A PARTICULAR PURPOSE. See the GNU Lesser General Public
//     License for more details.

//     You should have received a copy of the GNU Lesser General Public License
//     along with this program.  If not, see <http://www.gnu.org/licenses/>.
// */

// // #include "precompiled.hpp"

// // #ifdef HAVE_LIBGSSAPI_KRB5

// // #include <string.h>
// // #include <string>

// // #include "msg.hpp"
// // #include "session_base.hpp"
// // #include "err.hpp"
// // #include "gssapi_client.hpp"
// // #include "wire.hpp"

// use std::ptr::null_mut;

// use crate::context::ZmqContext;
// use libc::{EAGAIN, EPROTO};

// use crate::curve_client_tools::produce_initiate;
// use crate::defines::ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND;
// use crate::gssapi_client::ZmqGssApiClientState::{
//     call_next_init, connected, recv_next_token, recv_ready, send_ready,
// };
// use crate::mechanism::ZmqMechanism;
// use crate::message::ZmqMessage;
// use crate::session_base::ZmqSessionBase;
// use crate::utils::copy_bytes;

// pub enum ZmqGssApiClientState {
//     call_next_init,
//     send_next_token,
//     recv_next_token,
//     send_ready,
//     recv_ready,
//     connected,
// }

// // pub struct ZmqGssApiClient  : public ZmqGssApiMechanismBase
// #[derive(Default, Debug, Clone)]
// pub struct ZmqGssApiClient<'a> {
//     pub mechanism_base: ZmqGssApiMechanismBase,
//     //  Human-readable principal name of the service we are connecting to
//     // char * service_name;
//     pub service_name: String,
//     // gss_OID service_name_type;
//     pub service_name_type: gss_OID,
//     //  Current FSM state
//     // state_t state;
//     pub state: ZmqGssApiClientState,
//     //  Points to either send_tok or recv_tok
//     //  during context initialization
//     // gss_buffer_desc * token_ptr;
//     pub token_ptr: &'a mut gss_buffer_desc,
//     //  The desired underlying mechanism
//     // gss_OID_set_desc mechs;
//     pub mechs: gss_OID_set_desc,
//     //  True iff client considers the server authenticated
//     pub security_context_established: bool,
// }

// impl ZmqGssApiClient {
//     // ZmqGssApiClient (ZmqSessionBase *session_, options: &ZmqOptions);
//     pub fn new(session_: &mut ZmqSessionBase, ctx: &ZmqContext) -> Self {
//         let mut out = Self::default();
//         // ZmqMechanismBase (session_, options_),
//         //     ZmqGssApiMechanismBase (session_, options_),
//         //     state (call_next_init),
//         //     token_ptr (GSS_C_NO_BUFFER),
//         //     mechs (),
//         //     security_context_established (false)
//         let service_size = options_.gss_service_principal.size();
//         out.service_name = String::with_capacity(service_size + 1); //(malloc (service_size + 1));
//                                                                     // assert (service_name);
//         unsafe {
//             copy_bytes(
//                 out.service_name.as_bytes_mut(),
//                 0,
//                 options_.gss_service_principal,
//                 0,
//                 service_size + 1,
//             );
//         }

//         out.service_name_type = convert_nametype(options_.gss_service_principal_nt);
//         maj_stat = GSS_S_COMPLETE;
//         if (!options_.gss_principal.empty()) {
//             principal_size = options_.gss_principal.size();
//             principal_name = String::with_capacity(principal_size + 1);
//             // assert (principal_name);
//             copy_bytes(
//                 principal_name,
//                 0,
//                 options_.gss_principal,
//                 0,
//                 principal_size + 1,
//             );
//             let name_type: gss_OID = convert_nametype(options_.gss_principal_nt);
//             if (acquire_credentials(principal_name, &cred, name_type) != 0) {
//                 maj_stat = GSS_S_FAILURE;
//             }
//         }

//         out.mechs.elements = None;
//         out.mechs.count = 0;

//         out
//     }
//     // ~ZmqGssApiClient () ;

//     // mechanism implementation

//     // int encode (msg: &mut ZmqMessage) ;
//     pub fn encode(&mut self, msg: &mut ZmqMessage) -> i32 {
//         // zmq_assert (state == Connected);

//         if (do_encryption) {
//             return encode_message(msg);
//         }

//         return 0;
//     }

//     // int decode (msg: &mut ZmqMessage) ;
//     pub fn decode(&mut self, msg: &mut ZmqMessage) -> i32 {
//         // zmq_assert (state == Connected);

//         if (do_encryption) {
//             return decode_message(msg);
//         }

//         return 0;
//     }

//     // status_t status () const ;

//     // int initialize_context ();

//     // int produce_next_token (msg: &mut ZmqMessage);

//     // int next_handshake_command (msg: &mut ZmqMessage) ;
//     pub fn next_handshake_command(&mut self, msg: &mut ZmqMessage) -> i32 {
//         if (self.state == send_ready) {
//             let rc = produce_ready(msg);
//             if (rc == 0) {
//                 state = connected;
//             }

//             return rc;
//         }

//         if (self.state != call_next_init) {
//           // errno = EAGAIN;
//             return -1;
//         }

//         if (initialize_context() < 0) {
//             return -1;
//         }

//         if (produce_next_token(msg) < 0) {
//             return -1;
//         }

//         if (maj_stat != GSS_S_CONTINUE_NEEDED && maj_stat != GSS_S_COMPLETE) {
//             return -1;
//         }

//         if (maj_stat == GSS_S_COMPLETE) {
//             security_context_established = true;
//             self.state = recv_ready;
//         } else {
//             self.state = recv_next_token;
//         }

//         return 0;
//     }

//     // int process_handshake_command (msg: &mut ZmqMessage) ;
//     pub fn process_handshake_command(&mut self, msg: &mut ZmqMessage) -> i32 {
//         if (self.state == recv_ready) {
//             let mut rc = process_ready(msg);
//             if (rc == 0) {
//                 self.state = send_ready;
//             }

//             return rc;
//         }

//         if (self.state != recv_next_token) {
//             self.session.get_socket().event_handshake_failed_protocol(
//                 self.session.get_endpoint(),
//                 ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND,
//             );
//           // errno = EPROTO;
//             return -1;
//         }

//         if (process_next_token(msg) < 0) {
//             return -1;
//         }

//         if (maj_stat != GSS_S_COMPLETE && maj_stat != GSS_S_CONTINUE_NEEDED) {
//             return -1;
//         }
//         state = call_next_init;

//         // errno_assert (msg.close () == 0);
//         // errno_assert (msg.init () == 0);

//         return 0;
//     }

//     pub fn status(&mut self) -> ZmqGssApiClientStatus {
//         return if self.state == connected {
//             ZmqMechanism::ready
//         } else {
//             ZmqMechanism::handshaking
//         };
//     }

//     pub fn initialize_context(&mut self) -> i32 {
//         // principal was specified but credentials could not be acquired
//         if (principal_name != null_mut() && cred == null_mut()) {
//             return -1;
//         }

//         // First time through, import service_name into target_name
//         if (target_name == GSS_C_NO_NAME) {
//             send_tok.value = service_name;
//             send_tok.length = (service_name.len()) + 1;
//             let maj = gss_import_name(&min_stat, &send_tok, service_name_type, &target_name);

//             if (maj != GSS_S_COMPLETE) {
//                 return -1;
//             }
//         }

//         maj_stat = gss_init_sec_context(
//             &init_sec_min_stat,
//             cred,
//             &context,
//             target_name,
//             mechs.elements,
//             gss_flags,
//             0,
//             null_mut(),
//             token_ptr,
//             null_mut(),
//             &send_tok,
//             &ret_flags,
//             null_mut(),
//         );

//         if (token_ptr != GSS_C_NO_BUFFER) {
//             // free(recv_tok.value);
//         }

//         return 0;
//     }

//     pub fn produce_next_token(&mut self, msg: &mut ZmqMessage) -> i32 {
//         if (send_tok.length != 0) {
//             // Server expects another token
//             if self.produce_initiate(msg, send_tok.value, send_tok.length) < 0 {
//                 gss_release_buffer(&min_stat, &send_tok);
//                 gss_release_name(&min_stat, &target_name);
//                 return -1;
//             }
//         }
//         gss_release_buffer(&min_stat, &send_tok);

//         if (maj_stat != GSS_S_COMPLETE && maj_stat != GSS_S_CONTINUE_NEEDED) {
//             gss_release_name(&min_stat, &target_name);
//             if (context != GSS_C_NO_CONTEXT) {
//                 gss_delete_sec_context(&min_stat, &context, GSS_C_NO_BUFFER);
//             }
//             return -1;
//         }

//         return 0;
//     }

//     pub fn process_next_token(&mut self, msg: &mut ZmqMessage) -> i32 {
//         if (maj_stat == GSS_S_CONTINUE_NEEDED) {
//             if (process_initiate(msg, &recv_tok.value, recv_tok.length) < 0) {
//                 gss_release_name(&min_stat, &target_name);
//                 return -1;
//             }
//             token_ptr = &recv_tok;
//         }

//         return 0;
//     }
// }

// // ZmqGssApiClient::~ZmqGssApiClient ()
// // {
// //     if (service_name)
// //         free (service_name);
// //     if (cred)
// //         gss_release_cred (&min_stat, &cred);
// // }

// // #endif
