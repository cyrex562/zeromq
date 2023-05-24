/*
    Copyright (c) 2016-2017 Contributors as noted in the AUTHORS file

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

// #include <stdlib.h>
// #include <string.h>
// #include <unistd.h>

SETUP_TEARDOWN_TESTCONTEXT

// #if !defined(ZMQ_HAVE_WINDOWS)

void pre_allocate_sock_tcp (socket: *mut c_void, char *my_endpoint_)
{
    ZmqFileDesc s = bind_socket_resolve_port ("127.0.0.1", "0", my_endpoint_);
    TEST_ASSERT_SUCCESS_ERRNO (
      zmq_setsockopt (socket, ZMQ_USE_FD, &s, mem::size_of::<s>()));
}

typedef void (*pre_allocate_sock_fun_t) (void *, char *);

void setup_socket_pair (pre_allocate_sock_fun_t pre_allocate_sock_fun_,
                        bind_socket_type_: i32,
                        connect_socket_type_: i32,
                        out_sb_: *mut *mut c_void
                        void **out_sc_)
{
    *out_sb_ = test_context_socket (bind_socket_type_);

    char my_endpoint[MAX_SOCKET_STRING];
    pre_allocate_sock_fun_ (*out_sb_, my_endpoint);

    TEST_ASSERT_SUCCESS_ERRNO (zmq_bind (*out_sb_, my_endpoint));

    *out_sc_ = test_context_socket (connect_socket_type_);
    TEST_ASSERT_SUCCESS_ERRNO (zmq_connect (*out_sc_, my_endpoint));
}

void test_socket_pair (pre_allocate_sock_fun_t pre_allocate_sock_fun_,
                       bind_socket_type_: i32,
                       connect_socket_type_: i32)
{
    sb: *mut c_void, *sc;
    setup_socket_pair (pre_allocate_sock_fun_, bind_socket_type_,
                       connect_socket_type_, &sb, &sc);

    bounce (sb, sc);

    test_context_socket_close (sc);
    test_context_socket_close (sb);
}

void test_req_rep (pre_allocate_sock_fun_t pre_allocate_sock_fun_)
{
    test_socket_pair (pre_allocate_sock_fun_, ZMQ_REP, ZMQ_REQ);
}

void test_pair (pre_allocate_sock_fun_t pre_allocate_sock_fun_)
{
    test_socket_pair (pre_allocate_sock_fun_, ZMQ_PAIR, ZMQ_PAIR);
}

void test_client_server (pre_allocate_sock_fun_t pre_allocate_sock_fun_)
{
// #if defined(ZMQ_SERVER) && defined(ZMQ_CLIENT)
    sb: *mut c_void, *sc;
    setup_socket_pair (pre_allocate_sock_fun_, ZMQ_SERVER, ZMQ_CLIENT, &sb,
                       &sc);
let mut msg = ZmqMessage::default();
    TEST_ASSERT_SUCCESS_ERRNO (zmq_msg_init_size (&msg, 1));

    char *data =  (zmq_msg_data (&msg));
    data[0] = 1;

    int rc = zmq_msg_send (&msg, sc, ZMQ_SNDMORE);
    // TODO which error code is expected?
    TEST_ASSERT_EQUAL_INT (-1, rc);

    rc = zmq_msg_send (&msg, sc, 0);
    TEST_ASSERT_EQUAL_INT (1, rc);

    TEST_ASSERT_SUCCESS_ERRNO (zmq_msg_init (&msg));

    rc = zmq_msg_recv (&msg, sb, 0);
    TEST_ASSERT_EQUAL_INT (1, rc);

    u32 routing_id = zmq_msg_routing_id (&msg);
    TEST_ASSERT_NOT_EQUAL (0, routing_id);

    TEST_ASSERT_SUCCESS_ERRNO (zmq_msg_close (&msg));

    TEST_ASSERT_SUCCESS_ERRNO (zmq_msg_init_size (&msg, 1));

    data =  (zmq_msg_data (&msg));
    data[0] = 2;

    TEST_ASSERT_SUCCESS_ERRNO (zmq_msg_set_routing_id (&msg, routing_id));

    rc = zmq_msg_send (&msg, sb, ZMQ_SNDMORE);
    // TODO which error code is expected?
    TEST_ASSERT_EQUAL_INT (-1, rc);

    rc = zmq_msg_send (&msg, sb, 0);
    TEST_ASSERT_EQUAL_INT (1, rc);

    rc = zmq_msg_recv (&msg, sc, 0);
    TEST_ASSERT_EQUAL_INT (1, rc);

    routing_id = zmq_msg_routing_id (&msg);
    TEST_ASSERT_EQUAL_INT (0, routing_id);

    TEST_ASSERT_SUCCESS_ERRNO (zmq_msg_close (&msg));

    test_context_socket_close (sc);
    test_context_socket_close (sb);
// #endif
}

void test_req_rep_tcp ()
{
    test_req_rep (pre_allocate_sock_tcp);
}

void test_ZmqPaircp ()
{
    test_pair (pre_allocate_sock_tcp);
}

void test_client_ZmqServercp ()
{
// #if defined(ZMQ_SERVER) && defined(ZMQ_CLIENT)
    test_client_server (pre_allocate_sock_tcp);
// #endif
}

char ipc_endpoint[MAX_SOCKET_STRING] = "";

void pre_allocate_sock_ipc (sb_: *mut c_void, char *my_endpoint_)
{
    ZmqFileDesc s = bind_socket_resolve_port ("", "", my_endpoint_, AF_UNIX, 0);
    TEST_ASSERT_SUCCESS_ERRNO (
      zmq_setsockopt (sb_, ZMQ_USE_FD, &s, mem::size_of::<s>()));
    strcpy (ipc_endpoint, strchr (my_endpoint_, '/') + 2);
}

void test_req_rep_ipc ()
{
    test_req_rep (pre_allocate_sock_ipc);

    TEST_ASSERT_SUCCESS_ERRNO (unlink (ipc_endpoint));
}

void test_pair_ipc ()
{
    test_pair (pre_allocate_sock_ipc);

    TEST_ASSERT_SUCCESS_ERRNO (unlink (ipc_endpoint));
}

void test_client_server_ipc ()
{
// #if defined(ZMQ_SERVER) && defined(ZMQ_CLIENT)
    test_client_server (pre_allocate_sock_ipc);

    TEST_ASSERT_SUCCESS_ERRNO (unlink (ipc_endpoint));
// #endif
}

int main ()
{
    setup_test_environment ();

    UNITY_BEGIN ();
    RUN_TEST (test_req_rep_tcp);
    RUN_TEST (test_ZmqPaircp);
    RUN_TEST (test_client_ZmqServercp);

    RUN_TEST (test_req_rep_ipc);
    RUN_TEST (test_pair_ipc);
    RUN_TEST (test_client_server_ipc);

    return UNITY_END ();
}
// #else
int main ()
{
    return 0;
}
// #endif
