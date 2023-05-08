/*
Copyright (c) 2018 Contributors as noted in the AUTHORS file

This file is part of 0MQ.

0MQ is free software; you can redistribute it and/or modify it under
the terms of the GNU Lesser General Public License as published by
the Free Software Foundation; either version 3 of the License, or
(at your option) any later version.

0MQ is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Lesser General Public License for more details.

You should have received a copy of the GNU Lesser General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

// #include "../tests/testutil_unity.hpp"

// TODO: remove this ugly hack
// #ifdef close
#undef close
// #endif

// #include <curve_mechanism_base.hpp>
// #include <msg.hpp>
// #include <random.hpp>

// #include <unity.h>

// #include <vector>

void setUp ()
{
}

void tearDown ()
{
}

void test_roundtrip (msg: &mut ZmqMessage)
{
// #ifdef ZMQ_HAVE_CURVE
    const std::vector<uint8_t> original ( (msg.data ()),
                                          (msg.data ())
                                           + msg.size ());

    ZmqCurveEncoding encoding_client ("CurveZMQMESSAGEC",
                                           "CurveZMQMESSAGES",
                                           false);
    ZmqCurveEncoding encoding_server ("CurveZMQMESSAGES",
                                           "CurveZMQMESSAGEC",
                                           false);

    uint8_t client_public[32];
    uint8_t client_secret[32];
    TEST_ASSERT_SUCCESS_ERRNO (
      crypto_box_keypair (client_public, client_secret));

    uint8_t server_public[32];
    uint8_t server_secret[32];
    TEST_ASSERT_SUCCESS_ERRNO (
      crypto_box_keypair (server_public, server_secret));

    TEST_ASSERT_SUCCESS_ERRNO (
      crypto_box_beforenm (encoding_client.get_writable_precom_buffer (),
                           server_public, client_secret));
    TEST_ASSERT_SUCCESS_ERRNO (
      crypto_box_beforenm (encoding_server.get_writable_precom_buffer (),
                           client_public, server_secret));

    TEST_ASSERT_SUCCESS_ERRNO (encoding_client.encode (msg));

    // TODO: This is hacky...
    encoding_server.set_peer_nonce (0);
    error_event_code: i32;
    TEST_ASSERT_SUCCESS_ERRNO (
      encoding_server.decode (msg, &error_event_code));

    TEST_ASSERT_EQUAL_INT (original.size (), msg.size ());
    if (!original.empty ()) {
        TEST_ASSERT_EQUAL_UINT8_ARRAY (&original[0], msg.data (),
                                       original.size ());
    }
// #else
    TEST_IGNORE_MESSAGE ("CURVE support is disabled");
// #endif
}

void test_roundtrip_empty ()
{
let mut msg = ZmqMessage::default();
    msg.init ();

    test_roundtrip (&msg);

    msg.close ();
}

void test_roundtrip_small ()
{
let mut msg = ZmqMessage::default();
    msg.init_size (32);
    memcpy (msg.data (), "0123456789ABCDEF0123456789ABCDEF", 32);

    test_roundtrip (&msg);

    msg.close ();
}

void test_roundtrip_large ()
{
let mut msg = ZmqMessage::default();
    msg.init_size (2048);
    for (size_t pos = 0; pos < 2048; pos += 32) {
        memcpy (static_cast<char *> (msg.data ()) + pos,
                "0123456789ABCDEF0123456789ABCDEF", 32);
    }

    test_roundtrip (&msg);

    msg.close ();
}

void test_roundtrip_empty_more ()
{
let mut msg = ZmqMessage::default();
    msg.init ();
    msg.set_flags (ZMQ_MSG_MORE);

    test_roundtrip (&msg);
    TEST_ASSERT_TRUE (msg.flags () & ZMQ_MSG_MORE);

    msg.close ();
}

int main ()
{
    setup_test_environment ();
    random_open ();

    UNITY_BEGIN ();

    RUN_TEST (test_roundtrip_empty);
    RUN_TEST (test_roundtrip_small);
    RUN_TEST (test_roundtrip_large);

    RUN_TEST (test_roundtrip_empty_more);

    random_close ();

    return UNITY_END ();
}
