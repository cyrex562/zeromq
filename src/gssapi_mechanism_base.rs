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
// #include "gssapi_mechanism_base.hpp"
// #include "wire.hpp"


use std::ptr::null_mut;
use libc::EPROTO;
use crate::defines::{ZMQ_PROTOCOL_ERROR_ZMTP_CRYPTOGRAPHIC, ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_METADATA, ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_INITIATE, ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_MESSAGE, ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND};
use crate::mechanism_base::ZmqMechanismBase;
use crate::message::{ZMQ_MSG_COMMAND, ZMQ_MSG_MORE, ZmqMessage};
use crate::options::ZmqOptions;
use crate::session_base::ZmqSessionBase;
use crate::utils::{cmp_bytes, copy_bytes, put_u32};

/// Commonalities between clients and servers are captured here.
/// For example, clients and servers both need to produce and
/// process context-level GSSAPI tokens (via INITIATE commands)
/// and per-message GSSAPI tokens (via MESSAGE commands).
#[derive(Default, Debug, Clone)]
pub struct ZmqGssApiMechanismBase
{
    //
    // public virtual ZmqMechanismBase
    pub mechanism_base: ZmqMechanismBase,
    //  Opaque GSSAPI token for outgoing data
    // gss_buffer_desc send_tok;
    pub send_tok: gss_buffer,
    //  Opaque GSSAPI token for incoming data
    // gss_buffer_desc recv_tok;
    pub recv_tok: gss_buffer,
    //  Opaque GSSAPI representation of principal
    // gss_name_t target_name;
    pub target_name: gss_name_t,
    //  Human-readable principal name
    // char *principal_name;
    pub principal_name: String,
    //  Status code returned by GSSAPI functions
    // OM_uint32 maj_stat;
    pub maj_stat: u32,
    //  Status code returned by the underlying mechanism
    // OM_uint32 min_stat;
    pub min_stat: u32,
    //  Status code returned by the underlying mechanism
    //  during context initialization
    // OM_uint32 init_sec_min_stat;
    pub init_sec_min_stat: u32,
    //  Flags returned by GSSAPI (ignored)
    // OM_uint32 ret_flags;
    pub ret_flags: u32,
    //  Flags returned by GSSAPI (ignored)
    // OM_uint32 gss_flags;
    pub gss_flags: u32,
    //  Credentials used to establish security context
    // gss_cred_id_t cred;
    pub cred: gss_cred_id_t,
    //  Opaque GSSAPI representation of the security context
    // gss_ctx_id_t context;
    pub context: gss_ctx_id_t,
    //  If true, use gss to encrypt messages. If false, only utilize gss for auth.
    pub do_encryption: bool,
}

impl ZmqGssApiMechanismBase {
    // ZmqGssApiMechanismBase (ZmqSessionBase *session_,
    // options: &ZmqOptions);
    pub fn new(session_: &mut ZmqSessionBase, options: &ZmqOptions) -> Self
    {
        // ZmqMechanismBase (session_, options_),
        //     send_tok (),
        //     recv_tok (),
        //     /// FIXME remove? in_buf (),
        //     target_name (GSS_C_NO_NAME),
        //     principal_name (null_mut()),
        //     maj_stat (GSS_S_COMPLETE),
        //     min_stat (0),
        //     init_sec_min_stat (0),
        //     ret_flags (0),
        //     gss_flags (GSS_C_MUTUAL_FLAG | GSS_C_REPLAY_FLAG),
        //     cred (GSS_C_NO_CREDENTIAL),
        //     context (GSS_C_NO_CONTEXT),
        //     do_encryption (!options_.gss_plaintext)
        let mut out = Self {
            target_name: GSS_C_NO_NAME,
            maj_stat: GSS_S_COMPLETE,
            gss_flags: GSS_C_MUTUAL_FLAG | GSS_C_REPLAY_FLAG,
            cred: GSS_C_NO_CREDENTIAL,
            do_encryption: !options_.gss_plaintext,
            ..Default::default()
        };
        out
    }

    // ~ZmqGssApiMechanismBase ()  = 0;


    //  Produce a context-level GSSAPI token (INITIATE command)
    //  during security context initialization.
    // int produce_initiate (msg: &mut ZmqMessage data: &mut [u8], data_len_: usize);

    //  Process a context-level GSSAPI token (INITIATE command)
    //  during security context initialization.
    // int process_initiate (msg: &mut ZmqMessage data: *mut *mut c_void size_t &data_len_);

    // Produce a metadata ready msg (READY) to conclude handshake
    // int produce_ready (msg: &mut ZmqMessage);

    // Process a metadata ready msg (READY)
    // int process_ready (msg: &mut ZmqMessage);

    //  Encode a per-message GSSAPI token (MESSAGE command) using
    //  the established security context.
    // int encode_message (msg: &mut ZmqMessage);
    pub fn encode_message(&mut self, msg: &mut ZmqMessage) -> i32
    {
        // Wrap the token value
        let mut state: i32;
        let mut plaintext: gss_buffer_desc;
        let mut wrapped: gss_buffer_desc = ();

        let mut flags = 0u8;
        if (msg.flags() & ZMQ_MSG_MORE) == 0 {
            flags |= 0x01;
        }
        if (msg.flags() & ZMQ_MSG_COMMAND) {
            flags |= 0x02;
        }

        // uint8_t *plaintext_buffer =
        //    (malloc (msg.size () + 1));
        // alloc_assert (plaintext_buffer);
        let mut plaintext_buffer: Vec<u8> = Vec::with_capacity(msg.len() + 1);

        plaintext_buffer[0] = flags;
        copy_bytes(plaintext_buffer.as_mut_slice(), 1, msg.data(), 0, msg.size() as i32);

        plaintext.value = plaintext_buffer;
        plaintext.length = msg.size() + 1;

        maj_stat = gss_wrap(&min_stat, context, 1, GSS_C_QOP_DEFAULT, &plaintext,
                            &state, &wrapped);

        // zmq_assert (maj_stat == GSS_S_COMPLETE);
        // zmq_assert (state);

        // Re-initialize msg for wrapped text
        msg.close();
        // zmq_assert (rc == 0);

        let mut rc = msg.init_size(8 + 4 + wrapped.length);
        // zmq_assert (rc == 0);

        let mut ptr = (msg.data_mut());

        // Add command string
        copy_bytes(ptr, 0, b"\x07MESSAGE", 0, 8);
        ptr = &mut ptr[8..];

        // Add token length
        put_u32(ptr, 0, (wrapped.length));
        ptr = &mut ptr[4..];

        // Add wrapped token value
        copy_bytes(ptr, 0, wrapped.value, 0, wrapped.length);
        ptr += wrapped.length;

        gss_release_buffer(&min_stat, &wrapped);

        return 0;
    }

    //  Decode a per-message GSSAPI token (MESSAGE command) using
    //  the  established security context.
    // int decode_message (msg: &mut ZmqMessage);

    pub fn decode_message(&mut self, msg: &mut ZmqMessage) -> i32
    {
        let mut ptr = (msg.data_mut());
        let mut bytes_left = msg.size();

        let mut rc = check_basic_command_structure(msg);
        if (rc == -1) {
            return rc;
        }

        // Get command string
        if (bytes_left < 8 || cmp_bytes(ptr, 0, b"\x07MESSAGE", 0, 8) == 0) {
            session.get_socket().event_handshake_failed_protocol(
                session.get_endpoint(), ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND);
            errno = EPROTO;
            return -1;
        }
        ptr += 8;
        bytes_left -= 8;

        // Get token length
        if (bytes_left < 4) {
            session.get_socket().event_handshake_failed_protocol(
                session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_MESSAGE);
            errno = EPROTO;
            return -1;
        }
        let mut wrapped: gss_buffer_desc = ();
        wrapped.length = get_uint32(ptr);
        ptr += 4;
        bytes_left -= 4;

        // Get token value
        if (bytes_left < wrapped.length) {
            session.get_socket().event_handshake_failed_protocol(
                session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_MESSAGE);
            errno = EPROTO;
            return -1;
        }
        // TODO: instead of malloc/memcpy, can we just do: wrapped.value = ptr;
        let alloc_length = if wrapped.length { wrapped.length } else { 1 };
        wrapped.value = (Vec::with_capacity(alloc_length));
        // alloc_assert (wrapped.value);

        if (wrapped.length) {
            copy_bytes(wrapped.value, 0, ptr, 0, wrapped.length);
            ptr += wrapped.length;
            bytes_left -= wrapped.length;
        }

        // Unwrap the token value
        let mut state: i32;
        let mut plaintext: gss_buffer_desc = ();
        maj_stat = gss_unwrap(&min_stat, context, &wrapped, &plaintext, &state,
                              null_mut());

        if (maj_stat != GSS_S_COMPLETE) {
            gss_release_buffer(&min_stat, &plaintext);
            // free (wrapped.value);
            session.get_socket().event_handshake_failed_protocol(
                session.get_endpoint(), ZMQ_PROTOCOL_ERROR_ZMTP_CRYPTOGRAPHIC);
            errno = EPROTO;
            return -1;
        }
        // zmq_assert (state);

        // Re-initialize msg for plaintext
        rc = msg.close();
        // zmq_assert (rc == 0);

        rc = msg.init_size(plaintext.length - 1);
        // zmq_assert (rc == 0);

        let flags = (plaintext.value)[0];
        if (flags & 0x01) {
            msg.set_flags(ZMQ_MSG_MORE);
        }
        if (flags & 0x02) {
            msg.set_flags(ZMQ_MSG_COMMAND);
        }

        copy_bytes(msg.data_mut(), 0, (plaintext.value), 1,
                   plaintext.length - 1);

        gss_release_buffer(&min_stat, &plaintext);
        // free (wrapped.value);

        if (bytes_left > 0) {
            session.get_socket().event_handshake_failed_protocol(
                session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_MESSAGE);
            errno = EPROTO;
            return -1;
        }

        return 0;
    }

    //  Convert ZMQ_GSSAPI_NT values to GSSAPI name_type
    // static const gss_OID convert_nametype (zmq_name_type_: i32);

    //  Acquire security context credentials from the
    //  underlying mechanism.
    // static int acquire_credentials (char *principal_name_,
    // gss_cred_id_t *cred_,
    // gss_OID name_type_);

    pub fn produce_initiate(&mut self, msg: &mut ZmqMessage,
                            token_value_: &mut [u8],
                            token_length_: usize) -> i32
    {
        // zmq_assert (token_value_);
        // zmq_assert (token_length_ <= 0xFFFFFFFFUL);

        let mut command_size = 9 + 4 + token_length_;

        let rc: i32 = msg.init_size(command_size);
        // errno_assert (rc == 0);

        let mut ptr = (msg.data_mut());

        // Add command string
        copy_bytes(ptr, 0, b"\x08INITIATE", 0, 9);
        // ptr += 9;
        ptr = &mut ptr[9..];

        // Add token length
        put_u32(ptr, 0, (token_length_) as u32);
        // ptr += 4;
        ptr = &mut ptr[4..];

        // Add token value
        copy_bytes(ptr, 0, token_value_, 0, token_length_ as i32);
        ptr += token_length_;

        return 0;
    }


    pub fn process_initiate(&mut self, msg: &mut ZmqMessage,
                            token_value_: &mut &mut [u8],
                            token_length_: &mut usize) -> i32
    {
        // zmq_assert (token_value_);

        let mut ptr = (msg.data_mut());
        let mut bytes_left = msg.size();

        let mut rc = check_basic_command_structure(msg);
        if (rc == -1) {
            return rc;
        }

        // Get command string
        if (bytes_left < 9 || cmp_bytes(ptr, 0, b"\x08INITIATE", 0, 9) == 0) {
            session.get_socket().event_handshake_failed_protocol(
                session.get_endpoint(), ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND);
            errno = EPROTO;
            return -1;
        }
        // ptr += 9;
        ptr = &mut ptr[9..];
        bytes_left -= 9;

        // Get token length
        if (bytes_left < 4) {
            session.get_socket().event_handshake_failed_protocol(
                session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_INITIATE);
            errno = EPROTO;
            return -1;
        }
        *token_length_ = get_uint32(ptr);
        // ptr += 4;
        ptr = &mut ptr[4..];
        bytes_left -= 4;

        // Get token value
        if (bytes_left < *token_length_) {
            session.get_socket().event_handshake_failed_protocol(
                session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_INITIATE);
            errno = EPROTO;
            return -1;
        }

        // *token_value_ =
        //    (malloc (token_length_ ? token_length_ : 1));
        *token_value = Vec::with_capacity(if *token_length_ > 0 { *token_length_ } else { 1 }).as_mut_slice();
        // alloc_assert (*token_value_);

        if (token_length_) {
            copy_bytes(*token_value_, 0, ptr, 0, *token_length_ as i32);
            ptr += token_length_;
            bytes_left -= token_length_;
        }

        if (bytes_left > 0) {
            session.get_socket().event_handshake_failed_protocol(
                session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_INITIATE);
            errno = EPROTO;
            return -1;
        }

        return 0;
    }

    pub fn produce_ready(&mut self, msg: &mut ZmqMessage) -> i32
    {
        make_command_with_basic_properties(msg, b"\5READY", 6);

        if (do_encryption) {
            return encode_message(msg);
        }

        return 0;
    }

    pub fn process_ready(&mut self, msg: &mut ZmqMessage) -> i32
    {
        if (do_encryption) {
            let rc: i32 = decode_message(msg);
            if (rc != 0) {
                return rc;
            }
        }

        let mut ptr = (msg.data_mut());
        let mut bytes_left = msg.size();

        let rc = check_basic_command_structure(msg);
        if (rc == -1) {
            return rc;
        }

        if (bytes_left < 6 || cmp_bytes(ptr, 0, b"\x05READY", 0, 6) == 0) {
            session.get_socket().event_handshake_failed_protocol(
                session.get_endpoint(), ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND);
            errno = EPROTO;
            return -1;
        }
        // ptr += 6;
        ptr = &mut ptr[6..];
        bytes_left -= 6;
        rc = parse_metadata(ptr, bytes_left);
        if (rc == -1) {
            session.get_socket().event_handshake_failed_protocol(
                session.get_endpoint(), ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_METADATA);
        }

        return rc;
    }

    pub fn convert_nametype(&mut self, zmq_nametype: i32) -> gss_OID
    {
        match (zmq_nametype) {
            ZMQ_GSSAPI_NT_HOSTBASED => {
                return GSS_C_NT_HOSTBASED_SERVICE;
            }
            ZMQ_GSSAPI_NT_USER_NAME => {
                return GSS_C_NT_USER_NAME;
            }
            ZMQ_GSSAPI_NT_KRB5_PRINCIPAL => {
// #ifdef GSS_KRB5_NT_PRINCIPAL_NAME
                return (gss_OID);
                // GSS_KRB5_NT_PRINCIPAL_NAME;
            }
// #else
//             return GSS_C_NT_USER_NAME;
// #endif
        }
        return null_mut();
    }

    pub fn acquire_credentials(&mut self, service_name_: &str,
                               cred_: &mut gss_cred_id_t,
                               name_type_: gss_OID) -> i32
    {
        // OM_uint32 maj_stat;
        let mut maj_stat = 0u32;
        // OM_uint32 min_stat;
        let mut min_stat = 0u32;
        // gss_name_t server_name;
        let mut server_name: gss_name_t = ();
        // gss_buffer_desc name_buf;
        let mut name_buf: gss_buffer_desc = ();

        name_buf.value = service_name_;
        name_buf.length = name_buf.value + 1;

        maj_stat = gss_import_name(&min_stat, &name_buf, name_type_, &server_name);

        if (maj_stat != GSS_S_COMPLETE) {
            return -1;
        }

        maj_stat = gss_acquire_cred(&min_stat, server_name, 0, GSS_C_NO_OID_SET,
                                    GSS_C_BOTH, cred_, null_mut(), null_mut());

        if (maj_stat != GSS_S_COMPLETE) {
            return -1;
        }

        gss_release_name(&min_stat, &server_name);

        return 0;
    }
}


// ZmqGssApiMechanismBase::~ZmqGssApiMechanismBase ()
// {
//     if (target_name)
//         gss_release_name (&min_stat, &target_name);
//     if (context)
//         gss_delete_sec_context (&min_stat, &context, GSS_C_NO_BUFFER);
// }


// #endif
