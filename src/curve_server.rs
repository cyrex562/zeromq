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
// #include "macros.hpp"

// #ifdef ZMQ_HAVE_CURVE

use crate::curve_mechanism_base::ZmqCurveMechanismBase;
use crate::options::ZmqOptions;
use crate::session_base::ZmqSessionBase;
use crate::zap_client::zap_client_common_handshake_t;

// #include "msg.hpp"
// #include "session_base.hpp"
// #include "err.hpp"
// #include "curve_server.hpp"
// #include "wire.hpp"
// #include "secure_allocator.hpp"
// pub struct curve_server_t ZMQ_FINAL : public zap_client_common_handshake_t,
//                                  public ZmqCurveMechanismBase
#[derive(Default,Debug,Clone)]
pub struct curve_server_t
{
    pub zap_client_common_handshake: zap_client_common_handshake_t,
    pub curve_mechanism_base: ZmqCurveMechanismBase,

  // private:
    //  Our secret key (s)
    pub _secret_key: [u8;CRYPTO_BOX_SECRETKEYBYTES],
    //  Our short-term public key (S')
    pub _cn_public: [u8; CRYPTO_BOX_PUBLICKEYBYTES],
    //  Our short-term secret key (s')
    pub _cn_secret: [u8;CRYPTO_BOX_SECRETKEYBYTES],
    //  Client's short-term public key (C')
    pub _cn_client: [u8;CRYPTO_BOX_PUBLICKEYBYTES],
    //  Key used to produce cookie
    pub _cookie_key: [u8; CRYPTO_SECRETBOX_KEYBYTES],
}

impl curve_server_t {
    // public:
    // curve_server_t (ZmqSessionBase *session_,
    //                 const std::string &peer_address_,
    //                 options: &ZmqOptions,
    //                 const downgrade_sub_: bool);
    pub fn new(session: &mut ZmqSessionBase, peer_address: &str, options: &ZmqOptions, downgrade_sub: bool) -> Self {
        let mut mechanism_base =
        Self {

        }
    }

    // ~curve_server_t ();

    // mechanism implementation
    // int next_handshake_command (msg: &mut ZmqMessage);

    // int process_handshake_command (msg: &mut ZmqMessage);

    // int encode (msg: &mut ZmqMessage);

    // int decode (msg: &mut ZmqMessage);

    // int process_hello (msg: &mut ZmqMessage);

    // int produce_welcome (msg: &mut ZmqMessage);

    // int process_initiate (msg: &mut ZmqMessage);

    // int produce_ready (msg: &mut ZmqMessage);

    // int produce_error (msg: &mut ZmqMessage) const;

    // void send_zap_request (const key_: &mut [u8]);
}

curve_server_t::curve_server_t (ZmqSessionBase *session_,
                                     const std::string &peer_address_,
                                     options: &ZmqOptions,
                                     const downgrade_sub_: bool) :
    ZmqMechanismBase (session_, options_),
    zap_client_common_handshake_t (
      session_, peer_address_, options_, sending_ready),
    ZmqCurveMechanismBase (session_,
                            options_,
                            "CurveZMQMESSAGES",
                            "CurveZMQMESSAGEC",
                            downgrade_sub_)
{
    rc: i32;
    //  Fetch our secret key from socket options
    memcpy (_secret_key, options_.curve_secret_key, CRYPTO_BOX_SECRETKEYBYTES);

    //  Generate short-term key pair
    memset (_cn_secret, 0, CRYPTO_BOX_SECRETKEYBYTES);
    memset (_cn_public, 0, CRYPTO_BOX_PUBLICKEYBYTES);
    rc = crypto_box_keypair (_cn_public, _cn_secret);
    zmq_assert (rc == 0);
}

curve_server_t::~curve_server_t ()
{
}

int curve_server_t::next_handshake_command (msg: &mut ZmqMessage)
{
    int rc = 0;

    switch (state) {
        case sending_welcome:
            rc = produce_welcome (msg);
            if (rc == 0)
                state = waiting_for_initiate;
            break;
        case sending_ready:
            rc = produce_ready (msg);
            if (rc == 0)
                state = ready;
            break;
        case sending_error:
            rc = produce_error (msg);
            if (rc == 0)
                state = error_sent;
            break;
        _ =>
            errno = EAGAIN;
            rc = -1;
            break;
    }
    return rc;
}

int curve_server_t::process_handshake_command (msg: &mut ZmqMessage)
{
    int rc = 0;

    switch (state) {
        case waiting_for_hello:
            rc = process_hello (msg);
            break;
        case waiting_for_initiate:
            rc = process_initiate (msg);
            break;
        _ =>
            // TODO I think this is not a case reachable with a misbehaving
            // client. It is not an "invalid handshake command", but would be
            // trying to process a handshake command in an invalid state,
            // which is purely under control of this peer.
            // Therefore, it should be changed to zmq_assert (false);

            // CURVE I: invalid handshake command
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

int curve_server_t::encode (msg: &mut ZmqMessage)
{
    zmq_assert (state == ready);
    return ZmqCurveMechanismBase::encode (msg);
}

int curve_server_t::decode (msg: &mut ZmqMessage)
{
    zmq_assert (state == ready);
    return ZmqCurveMechanismBase::decode (msg);
}

int curve_server_t::process_hello (msg: &mut ZmqMessage)
{
    int rc = check_basic_command_structure (msg);
    if (rc == -1)
        return -1;

    const size_t size = msg.size ();
    const uint8_t *const hello = static_cast<uint8_t *> (msg.data ());

    if (size < 6 || memcmp (hello, "\x05HELLO", 6)) {
        session.get_socket ().event_handshake_failed_protocol (
          session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND);
        errno = EPROTO;
        return -1;
    }

    if (size != 200) {
        session.get_socket ().event_handshake_failed_protocol (
          session.get_endpoint (),
          ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_HELLO);
        errno = EPROTO;
        return -1;
    }

    const uint8_t major = hello[6];
    const uint8_t minor = hello[7];

    if (major != 1 || minor != 0) {
        // CURVE I: client HELLO has unknown version number
        session.get_socket ().event_handshake_failed_protocol (
          session.get_endpoint (),
          ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_HELLO);
        errno = EPROTO;
        return -1;
    }

    //  Save client's short-term public key (C')
    memcpy (_cn_client, hello + 80, 32);

    uint8_t hello_nonce[CRYPTO_BOX_NONCEBYTES];
    std::vector<uint8_t, secure_allocator_t<uint8_t> > hello_plaintext (
      CRYPTO_BOX_ZEROBYTES + 64);
    uint8_t hello_box[CRYPTO_BOX_BOXZEROBYTES + 80];

    memcpy (hello_nonce, "CurveZMQHELLO---", 16);
    memcpy (hello_nonce + 16, hello + 112, 8);
    set_peer_nonce (get_uint64 (hello + 112));

    memset (hello_box, 0, CRYPTO_BOX_BOXZEROBYTES);
    memcpy (hello_box + CRYPTO_BOX_BOXZEROBYTES, hello + 120, 80);

    //  Open Box [64 * %x0](C'->S)
    rc = crypto_box_open (&hello_plaintext[0], hello_box, sizeof hello_box,
                          hello_nonce, _cn_client, _secret_key);
    if (rc != 0) {
        // CURVE I: cannot open client HELLO -- wrong server key?
        session.get_socket ().event_handshake_failed_protocol (
          session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_CRYPTOGRAPHIC);
        errno = EPROTO;
        return -1;
    }

    state = sending_welcome;
    return rc;
}

int curve_server_t::produce_welcome (msg: &mut ZmqMessage)
{
    uint8_t cookie_nonce[CRYPTO_SECRETBOX_NONCEBYTES];
    std::vector<uint8_t, secure_allocator_t<uint8_t> > cookie_plaintext (
      CRYPTO_SECRETBOX_ZEROBYTES + 64);
    uint8_t cookie_ciphertext[CRYPTO_SECRETBOX_BOXZEROBYTES + 80];

    //  Create full nonce for encryption
    //  8-byte prefix plus 16-byte random nonce
    memset (cookie_nonce, 0, CRYPTO_SECRETBOX_NONCEBYTES);
    memcpy (cookie_nonce, "COOKIE--", 8);
    randombytes (cookie_nonce + 8, 16);

    //  Generate cookie = Box [C' + s'](t)
    std::fill (cookie_plaintext.begin (),
               cookie_plaintext.begin () + CRYPTO_SECRETBOX_ZEROBYTES, 0);
    memcpy (&cookie_plaintext[CRYPTO_SECRETBOX_ZEROBYTES], _cn_client, 32);
    memcpy (&cookie_plaintext[CRYPTO_SECRETBOX_ZEROBYTES + 32], _cn_secret, 32);

    //  Generate fresh cookie key
    memset (_cookie_key, 0, CRYPTO_SECRETBOX_KEYBYTES);
    randombytes (_cookie_key, CRYPTO_SECRETBOX_KEYBYTES);

    //  Encrypt using symmetric cookie key
    int rc =
      crypto_secretbox (cookie_ciphertext, &cookie_plaintext[0],
                        cookie_plaintext.size (), cookie_nonce, _cookie_key);
    zmq_assert (rc == 0);

    uint8_t welcome_nonce[CRYPTO_BOX_NONCEBYTES];
    std::vector<uint8_t, secure_allocator_t<uint8_t> > welcome_plaintext (
      CRYPTO_BOX_ZEROBYTES + 128);
    uint8_t welcome_ciphertext[CRYPTO_BOX_BOXZEROBYTES + 144];

    //  Create full nonce for encryption
    //  8-byte prefix plus 16-byte random nonce
    memset (welcome_nonce, 0, CRYPTO_BOX_NONCEBYTES);
    memcpy (welcome_nonce, "WELCOME-", 8);
    randombytes (welcome_nonce + 8, CRYPTO_BOX_NONCEBYTES - 8);

    //  Create 144-byte Box [S' + cookie](S->C')
    std::fill (welcome_plaintext.begin (),
               welcome_plaintext.begin () + CRYPTO_BOX_ZEROBYTES, 0);
    memcpy (&welcome_plaintext[CRYPTO_BOX_ZEROBYTES], _cn_public, 32);
    memcpy (&welcome_plaintext[CRYPTO_BOX_ZEROBYTES + 32], cookie_nonce + 8,
            16);
    memcpy (&welcome_plaintext[CRYPTO_BOX_ZEROBYTES + 48],
            cookie_ciphertext + CRYPTO_SECRETBOX_BOXZEROBYTES, 80);

    rc = crypto_box (welcome_ciphertext, &welcome_plaintext[0],
                     welcome_plaintext.size (), welcome_nonce, _cn_client,
                     _secret_key);

    //  TODO I think we should change this back to zmq_assert (rc == 0);
    //  as it was before https://github.com/zeromq/libzmq/pull/1832
    //  The reason given there was that secret_key might be 0ed.
    //  But if it were, we would never get this far, since we could
    //  not have opened the client's hello box with a 0ed key.

    if (rc == -1)
        return -1;

    rc = msg.init_size (168);
    errno_assert (rc == 0);

    uint8_t *const welcome = static_cast<uint8_t *> (msg.data ());
    memcpy (welcome, "\x07WELCOME", 8);
    memcpy (welcome + 8, welcome_nonce + 8, 16);
    memcpy (welcome + 24, welcome_ciphertext + CRYPTO_BOX_BOXZEROBYTES, 144);

    return 0;
}

int curve_server_t::process_initiate (msg: &mut ZmqMessage)
{
    int rc = check_basic_command_structure (msg);
    if (rc == -1)
        return -1;

    const size_t size = msg.size ();
    const uint8_t *initiate = static_cast<uint8_t *> (msg.data ());

    if (size < 9 || memcmp (initiate, "\x08INITIATE", 9)) {
        session.get_socket ().event_handshake_failed_protocol (
          session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND);
        errno = EPROTO;
        return -1;
    }

    if (size < 257) {
        session.get_socket ().event_handshake_failed_protocol (
          session.get_endpoint (),
          ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_INITIATE);
        errno = EPROTO;
        return -1;
    }

    uint8_t cookie_nonce[CRYPTO_SECRETBOX_NONCEBYTES];
    uint8_t cookie_plaintext[CRYPTO_SECRETBOX_ZEROBYTES + 64];
    uint8_t cookie_box[CRYPTO_SECRETBOX_BOXZEROBYTES + 80];

    //  Open Box [C' + s'](t)
    memset (cookie_box, 0, CRYPTO_SECRETBOX_BOXZEROBYTES);
    memcpy (cookie_box + CRYPTO_SECRETBOX_BOXZEROBYTES, initiate + 25, 80);

    memcpy (cookie_nonce, "COOKIE--", 8);
    memcpy (cookie_nonce + 8, initiate + 9, 16);

    rc = crypto_secretbox_open (cookie_plaintext, cookie_box, sizeof cookie_box,
                                cookie_nonce, _cookie_key);
    if (rc != 0) {
        // CURVE I: cannot open client INITIATE cookie
        session.get_socket ().event_handshake_failed_protocol (
          session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_CRYPTOGRAPHIC);
        errno = EPROTO;
        return -1;
    }

    //  Check cookie plain text is as expected [C' + s']
    if (memcmp (cookie_plaintext + CRYPTO_SECRETBOX_ZEROBYTES, _cn_client, 32)
        || memcmp (cookie_plaintext + CRYPTO_SECRETBOX_ZEROBYTES + 32,
                   _cn_secret, 32)) {
        // TODO this case is very hard to test, as it would require a modified
        //  client that knows the server's secret temporary cookie key

        // CURVE I: client INITIATE cookie is not valid
        session.get_socket ().event_handshake_failed_protocol (
          session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_CRYPTOGRAPHIC);
        errno = EPROTO;
        return -1;
    }

    const size_t clen = (size - 113) + CRYPTO_BOX_BOXZEROBYTES;

    uint8_t initiate_nonce[CRYPTO_BOX_NONCEBYTES];
    std::vector<uint8_t, secure_allocator_t<uint8_t> > initiate_plaintext (
      CRYPTO_BOX_ZEROBYTES + clen);
    std::vector<uint8_t> initiate_box (CRYPTO_BOX_BOXZEROBYTES + clen);

    //  Open Box [C + vouch + metadata](C'->S')
    std::fill (initiate_box.begin (),
               initiate_box.begin () + CRYPTO_BOX_BOXZEROBYTES, 0);
    memcpy (&initiate_box[CRYPTO_BOX_BOXZEROBYTES], initiate + 113,
            clen - CRYPTO_BOX_BOXZEROBYTES);

    memcpy (initiate_nonce, "CurveZMQINITIATE", 16);
    memcpy (initiate_nonce + 16, initiate + 105, 8);
    set_peer_nonce (get_uint64 (initiate + 105));

    const uint8_t *client_key = &initiate_plaintext[CRYPTO_BOX_ZEROBYTES];

    rc = crypto_box_open (&initiate_plaintext[0], &initiate_box[0], clen,
                          initiate_nonce, _cn_client, _cn_secret);
    if (rc != 0) {
        // CURVE I: cannot open client INITIATE
        session.get_socket ().event_handshake_failed_protocol (
          session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_CRYPTOGRAPHIC);
        errno = EPROTO;
        return -1;
    }

    uint8_t vouch_nonce[CRYPTO_BOX_NONCEBYTES];
    std::vector<uint8_t, secure_allocator_t<uint8_t> > vouch_plaintext (
      CRYPTO_BOX_ZEROBYTES + 64);
    uint8_t vouch_box[CRYPTO_BOX_BOXZEROBYTES + 80];

    //  Open Box Box [C',S](C->S') and check contents
    memset (vouch_box, 0, CRYPTO_BOX_BOXZEROBYTES);
    memcpy (vouch_box + CRYPTO_BOX_BOXZEROBYTES,
            &initiate_plaintext[CRYPTO_BOX_ZEROBYTES + 48], 80);

    memset (vouch_nonce, 0, CRYPTO_BOX_NONCEBYTES);
    memcpy (vouch_nonce, "VOUCH---", 8);
    memcpy (vouch_nonce + 8, &initiate_plaintext[CRYPTO_BOX_ZEROBYTES + 32],
            16);

    rc = crypto_box_open (&vouch_plaintext[0], vouch_box, sizeof vouch_box,
                          vouch_nonce, client_key, _cn_secret);
    if (rc != 0) {
        // CURVE I: cannot open client INITIATE vouch
        session.get_socket ().event_handshake_failed_protocol (
          session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_CRYPTOGRAPHIC);
        errno = EPROTO;
        return -1;
    }

    //  What we decrypted must be the client's short-term public key
    if (memcmp (&vouch_plaintext[CRYPTO_BOX_ZEROBYTES], _cn_client, 32)) {
        // TODO this case is very hard to test, as it would require a modified
        //  client that knows the server's secret short-term key

        // CURVE I: invalid handshake from client (public key)
        session.get_socket ().event_handshake_failed_protocol (
          session.get_endpoint (), ZMQ_PROTOCOL_ERROR_ZMTP_KEY_EXCHANGE);
        errno = EPROTO;
        return -1;
    }

    //  Precompute connection secret from client key
    rc = crypto_box_beforenm (get_writable_precom_buffer (), _cn_client,
                              _cn_secret);
    zmq_assert (rc == 0);

    //  Given this is a backward-incompatible change, it's behind a socket
    //  option disabled by default.
    if (zap_required () || !options.zap_enforce_domain) {
        //  Use ZAP protocol (RFC 27) to authenticate the user.
        rc = session.zap_connect ();
        if (rc == 0) {
            send_zap_request (client_key);
            state = waiting_for_zap_reply;

            //  TODO actually, it is quite unlikely that we can read the ZAP
            //  reply already, but removing this has some strange side-effect
            //  (probably because the pipe's in_active flag is true until a read
            //  is attempted)
            if (-1 == receive_and_process_zap_reply ())
                return -1;
        } else if (!options.zap_enforce_domain) {
            //  This supports the Stonehouse pattern (encryption without
            //  authentication) in legacy mode (domain set but no handler).
            state = sending_ready;
        } else {
            session.get_socket ()->event_handshake_failed_no_detail (
              session.get_endpoint (), EFAULT);
            return -1;
        }
    } else {
        //  This supports the Stonehouse pattern (encryption without authentication).
        state = sending_ready;
    }

    return parse_metadata (&initiate_plaintext[CRYPTO_BOX_ZEROBYTES + 128],
                           clen - CRYPTO_BOX_ZEROBYTES - 128);
}

int curve_server_t::produce_ready (msg: &mut ZmqMessage)
{
    const size_t metadata_length = basic_properties_len ();
    uint8_t ready_nonce[CRYPTO_BOX_NONCEBYTES];

    std::vector<uint8_t, secure_allocator_t<uint8_t> > ready_plaintext (
      CRYPTO_BOX_ZEROBYTES + metadata_length);

    //  Create Box [metadata](S'->C')
    std::fill (ready_plaintext.begin (),
               ready_plaintext.begin () + CRYPTO_BOX_ZEROBYTES, 0);
    uint8_t *ptr = &ready_plaintext[CRYPTO_BOX_ZEROBYTES];

    ptr += add_basic_properties (ptr, metadata_length);
    const size_t mlen = ptr - &ready_plaintext[0];

    memcpy (ready_nonce, "CurveZMQREADY---", 16);
    put_uint64 (ready_nonce + 16, get_and_inc_nonce ());

    std::vector<uint8_t> ready_box (CRYPTO_BOX_BOXZEROBYTES + 16
                                    + metadata_length);

    int rc = crypto_box_afternm (&ready_box[0], &ready_plaintext[0], mlen,
                                 ready_nonce, get_precom_buffer ());
    zmq_assert (rc == 0);

    rc = msg.init_size (14 + mlen - CRYPTO_BOX_BOXZEROBYTES);
    errno_assert (rc == 0);

    uint8_t *ready = static_cast<uint8_t *> (msg.data ());

    memcpy (ready, "\x05READY", 6);
    //  Short nonce, prefixed by "CurveZMQREADY---"
    memcpy (ready + 6, ready_nonce + 16, 8);
    //  Box [metadata](S'->C')
    memcpy (ready + 14, &ready_box[CRYPTO_BOX_BOXZEROBYTES],
            mlen - CRYPTO_BOX_BOXZEROBYTES);

    return 0;
}

int curve_server_t::produce_error (msg: &mut ZmqMessage) const
{
    const size_t expected_status_code_length = 3;
    zmq_assert (status_code.length () == 3);
    let rc: i32 = msg.init_size (6 + 1 + expected_status_code_length);
    zmq_assert (rc == 0);
    char *msg_data = static_cast<char *> (msg.data ());
    memcpy (msg_data, "\5ERROR", 6);
    msg_data[6] = expected_status_code_length;
    memcpy (msg_data + 7, status_code, expected_status_code_length);
    return 0;
}

void curve_server_t::send_zap_request (const key_: &mut [u8])
{
    zap_client_t::send_zap_request ("CURVE", 5, key_,
                                    CRYPTO_BOX_PUBLICKEYBYTES);
}

// #endif
