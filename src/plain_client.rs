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

// #include <string>
// #include <limits.h>

// #include "msg.hpp"
// #include "err.hpp"
// #include "plain_client.hpp"
// #include "session_base.hpp"
// #include "plain_common.hpp"
pub struct plain_client_t  : public ZmqMechanismBase
{
// public:
    plain_client_t (ZmqSessionBase *session_, options: &ZmqOptions);
    ~plain_client_t ();

    // mechanism implementation
    int next_handshake_command (msg: &mut ZmqMessage);
    int process_handshake_command (msg: &mut ZmqMessage);
    status_t status () const;

  // private:
    enum state_t
    {
        sending_hello,
        waiting_for_welcome,
        sending_initiate,
        waiting_for_ready,
        error_command_received,
        ready
    };

    state_t _state;

    void produce_hello (msg: &mut ZmqMessage) const;
    void produce_initiate (msg: &mut ZmqMessage) const;

    int process_welcome (const cmd_data_: &mut [u8], data_size_: usize);
    int process_ready (const cmd_data_: &mut [u8], data_size_: usize);
    int process_error (const cmd_data_: &mut [u8], data_size_: usize);
};

plain_client_t::plain_client_t (ZmqSessionBase *const session_,
                                     options: &ZmqOptions) :
    ZmqMechanismBase (session_, options_), _state (sending_hello)
{
}

plain_client_t::~plain_client_t ()
{
}

int plain_client_t::next_handshake_command (msg: &mut ZmqMessage)
{
    int rc = 0;

    switch (_state) {
        case sending_hello:
            produce_hello (msg);
            _state = waiting_for_welcome;
            break;
        case sending_initiate:
            produce_initiate (msg);
            _state = waiting_for_ready;
            break;
        _ =>
            errno = EAGAIN;
            rc = -1;
    }
    return rc;
}

int plain_client_t::process_handshake_command (msg: &mut ZmqMessage)
{
    const unsigned char *cmd_data =
       (msg.data ());
    const size_t data_size = msg.size ();

    int rc = 0;
    if (data_size >= welcome_prefix_len
        && !memcmp (cmd_data, welcome_prefix, welcome_prefix_len))
        rc = process_welcome (cmd_data, data_size);
    else if (data_size >= ready_prefix_len
             && !memcmp (cmd_data, ready_prefix, ready_prefix_len))
        rc = process_ready (cmd_data, data_size);
    else if (data_size >= error_prefix_len
             && !memcmp (cmd_data, error_prefix, error_prefix_len))
        rc = process_error (cmd_data, data_size);
    else {
        session.get_socket ().event_handshake_failed_protocol (
          session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND);
        errno = EPROTO;
        rc = -1;
    }

    if (rc == 0) {
        rc = msg.close ();
        errno_assert (rc == 0);
        rc = msg.init ();
        errno_assert (rc == 0);
    }

    return rc;
}

ZmqMechanism::status_t plain_client_t::status () const
{
    switch (_state) {
        case ready:
            return ZmqMechanism::ready;
        case error_command_received:
            return ZmqMechanism::error;
        _ =>
            return ZmqMechanism::handshaking;
    }
}

void plain_client_t::produce_hello (msg: &mut ZmqMessage) const
{
    const std::string username = options.plain_username;
    zmq_assert (username.length () <= UCHAR_MAX);

    const std::string password = options.plain_password;
    zmq_assert (password.length () <= UCHAR_MAX);

    const size_t command_size = hello_prefix_len + brief_len_size
                                + username.length () + brief_len_size
                                + password.length ();

    let rc: i32 = msg.init_size (command_size);
    errno_assert (rc == 0);

    unsigned char *ptr =  (msg.data ());
    memcpy (ptr, hello_prefix, hello_prefix_len);
    ptr += hello_prefix_len;

    *ptr+= 1 = static_cast<unsigned char> (username.length ());
    memcpy (ptr, username, username.length ());
    ptr += username.length ();

    *ptr+= 1 = static_cast<unsigned char> (password.length ());
    memcpy (ptr, password, password.length ());
}

int plain_client_t::process_welcome (const cmd_data_: &mut [u8],
                                          data_size_: usize)
{
    LIBZMQ_UNUSED (cmd_data_);

    if (_state != waiting_for_welcome) {
        session.get_socket ().event_handshake_failed_protocol (
          session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND);
        errno = EPROTO;
        return -1;
    }
    if (data_size_ != welcome_prefix_len) {
        session.get_socket ().event_handshake_failed_protocol (
          session.get_endpoint (),
          ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_WELCOME);
        errno = EPROTO;
        return -1;
    }
    _state = sending_initiate;
    return 0;
}

void plain_client_t::produce_initiate (msg: &mut ZmqMessage) const
{
    make_command_with_basic_properties (msg, initiate_prefix,
                                        initiate_prefix_len);
}

int plain_client_t::process_ready (const cmd_data_: &mut [u8],
                                        data_size_: usize)
{
    if (_state != waiting_for_ready) {
        session.get_socket ().event_handshake_failed_protocol (
          session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND);
        errno = EPROTO;
        return -1;
    }
    let rc: i32 = parse_metadata (cmd_data_ + ready_prefix_len,
                                   data_size_ - ready_prefix_len);
    if (rc == 0)
        _state = ready;
    else
        session.get_socket ().event_handshake_failed_protocol (
          session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_METADATA);

    return rc;
}

int plain_client_t::process_error (const cmd_data_: &mut [u8],
                                        data_size_: usize)
{
    if (_state != waiting_for_welcome && _state != waiting_for_ready) {
        session.get_socket ().event_handshake_failed_protocol (
          session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND);
        errno = EPROTO;
        return -1;
    }
    const size_t start_of_error_reason = error_prefix_len + brief_len_size;
    if (data_size_ < start_of_error_reason) {
        session.get_socket ().event_handshake_failed_protocol (
          session.get_endpoint (),
          ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR);
        errno = EPROTO;
        return -1;
    }
    const size_t error_reason_len =
      static_cast<size_t> (cmd_data_[error_prefix_len]);
    if (error_reason_len > data_size_ - start_of_error_reason) {
        session.get_socket ()->event_handshake_failed_protocol (
          session.get_endpoint (),
          ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR);
        errno = EPROTO;
        return -1;
    }
    const char *error_reason =
      reinterpret_cast<const char *> (cmd_data_) + start_of_error_reason;
    handle_error_reason (error_reason, error_reason_len);
    _state = error_command_received;
    return 0;
}
