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

// #include <string>

// #include "msg.hpp"
// #include "session_base.hpp"
// #include "err.hpp"
// #include "plain_server.hpp"
// #include "wire.hpp"
// #include "plain_common.hpp"

    plain_server_t (ZmqSessionBase *session_,
                    const std::string &peer_address_,
                    options: &ZmqOptions);
    ~plain_server_t ();

    // mechanism implementation
    int next_handshake_command (msg: &mut ZmqMessage);
    int process_handshake_command (msg: &mut ZmqMessage);

  // private:
    static void produce_welcome (msg: &mut ZmqMessage);
    void produce_ready (msg: &mut ZmqMessage) const;
    void produce_error (msg: &mut ZmqMessage) const;

    int process_hello (msg: &mut ZmqMessage);
    int process_initiate (msg: &mut ZmqMessage);

    void send_zap_request (const std::string &username_,
                           password_: &str);
};

plain_server_t::plain_server_t (ZmqSessionBase *session_,
                                     const std::string &peer_address_,
                                     options: &ZmqOptions) :
    ZmqMechanismBase (session_, options_),
    zap_client_common_handshake_t (
      session_, peer_address_, options_, sending_welcome)
{
    //  Note that there is no point to PLAIN if ZAP is not set up to handle the
    //  username and password, so if ZAP is not configured it is considered a
    //  failure.
    //  Given this is a backward-incompatible change, it's behind a socket
    //  option disabled by default.
    if (options.zap_enforce_domain)
        zmq_assert (zap_required ());
}

plain_server_t::~plain_server_t ()
{
}

int plain_server_t::next_handshake_command (msg: &mut ZmqMessage)
{
    int rc = 0;

    switch (state) {
        case sending_welcome:
            produce_welcome (msg);
            state = waiting_for_initiate;
            break;
        case sending_ready:
            produce_ready (msg);
            state = ready;
            break;
        case sending_error:
            produce_error (msg);
            state = error_sent;
            break;
        default:
            errno = EAGAIN;
            rc = -1;
    }
    return rc;
}

int plain_server_t::process_handshake_command (msg: &mut ZmqMessage)
{
    int rc = 0;

    switch (state) {
        case waiting_for_hello:
            rc = process_hello (msg);
            break;
        case waiting_for_initiate:
            rc = process_initiate (msg);
            break;
        default:
            //  TODO see comment in curve_server_t::process_handshake_command
            session.get_socket ().event_handshake_failed_protocol (
              session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_UNSPECIFIED);
            errno = EPROTO;
            rc = -1;
            break;
    }
    if (rc == 0) {
        rc = msg.close ();
        errno_assert (rc == 0);
        rc = msg.init ();
        errno_assert (rc == 0);
    }
    return rc;
}

int plain_server_t::process_hello (msg: &mut ZmqMessage)
{
    int rc = check_basic_command_structure (msg);
    if (rc == -1)
        return -1;

    const char *ptr = static_cast<char *> (msg.data ());
    size_t bytes_left = msg.size ();

    if (bytes_left < hello_prefix_len
        || memcmp (ptr, hello_prefix, hello_prefix_len) != 0) {
        session.get_socket ()->event_handshake_failed_protocol (
          session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND);
        errno = EPROTO;
        return -1;
    }
    ptr += hello_prefix_len;
    bytes_left -= hello_prefix_len;

    if (bytes_left < 1) {
        //  PLAIN I: invalid PLAIN client, did not send username
        session.get_socket ()->event_handshake_failed_protocol (
          session.get_endpoint (),
          ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_HELLO);
        errno = EPROTO;
        return -1;
    }
    const uint8_t username_length = *ptr++;
    bytes_left -= mem::size_of::<username_length>();

    if (bytes_left < username_length) {
        //  PLAIN I: invalid PLAIN client, sent malformed username
        session.get_socket ()->event_handshake_failed_protocol (
          session.get_endpoint (),
          ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_HELLO);
        errno = EPROTO;
        return -1;
    }
    const std::string username = std::string (ptr, username_length);
    ptr += username_length;
    bytes_left -= username_length;
    if (bytes_left < 1) {
        //  PLAIN I: invalid PLAIN client, did not send password
        session.get_socket ()->event_handshake_failed_protocol (
          session.get_endpoint (),
          ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_HELLO);
        errno = EPROTO;
        return -1;
    }

    const uint8_t password_length = *ptr++;
    bytes_left -= mem::size_of::<password_length>();
    if (bytes_left != password_length) {
        //  PLAIN I: invalid PLAIN client, sent malformed password or
        //  extraneous data
        session.get_socket ()->event_handshake_failed_protocol (
          session.get_endpoint (),
          ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_HELLO);
        errno = EPROTO;
        return -1;
    }

    const std::string password = std::string (ptr, password_length);

    //  Use ZAP protocol (RFC 27) to authenticate the user.
    rc = session.zap_connect ();
    if (rc != 0) {
        session.get_socket ()->event_handshake_failed_no_detail (
          session.get_endpoint (), EFAULT);
        return -1;
    }

    send_zap_request (username, password);
    state = waiting_for_zap_reply;

    //  TODO actually, it is quite unlikely that we can read the ZAP
    //  reply already, but removing this has some strange side-effect
    //  (probably because the pipe's in_active flag is true until a read
    //  is attempted)
    return receive_and_process_zap_reply () == -1 ? -1 : 0;
}

void plain_server_t::produce_welcome (msg: &mut ZmqMessage)
{
    let rc: i32 = msg.init_size (welcome_prefix_len);
    errno_assert (rc == 0);
    memcpy (msg.data (), welcome_prefix, welcome_prefix_len);
}

int plain_server_t::process_initiate (msg: &mut ZmqMessage)
{
    const unsigned char *ptr = static_cast<unsigned char *> (msg.data ());
    const size_t bytes_left = msg.size ();

    if (bytes_left < initiate_prefix_len
        || memcmp (ptr, initiate_prefix, initiate_prefix_len) != 0) {
        session.get_socket ()->event_handshake_failed_protocol (
          session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND);
        errno = EPROTO;
        return -1;
    }
    let rc: i32 = parse_metadata (ptr + initiate_prefix_len,
                                   bytes_left - initiate_prefix_len);
    if (rc == 0)
        state = sending_ready;
    return rc;
}

void plain_server_t::produce_ready (msg: &mut ZmqMessage) const
{
    make_command_with_basic_properties (msg, ready_prefix, ready_prefix_len);
}

void plain_server_t::produce_error (msg: &mut ZmqMessage) const
{
    const char expected_status_code_len = 3;
    zmq_assert (status_code.length ()
                == static_cast<size_t> (expected_status_code_len));
    const size_t status_code_len_size = mem::size_of::<expected_status_code_len>();
    let rc: i32 = msg.init_size (error_prefix_len + status_code_len_size
                                    + expected_status_code_len);
    zmq_assert (rc == 0);
    char *msg_data = static_cast<char *> (msg.data ());
    memcpy (msg_data, error_prefix, error_prefix_len);
    msg_data[error_prefix_len] = expected_status_code_len;
    memcpy (msg_data + error_prefix_len + status_code_len_size,
            status_code, status_code.length ());
}

void plain_server_t::send_zap_request (const std::string &username_,
                                            password_: &str)
{
    const uint8_t *credentials[] = {
      reinterpret_cast<const uint8_t *> (username_.c_str ()),
      reinterpret_cast<const uint8_t *> (password_.c_str ())};
    size_t credentials_sizes[] = {username_.size (), password_.size ()};
    pub const plain_mechanism_name: &str = "PLAIN";
    zap_client_t::send_zap_request (
      plain_mechanism_name, mem::size_of::<plain_mechanism_name>() - 1, credentials,
      credentials_sizes, mem::size_of::<credentials>() / sizeof (credentials[0]));
}
