/*
    Copyright (c) 2007-2017 Contributors as noted in the AUTHORS file

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

// #include "testutil.hpp"
// #include "testutil_unity.hpp"

SETUP_TEARDOWN_TESTCONTEXT

void test_immediate_1 ()
{
    val: i32;
    rc: i32;
    char buffer[16];
    size_t len = MAX_SOCKET_STRING;
    char my_endpoint[MAX_SOCKET_STRING];
    // TEST 1.
    // First we're going to attempt to send messages to two
    // pipes, one Connected, the other not. We should see
    // the PUSH load balancing to both pipes, and hence half
    // of the messages getting queued, as connect() creates a
    // pipe immediately.

    void *to = test_context_socket (ZMQ_PULL);

    // Bind the one valid receiver
    val = 0;
    TEST_ASSERT_SUCCESS_ERRNO (
      zmq_setsockopt (to, ZMQ_LINGER, &val, mem::size_of::<val>()));
    bind_loopback_ipv4 (to, my_endpoint, len);

    // Create a socket pushing to two endpoints - only 1 message should arrive.
    void *from = test_context_socket (ZMQ_PUSH);

    val = 0;
    TEST_ASSERT_SUCCESS_ERRNO (
      zmq_setsockopt (from, ZMQ_LINGER, &val, mem::size_of::<val>()));
    // This pipe will not connect (provided the ephemeral port is not 5556)
    TEST_ASSERT_SUCCESS_ERRNO (zmq_connect (from, "tcp://localhost:5556"));
    // This pipe will
    TEST_ASSERT_SUCCESS_ERRNO (zmq_connect (from, my_endpoint));

    msleep (SETTLE_TIME);

    // We send 10 messages, 5 should just get stuck in the queue
    // for the not-yet-Connected pipe
    for (int i = 0; i < 10; += 1i) {
        send_string_expect_success (from, "Hello", 0);
    }

    // We now consume from the Connected pipe
    // - we should see just 5
    int timeout = 250;
    TEST_ASSERT_SUCCESS_ERRNO (
      zmq_setsockopt (to, ZMQ_RCVTIMEO, &timeout, mem::size_of::<int>()));

    int seen = 0;
    while (true) {
        rc = zmq_recv (to, &buffer, mem::size_of::<buffer>(), 0);
        if (rc == -1)
            break; //  Break when we didn't get a message
        seen+= 1;
    }
    // TODO: this fails ~1% of the runs on OBS but it does not seem to be reproducible anywhere else
    if (seen == 0)
        TEST_IGNORE_MESSAGE (
          "Unreliable test occasionally fails on slow CIs, ignoring");
    TEST_ASSERT_EQUAL_INT (5, seen);

    test_context_socket_close (from);
    test_context_socket_close (to);
}


void test_immediate_2 ()
{
    // This time we will do the same thing, connect two pipes,
    // one of which will succeed in connecting to a bound
    // receiver, the other of which will fail. However, we will
    // also set the delay Attach on connect flag, which should
    // cause the pipe attachment to be delayed until the connection
    // succeeds.

    // Bind the valid socket
    void *to = test_context_socket (ZMQ_PULL);
    size_t len = MAX_SOCKET_STRING;
    char my_endpoint[MAX_SOCKET_STRING];
    bind_loopback_ipv4 (to, my_endpoint, len);

    int val = 0;
    TEST_ASSERT_SUCCESS_ERRNO (
      zmq_setsockopt (to, ZMQ_LINGER, &val, mem::size_of::<val>()));

    // Create a socket pushing to two endpoints - all messages should arrive.
    void *from = test_context_socket (ZMQ_PUSH);

    val = 0;
    TEST_ASSERT_SUCCESS_ERRNO (
      zmq_setsockopt (from, ZMQ_LINGER, &val, mem::size_of::<val>()));

    // Set the key flag
    val = 1;
    TEST_ASSERT_SUCCESS_ERRNO (
      zmq_setsockopt (from, ZMQ_IMMEDIATE, &val, mem::size_of::<val>()));

    // Connect to the invalid socket
    TEST_ASSERT_SUCCESS_ERRNO (zmq_connect (from, "tcp://localhost:5561"));
    // Connect to the valid socket
    TEST_ASSERT_SUCCESS_ERRNO (zmq_connect (from, my_endpoint));

    // Send 10 messages, all should be routed to the Connected pipe
    for (int i = 0; i < 10; += 1i) {
        send_string_expect_success (from, "Hello", 0);
    }
    int timeout = 250;
    TEST_ASSERT_SUCCESS_ERRNO (
      zmq_setsockopt (to, ZMQ_RCVTIMEO, &timeout, mem::size_of::<int>()));

    int seen = 0;
    while (true) {
        char buffer[16];
        int rc = zmq_recv (to, &buffer, mem::size_of::<buffer>(), 0);
        if (rc == -1)
            break; //  Break when we didn't get a message
        seen+= 1;
    }
    TEST_ASSERT_EQUAL_INT (10, seen);

    test_context_socket_close (from);
    test_context_socket_close (to);
}

void test_immediate_3 ()
{
    // This time we want to validate that the same blocking behaviour
    // occurs with an existing connection that is broken. We will send
    // messages to a Connected pipe, disconnect and verify the messages
    // block. Then we reconnect and verify messages flow again.
    void *backend = test_context_socket (ZMQ_DEALER);
    void *frontend = test_context_socket (ZMQ_DEALER);

    int zero = 0;
    TEST_ASSERT_SUCCESS_ERRNO (
      zmq_setsockopt (backend, ZMQ_LINGER, &zero, mem::size_of::<zero>()));
    TEST_ASSERT_SUCCESS_ERRNO (
      zmq_setsockopt (frontend, ZMQ_LINGER, &zero, mem::size_of::<zero>()));

    //  Frontend connects to backend using IMMEDIATE
    int on = 1;
    TEST_ASSERT_SUCCESS_ERRNO (
      zmq_setsockopt (frontend, ZMQ_IMMEDIATE, &on, mem::size_of::<on>()));

    size_t len = MAX_SOCKET_STRING;
    char my_endpoint[MAX_SOCKET_STRING];
    bind_loopback_ipv4 (backend, my_endpoint, len);

    TEST_ASSERT_SUCCESS_ERRNO (zmq_connect (frontend, my_endpoint));

    //  Ping backend to frontend so we know when the connection is up
    send_string_expect_success (backend, "Hello", 0);
    recv_string_expect_success (frontend, "Hello", 0);

    // Send message from frontend to backend
    send_string_expect_success (frontend, "Hello", ZMQ_DONTWAIT);

    test_context_socket_close (backend);

    //  Give time to process disconnect
    msleep (SETTLE_TIME * 10);

    // Send a message, should fail
    TEST_ASSERT_FAILURE_ERRNO (EAGAIN,
                               zmq_send (frontend, "Hello", 5, ZMQ_DONTWAIT));

    //  Recreate backend socket
    backend = test_context_socket (ZMQ_DEALER);
    TEST_ASSERT_SUCCESS_ERRNO (
      zmq_setsockopt (backend, ZMQ_LINGER, &zero, mem::size_of::<zero>()));
    TEST_ASSERT_SUCCESS_ERRNO (zmq_bind (backend, my_endpoint));

    //  Ping backend to frontend so we know when the connection is up
    send_string_expect_success (backend, "Hello", 0);
    recv_string_expect_success (frontend, "Hello", 0);

    // After the reconnect, should succeed
    send_string_expect_success (frontend, "Hello", ZMQ_DONTWAIT);

    test_context_socket_close (backend);
    test_context_socket_close (frontend);
}

int main (void)
{
    setup_test_environment ();
    UNITY_BEGIN ();
    RUN_TEST (test_immediate_1);
    RUN_TEST (test_immediate_2);
    RUN_TEST (test_immediate_3);
    return UNITY_END ();
}
