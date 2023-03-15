/*
    Copyright (c) 2007-2016 Contributors as noted in the AUTHORS file

    This file is part of libzmq, the ZeroMQ core engine in C++.

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


/// Commonalities between clients and servers are captured here.
/// For example, clients and servers both need to produce and
/// process context-level GSSAPI tokens (via INITIATE commands)
/// and per-message GSSAPI tokens (via MESSAGE commands).
pub struct gssapi_ZmqMechanismBase : public virtual ZmqMechanismBase
{
// public:
    gssapi_ZmqMechanismBase (ZmqSessionBase *session_,
                             const ZmqOptions &options_);
    ~gssapi_ZmqMechanismBase () ZMQ_OVERRIDE = 0;

  protected:
    //  Produce a context-level GSSAPI token (INITIATE command)
    //  during security context initialization.
    int produce_initiate (msg: &mut ZmqMessage data: &mut [u8], data_len_: usize);

    //  Process a context-level GSSAPI token (INITIATE command)
    //  during security context initialization.
    int process_initiate (msg: &mut ZmqMessage data: *mut *mut c_void size_t &data_len_);

    // Produce a metadata ready msg (READY) to conclude handshake
    int produce_ready (msg: &mut ZmqMessage);

    // Process a metadata ready msg (READY)
    int process_ready (msg: &mut ZmqMessage);

    //  Encode a per-message GSSAPI token (MESSAGE command) using
    //  the established security context.
    int encode_message (msg: &mut ZmqMessage);

    //  Decode a per-message GSSAPI token (MESSAGE command) using
    //  the  established security context.
    int decode_message (msg: &mut ZmqMessage);

    //  Convert ZMQ_GSSAPI_NT values to GSSAPI name_type
    static const gss_OID convert_nametype (zmq_name_type_: i32);

    //  Acquire security context credentials from the
    //  underlying mechanism.
    static int acquire_credentials (char *principal_name_,
                                    gss_cred_id_t *cred_,
                                    gss_OID name_type_);

  protected:
    //  Opaque GSSAPI token for outgoing data
    gss_buffer_desc send_tok;

    //  Opaque GSSAPI token for incoming data
    gss_buffer_desc recv_tok;

    //  Opaque GSSAPI representation of principal
    gss_name_t target_name;

    //  Human-readable principal name
    char *principal_name;

    //  Status code returned by GSSAPI functions
    OM_uint32 maj_stat;

    //  Status code returned by the underlying mechanism
    OM_uint32 min_stat;

    //  Status code returned by the underlying mechanism
    //  during context initialization
    OM_uint32 init_sec_min_stat;

    //  Flags returned by GSSAPI (ignored)
    OM_uint32 ret_flags;

    //  Flags returned by GSSAPI (ignored)
    OM_uint32 gss_flags;

    //  Credentials used to establish security context
    gss_cred_id_t cred;

    //  Opaque GSSAPI representation of the security context
    gss_ctx_id_t context;

    //  If true, use gss to encrypt messages. If false, only utilize gss for auth.
    do_encryption: bool
};

gssapi_ZmqMechanismBase::gssapi_ZmqMechanismBase (
  ZmqSessionBase *session_, const ZmqOptions &options_) :
    ZmqMechanismBase (session_, options_),
    send_tok (),
    recv_tok (),
    /// FIXME remove? in_buf (),
    target_name (GSS_C_NO_NAME),
    principal_name (null_mut()),
    maj_stat (GSS_S_COMPLETE),
    min_stat (0),
    init_sec_min_stat (0),
    ret_flags (0),
    gss_flags (GSS_C_MUTUAL_FLAG | GSS_C_REPLAY_FLAG),
    cred (GSS_C_NO_CREDENTIAL),
    context (GSS_C_NO_CONTEXT),
    do_encryption (!options_.gss_plaintext)
{
}

gssapi_ZmqMechanismBase::~gssapi_ZmqMechanismBase ()
{
    if (target_name)
        gss_release_name (&min_stat, &target_name);
    if (context)
        gss_delete_sec_context (&min_stat, &context, GSS_C_NO_BUFFER);
}

int gssapi_ZmqMechanismBase::encode_message (msg: &mut ZmqMessage)
{
    // Wrap the token value
    state: i32;
    gss_buffer_desc plaintext;
    gss_buffer_desc wrapped;

    uint8_t flags = 0;
    if (msg.flags () & ZMQ_MSG_MORE)
        flags |= 0x01;
    if (msg.flags () & ZMQ_MSG_COMMAND)
        flags |= 0x02;

    uint8_t *plaintext_buffer =
      static_cast<uint8_t *> (malloc (msg.size () + 1));
    alloc_assert (plaintext_buffer);

    plaintext_buffer[0] = flags;
    memcpy (plaintext_buffer + 1, msg.data (), msg.size ());

    plaintext.value = plaintext_buffer;
    plaintext.length = msg.size () + 1;

    maj_stat = gss_wrap (&min_stat, context, 1, GSS_C_QOP_DEFAULT, &plaintext,
                         &state, &wrapped);

    zmq_assert (maj_stat == GSS_S_COMPLETE);
    zmq_assert (state);

    // Re-initialize msg for wrapped text
    int rc = msg.close ();
    zmq_assert (rc == 0);

    rc = msg.init_size (8 + 4 + wrapped.length);
    zmq_assert (rc == 0);

    uint8_t *ptr = static_cast<uint8_t *> (msg.data ());

    // Add command string
    memcpy (ptr, "\x07MESSAGE", 8);
    ptr += 8;

    // Add token length
    put_u32 (ptr, static_cast<u32> (wrapped.length));
    ptr += 4;

    // Add wrapped token value
    memcpy (ptr, wrapped.value, wrapped.length);
    ptr += wrapped.length;

    gss_release_buffer (&min_stat, &wrapped);

    return 0;
}

int gssapi_ZmqMechanismBase::decode_message (msg: &mut ZmqMessage)
{
    const uint8_t *ptr = static_cast<uint8_t *> (msg.data ());
    size_t bytes_left = msg.size ();

    int rc = check_basic_command_structure (msg);
    if (rc == -1)
        return rc;

    // Get command string
    if (bytes_left < 8 || memcmp (ptr, "\x07MESSAGE", 8)) {
        session.get_socket ()->event_handshake_failed_protocol (
          session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND);
        errno = EPROTO;
        return -1;
    }
    ptr += 8;
    bytes_left -= 8;

    // Get token length
    if (bytes_left < 4) {
        session.get_socket ()->event_handshake_failed_protocol (
          session.get_endpoint (),
          ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_MESSAGE);
        errno = EPROTO;
        return -1;
    }
    gss_buffer_desc wrapped;
    wrapped.length = get_uint32 (ptr);
    ptr += 4;
    bytes_left -= 4;

    // Get token value
    if (bytes_left < wrapped.length) {
        session.get_socket ()->event_handshake_failed_protocol (
          session.get_endpoint (),
          ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_MESSAGE);
        errno = EPROTO;
        return -1;
    }
    // TODO: instead of malloc/memcpy, can we just do: wrapped.value = ptr;
    const size_t alloc_length = wrapped.length ? wrapped.length : 1;
    wrapped.value = static_cast<char *> (malloc (alloc_length));
    alloc_assert (wrapped.value);

    if (wrapped.length) {
        memcpy (wrapped.value, ptr, wrapped.length);
        ptr += wrapped.length;
        bytes_left -= wrapped.length;
    }

    // Unwrap the token value
    state: i32;
    gss_buffer_desc plaintext;
    maj_stat = gss_unwrap (&min_stat, context, &wrapped, &plaintext, &state,
                           (gss_qop_t *) null_mut());

    if (maj_stat != GSS_S_COMPLETE) {
        gss_release_buffer (&min_stat, &plaintext);
        free (wrapped.value);
        session.get_socket ()->event_handshake_failed_protocol (
          session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_CRYPTOGRAPHIC);
        errno = EPROTO;
        return -1;
    }
    zmq_assert (state);

    // Re-initialize msg for plaintext
    rc = msg.close ();
    zmq_assert (rc == 0);

    rc = msg.init_size (plaintext.length - 1);
    zmq_assert (rc == 0);

    const uint8_t flags = static_cast<char *> (plaintext.value)[0];
    if (flags & 0x01)
        msg.set_flags (ZMQ_MSG_MORE);
    if (flags & 0x02)
        msg.set_flags (ZMQ_MSG_COMMAND);

    memcpy (msg.data (), static_cast<char *> (plaintext.value) + 1,
            plaintext.length - 1);

    gss_release_buffer (&min_stat, &plaintext);
    free (wrapped.value);

    if (bytes_left > 0) {
        session.get_socket ()->event_handshake_failed_protocol (
          session.get_endpoint (),
          ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_MESSAGE);
        errno = EPROTO;
        return -1;
    }

    return 0;
}

int gssapi_ZmqMechanismBase::produce_initiate (msg: &mut ZmqMessage
                                                    token_value_: &mut [u8],
                                                    token_length_: usize)
{
    zmq_assert (token_value_);
    zmq_assert (token_length_ <= 0xFFFFFFFFUL);

    const size_t command_size = 9 + 4 + token_length_;

    let rc: i32 = msg.init_size (command_size);
    errno_assert (rc == 0);

    uint8_t *ptr = static_cast<uint8_t *> (msg.data ());

    // Add command string
    memcpy (ptr, "\x08INITIATE", 9);
    ptr += 9;

    // Add token length
    put_u32 (ptr, static_cast<u32> (token_length_));
    ptr += 4;

    // Add token value
    memcpy (ptr, token_value_, token_length_);
    ptr += token_length_;

    return 0;
}

int gssapi_ZmqMechanismBase::process_initiate (msg: &mut ZmqMessage
                                                    token_value_: *mut *mut c_void
                                                    size_t &token_length_)
{
    zmq_assert (token_value_);

    const uint8_t *ptr = static_cast<uint8_t *> (msg.data ());
    size_t bytes_left = msg.size ();

    int rc = check_basic_command_structure (msg);
    if (rc == -1)
        return rc;

    // Get command string
    if (bytes_left < 9 || memcmp (ptr, "\x08INITIATE", 9)) {
        session.get_socket ()->event_handshake_failed_protocol (
          session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND);
        errno = EPROTO;
        return -1;
    }
    ptr += 9;
    bytes_left -= 9;

    // Get token length
    if (bytes_left < 4) {
        session.get_socket ()->event_handshake_failed_protocol (
          session.get_endpoint (),
          ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_INITIATE);
        errno = EPROTO;
        return -1;
    }
    token_length_ = get_uint32 (ptr);
    ptr += 4;
    bytes_left -= 4;

    // Get token value
    if (bytes_left < token_length_) {
        session.get_socket ()->event_handshake_failed_protocol (
          session.get_endpoint (),
          ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_INITIATE);
        errno = EPROTO;
        return -1;
    }

    *token_value_ =
      static_cast<char *> (malloc (token_length_ ? token_length_ : 1));
    alloc_assert (*token_value_);

    if (token_length_) {
        memcpy (*token_value_, ptr, token_length_);
        ptr += token_length_;
        bytes_left -= token_length_;
    }

    if (bytes_left > 0) {
        session.get_socket ()->event_handshake_failed_protocol (
          session.get_endpoint (),
          ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_INITIATE);
        errno = EPROTO;
        return -1;
    }

    return 0;
}

int gssapi_ZmqMechanismBase::produce_ready (msg: &mut ZmqMessage)
{
    make_command_with_basic_properties (msg, "\5READY", 6);

    if (do_encryption)
        return encode_message (msg);

    return 0;
}

int gssapi_ZmqMechanismBase::process_ready (msg: &mut ZmqMessage)
{
    if (do_encryption) {
        let rc: i32 = decode_message (msg);
        if (rc != 0)
            return rc;
    }

    const unsigned char *ptr = static_cast<unsigned char *> (msg.data ());
    size_t bytes_left = msg.size ();

    int rc = check_basic_command_structure (msg);
    if (rc == -1)
        return rc;

    if (bytes_left < 6 || memcmp (ptr, "\x05READY", 6)) {
        session.get_socket ()->event_handshake_failed_protocol (
          session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND);
        errno = EPROTO;
        return -1;
    }
    ptr += 6;
    bytes_left -= 6;
    rc = parse_metadata (ptr, bytes_left);
    if (rc == -1)
        session.get_socket ()->event_handshake_failed_protocol (
          session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_METADATA);

    return rc;
}

const gss_OID gssapi_ZmqMechanismBase::convert_nametype (zmq_nametype: i32)
{
    switch (zmq_nametype) {
        case ZMQ_GSSAPI_NT_HOSTBASED:
            return GSS_C_NT_HOSTBASED_SERVICE;
        case ZMQ_GSSAPI_NT_USER_NAME:
            return GSS_C_NT_USER_NAME;
        case ZMQ_GSSAPI_NT_KRB5_PRINCIPAL:
// #ifdef GSS_KRB5_NT_PRINCIPAL_NAME
            return (gss_OID) GSS_KRB5_NT_PRINCIPAL_NAME;
// #else
            return GSS_C_NT_USER_NAME;
// #endif
    }
    return null_mut();
}

int gssapi_ZmqMechanismBase::acquire_credentials (char *service_name_,
                                                       gss_cred_id_t *cred_,
                                                       gss_OID name_type_)
{
    OM_uint32 maj_stat;
    OM_uint32 min_stat;
    gss_name_t server_name;

    gss_buffer_desc name_buf;
    name_buf.value = service_name_;
    name_buf.length = strlen ((char *) name_buf.value) + 1;

    maj_stat = gss_import_name (&min_stat, &name_buf, name_type_, &server_name);

    if (maj_stat != GSS_S_COMPLETE)
        return -1;

    maj_stat = gss_acquire_cred (&min_stat, server_name, 0, GSS_C_NO_OID_SET,
                                 GSS_C_BOTH, cred_, null_mut(), null_mut());

    if (maj_stat != GSS_S_COMPLETE)
        return -1;

    gss_release_name (&min_stat, &server_name);

    return 0;
}

// #endif
