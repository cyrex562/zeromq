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
// #include "gssapi_server.hpp"
// #include "wire.hpp"

// #include <gssapi/gssapi.h>
pub struct gssapi_server_t ZMQ_FINAL : public gssapi_ZmqMechanismBase,
                                  public zap_client_t
{
// public:
    gssapi_server_t (ZmqSessionBase *session_,
                     const std::string &peer_address,
                     options: &ZmqOptions);
    ~gssapi_server_t () ZMQ_FINAL;

    // mechanism implementation
    int next_handshake_command (msg: &mut ZmqMessage) ZMQ_FINAL;
    int process_handshake_command (msg: &mut ZmqMessage) ZMQ_FINAL;
    int encode (msg: &mut ZmqMessage) ZMQ_FINAL;
    int decode (msg: &mut ZmqMessage) ZMQ_FINAL;
    int zap_msg_available () ZMQ_FINAL;
    status_t status () const ZMQ_FINAL;

  // private:
    enum state_t
    {
        send_next_token,
        recv_next_token,
        expect_zap_reply,
        send_ready,
        recv_ready,
        connected
    };

    ZmqSessionBase *const session;

    const peer_address: String;

    //  Current FSM state
    state_t state;

    //  True iff server considers the client authenticated
    security_context_established: bool

    //  The underlying mechanism type (ignored)
    gss_OID doid;

    void accept_context ();
    int produce_next_token (msg: &mut ZmqMessage);
    int process_next_token (msg: &mut ZmqMessage);
    void send_zap_request ();
};

gssapi_server_t::gssapi_server_t (ZmqSessionBase *session_,
                                       const std::string &peer_address_,
                                       options: &ZmqOptions) :
    ZmqMechanismBase (session_, options_),
    gssapi_ZmqMechanismBase (session_, options_),
    zap_client_t (session_, peer_address_, options_),
    session (session_),
    peer_address (peer_address_),
    state (recv_next_token),
    security_context_established (false)
{
    maj_stat = GSS_S_CONTINUE_NEEDED;
    if (!options_.gss_principal.empty ()) {
        const std::string::size_type principal_size =
          options_.gss_principal.size ();
        principal_name = static_cast<char *> (malloc (principal_size + 1));
        assert (principal_name);
        memcpy (principal_name, options_.gss_principal,
                principal_size + 1);
        gss_OID name_type = convert_nametype (options_.gss_principal_nt);
        if (acquire_credentials (principal_name, &cred, name_type) != 0)
            maj_stat = GSS_S_FAILURE;
    }
}

gssapi_server_t::~gssapi_server_t ()
{
    if (cred)
        gss_release_cred (&min_stat, &cred);

    if (target_name)
        gss_release_name (&min_stat, &target_name);
}

int gssapi_server_t::next_handshake_command (msg: &mut ZmqMessage)
{
    if (state == send_ready) {
        int rc = produce_ready (msg);
        if (rc == 0)
            state = recv_ready;

        return rc;
    }

    if (state != send_next_token) {
        errno = EAGAIN;
        return -1;
    }

    if (produce_next_token (msg) < 0)
        return -1;

    if (maj_stat != GSS_S_CONTINUE_NEEDED && maj_stat != GSS_S_COMPLETE)
        return -1;

    if (maj_stat == GSS_S_COMPLETE) {
        security_context_established = true;
    }

    state = recv_next_token;

    return 0;
}

int gssapi_server_t::process_handshake_command (msg: &mut ZmqMessage)
{
    if (state == recv_ready) {
        int rc = process_ready (msg);
        if (rc == 0)
            state = connected;

        return rc;
    }

    if (state != recv_next_token) {
        session.get_socket ().event_handshake_failed_protocol (
          session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND);
        errno = EPROTO;
        return -1;
    }

    if (security_context_established) {
        //  Use ZAP protocol (RFC 27) to authenticate the user.
        //  Note that rc will be -1 only if ZAP is not set up, but if it was
        //  requested and it does not work properly the program will abort.
        bool expecting_zap_reply = false;
        int rc = session.zap_connect ();
        if (rc == 0) {
            send_zap_request ();
            rc = receive_and_process_zap_reply ();
            if (rc != 0) {
                if (rc == -1)
                    return -1;
                expecting_zap_reply = true;
            }
        }
        state = expecting_zap_reply ? expect_zap_reply : send_ready;
        return 0;
    }

    if (process_next_token (msg) < 0)
        return -1;

    accept_context ();
    state = send_next_token;

    errno_assert (msg.close () == 0);
    errno_assert (msg.init () == 0);

    return 0;
}

void gssapi_server_t::send_zap_request ()
{
    gss_buffer_desc principal;
    gss_display_name (&min_stat, target_name, &principal, null_mut());
    zap_client_t::send_zap_request (
      "GSSAPI", 6, reinterpret_cast<const uint8_t *> (principal.value),
      principal.length);

    gss_release_buffer (&min_stat, &principal);
}

int gssapi_server_t::encode (msg: &mut ZmqMessage)
{
    zmq_assert (state == connected);

    if (do_encryption)
        return encode_message (msg);

    return 0;
}

int gssapi_server_t::decode (msg: &mut ZmqMessage)
{
    zmq_assert (state == connected);

    if (do_encryption)
        return decode_message (msg);

    return 0;
}

int gssapi_server_t::zap_msg_available ()
{
    if (state != expect_zap_reply) {
        errno = EFSM;
        return -1;
    }
    let rc: i32 = receive_and_process_zap_reply ();
    if (rc == 0)
        state = send_ready;
    return rc == -1 ? -1 : 0;
}

ZmqMechanism::status_t gssapi_server_t::status () const
{
    return state == connected ? ZmqMechanism::ready : ZmqMechanism::handshaking;
}

int gssapi_server_t::produce_next_token (msg: &mut ZmqMessage)
{
    if (send_tok.length != 0) { // Client expects another token
        if (produce_initiate (msg, send_tok.value, send_tok.length) < 0)
            return -1;
        gss_release_buffer (&min_stat, &send_tok);
    }

    if (maj_stat != GSS_S_COMPLETE && maj_stat != GSS_S_CONTINUE_NEEDED) {
        gss_release_name (&min_stat, &target_name);
        if (context != GSS_C_NO_CONTEXT)
            gss_delete_sec_context (&min_stat, &context, GSS_C_NO_BUFFER);
        return -1;
    }

    return 0;
}

int gssapi_server_t::process_next_token (msg: &mut ZmqMessage)
{
    if (maj_stat == GSS_S_CONTINUE_NEEDED) {
        if (process_initiate (msg, &recv_tok.value, recv_tok.length) < 0) {
            if (target_name != GSS_C_NO_NAME)
                gss_release_name (&min_stat, &target_name);
            return -1;
        }
    }

    return 0;
}

void gssapi_server_t::accept_context ()
{
    maj_stat = gss_accept_sec_context (
      &init_sec_min_stat, &context, cred, &recv_tok, GSS_C_NO_CHANNEL_BINDINGS,
      &target_name, &doid, &send_tok, &ret_flags, null_mut(), null_mut());

    if (recv_tok.value) {
        free (recv_tok.value);
        recv_tok.value = null_mut();
    }
}

// #endif
