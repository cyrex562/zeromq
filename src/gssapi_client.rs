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
// #include "gssapi_client.hpp"
// #include "wire.hpp"
pub struct gssapi_client_t ZMQ_FINAL : public gssapi_mechanism_base_t
{
// public:
    gssapi_client_t (session_base_t *session_, const ZmqOptions &options_);
    ~gssapi_client_t () ZMQ_FINAL;

    // mechanism implementation
    int next_handshake_command (ZmqMessage *msg) ZMQ_FINAL;
    int process_handshake_command (ZmqMessage *msg) ZMQ_FINAL;
    int encode (ZmqMessage *msg) ZMQ_FINAL;
    int decode (ZmqMessage *msg) ZMQ_FINAL;
    status_t status () const ZMQ_FINAL;

  // private:
    enum state_t
    {
        call_next_init,
        send_next_token,
        recv_next_token,
        send_ready,
        recv_ready,
        connected
    };

    //  Human-readable principal name of the service we are connecting to
    char *service_name;

    gss_OID service_name_type;

    //  Current FSM state
    state_t state;

    //  Points to either send_tok or recv_tok
    //  during context initialization
    gss_buffer_desc *token_ptr;

    //  The desired underlying mechanism
    gss_OID_set_desc mechs;

    //  True iff client considers the server authenticated
    bool security_context_established;

    int initialize_context ();
    int produce_next_token (ZmqMessage *msg);
    int process_next_token (ZmqMessage *msg);
};

zmq::gssapi_client_t::gssapi_client_t (session_base_t *session_,
                                       const ZmqOptions &options_) :
    mechanism_base_t (session_, options_),
    gssapi_mechanism_base_t (session_, options_),
    state (call_next_init),
    token_ptr (GSS_C_NO_BUFFER),
    mechs (),
    security_context_established (false)
{
    const std::string::size_type service_size =
      options_.gss_service_principal.size ();
    service_name = static_cast<char *> (malloc (service_size + 1));
    assert (service_name);
    memcpy (service_name, options_.gss_service_principal,
            service_size + 1);

    service_name_type = convert_nametype (options_.gss_service_principal_nt);
    maj_stat = GSS_S_COMPLETE;
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

    mechs.elements = NULL;
    mechs.count = 0;
}

zmq::gssapi_client_t::~gssapi_client_t ()
{
    if (service_name)
        free (service_name);
    if (cred)
        gss_release_cred (&min_stat, &cred);
}

int zmq::gssapi_client_t::next_handshake_command (ZmqMessage *msg)
{
    if (state == send_ready) {
        int rc = produce_ready (msg);
        if (rc == 0)
            state = connected;

        return rc;
    }

    if (state != call_next_init) {
        errno = EAGAIN;
        return -1;
    }

    if (initialize_context () < 0)
        return -1;

    if (produce_next_token (msg) < 0)
        return -1;

    if (maj_stat != GSS_S_CONTINUE_NEEDED && maj_stat != GSS_S_COMPLETE)
        return -1;

    if (maj_stat == GSS_S_COMPLETE) {
        security_context_established = true;
        state = recv_ready;
    } else
        state = recv_next_token;

    return 0;
}

int zmq::gssapi_client_t::process_handshake_command (ZmqMessage *msg)
{
    if (state == recv_ready) {
        int rc = process_ready (msg);
        if (rc == 0)
            state = send_ready;

        return rc;
    }

    if (state != recv_next_token) {
        session->get_socket ()->event_handshake_failed_protocol (
          session->get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND);
        errno = EPROTO;
        return -1;
    }

    if (process_next_token (msg) < 0)
        return -1;

    if (maj_stat != GSS_S_COMPLETE && maj_stat != GSS_S_CONTINUE_NEEDED)
        return -1;

    state = call_next_init;

    errno_assert (msg->close () == 0);
    errno_assert (msg->init () == 0);

    return 0;
}

int zmq::gssapi_client_t::encode (ZmqMessage *msg)
{
    zmq_assert (state == connected);

    if (do_encryption)
        return encode_message (msg);

    return 0;
}

int zmq::gssapi_client_t::decode (ZmqMessage *msg)
{
    zmq_assert (state == connected);

    if (do_encryption)
        return decode_message (msg);

    return 0;
}

zmq::mechanism_t::status_t zmq::gssapi_client_t::status () const
{
    return state == connected ? mechanism_t::ready : mechanism_t::handshaking;
}

int zmq::gssapi_client_t::initialize_context ()
{
    // principal was specified but credentials could not be acquired
    if (principal_name != NULL && cred == NULL)
        return -1;

    // First time through, import service_name into target_name
    if (target_name == GSS_C_NO_NAME) {
        send_tok.value = service_name;
        send_tok.length = strlen (service_name) + 1;
        OM_uint32 maj = gss_import_name (&min_stat, &send_tok,
                                         service_name_type, &target_name);

        if (maj != GSS_S_COMPLETE)
            return -1;
    }

    maj_stat = gss_init_sec_context (
      &init_sec_min_stat, cred, &context, target_name, mechs.elements,
      gss_flags, 0, NULL, token_ptr, NULL, &send_tok, &ret_flags, NULL);

    if (token_ptr != GSS_C_NO_BUFFER)
        free (recv_tok.value);

    return 0;
}

int zmq::gssapi_client_t::produce_next_token (ZmqMessage *msg)
{
    if (send_tok.length != 0) { // Server expects another token
        if (produce_initiate (msg, send_tok.value, send_tok.length) < 0) {
            gss_release_buffer (&min_stat, &send_tok);
            gss_release_name (&min_stat, &target_name);
            return -1;
        }
    }
    gss_release_buffer (&min_stat, &send_tok);

    if (maj_stat != GSS_S_COMPLETE && maj_stat != GSS_S_CONTINUE_NEEDED) {
        gss_release_name (&min_stat, &target_name);
        if (context != GSS_C_NO_CONTEXT)
            gss_delete_sec_context (&min_stat, &context, GSS_C_NO_BUFFER);
        return -1;
    }

    return 0;
}

int zmq::gssapi_client_t::process_next_token (ZmqMessage *msg)
{
    if (maj_stat == GSS_S_CONTINUE_NEEDED) {
        if (process_initiate (msg, &recv_tok.value, recv_tok.length) < 0) {
            gss_release_name (&min_stat, &target_name);
            return -1;
        }
        token_ptr = &recv_tok;
    }

    return 0;
}

// #endif
