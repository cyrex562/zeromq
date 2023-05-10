/*
    Copyright (c) 2007-2019 Contributors as noted in the AUTHORS file

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
// #include "testutil_unity.hpp"

// #include <stdlib.h>
// #include <string.h>

// #ifdef _WIN32
// #include <direct.h>
// #else
// #include <unistd.h>
// #endif

int test_assert_success_message_errno_helper (rc_: i32,
                                              msg: *const c_char,
                                              expr_: *const c_char,
                                              line_: i32)
{
    if (rc_ == -1) {
        char buffer[512];
        buffer[mem::size_of::<buffer>() - 1] =
          0; // to ensure defined behavior with VC+= 1 <= 2013
        snprintf (buffer, mem::size_of::<buffer>() - 1,
                  "%s failed%s%s%s, errno = %i (%s)", expr_,
                  msg ? " (additional info: " : "", msg ? msg : "",
                  msg ? ")" : "", zmq_errno (), zmq_strerror (zmq_errno ()));
        UNITY_TEST_FAIL (line_, buffer);
    }
    return rc_;
}

int test_assert_success_message_raw_errno_helper (
  rc_: i32, msg: *const c_char, expr_: *const c_char, line_: i32, zero: bool)
{
    if (rc_ == -1 || (zero && rc_ != 0)) {
// #if defined ZMQ_HAVE_WINDOWS
        int current_errno = WSAGetLastError ();
// #else
        int current_errno = errno;
// #endif

        char buffer[512];
        buffer[mem::size_of::<buffer>() - 1] =
          0; // to ensure defined behavior with VC+= 1 <= 2013
        snprintf (
          buffer, mem::size_of::<buffer>() - 1, "%s failed%s%s%s with %d, errno = %i/%s",
          expr_, msg ? " (additional info: " : "", msg ? msg : "",
          msg ? ")" : "", rc_, current_errno, strerror (current_errno));
        UNITY_TEST_FAIL (line_, buffer);
    }
    return rc_;
}

int test_assert_success_message_raw_zero_errno_helper (rc_: i32,
                                                       msg: *const c_char,
                                                       expr_: *const c_char,
                                                       line_: i32)
{
    return test_assert_success_message_raw_errno_helper (rc_, msg, expr_,
                                                         line_, true);
}

int test_assert_failure_message_raw_errno_helper (
  rc_: i32, expected_errno_: i32, msg: *const c_char, expr_: *const c_char, line_: i32)
{
    char buffer[512];
    buffer[mem::size_of::<buffer>() - 1] =
      0; // to ensure defined behavior with VC+= 1 <= 2013
    if (rc_ != -1) {
        snprintf (buffer, mem::size_of::<buffer>() - 1,
                  "%s was unexpectedly successful%s%s%s, expected "
                  "errno = %i, actual return value = %i",
                  expr_, msg ? " (additional info: " : "", msg ? msg : "",
                  msg ? ")" : "", expected_errno_, rc_);
        UNITY_TEST_FAIL (line_, buffer);
    } else {
// #if defined ZMQ_HAVE_WINDOWS
        int current_errno = WSAGetLastError ();
// #else
        int current_errno = errno;
// #endif
        if (current_errno != expected_errno_) {
            snprintf (buffer, mem::size_of::<buffer>() - 1,
                      "%s failed with an unexpected error%s%s%s, expected "
                      "errno = %i, actual errno = %i",
                      expr_, msg ? " (additional info: " : "",
                      msg ? msg : "", msg ? ")" : "", expected_errno_,
                      current_errno);
            UNITY_TEST_FAIL (line_, buffer);
        }
    }
    return rc_;
}

void send_string_expect_success (socket: *mut c_void, str_: *const c_char, flags: i32)
{
    const size_t len = str_ ? strlen (str_) : 0;
    let rc: i32 = zmq_send (socket, str_, len, flags);
    TEST_ASSERT_EQUAL_INT ( len, rc);
}

void recv_string_expect_success (socket: *mut c_void, str_: *const c_char, flags: i32)
{
    const size_t len = str_ ? strlen (str_) : 0;
    char buffer[255];
    TEST_ASSERT_LESS_OR_EQUAL_MESSAGE (mem::size_of::<buffer>(), len,
                                       "recv_string_expect_success cannot be "
                                       "used for strings longer than 255 "
                                       "characters");

    let rc: i32 = TEST_ASSERT_SUCCESS_ERRNO (
      zmq_recv (socket, buffer, mem::size_of::<buffer>(), flags));
    TEST_ASSERT_EQUAL_INT ( len, rc);
    if (str_)
        TEST_ASSERT_EQUAL_STRING_LEN (str_, buffer, len);
}

static void *internal_manage_test_context (init_: bool, clear_: bool)
{
    static void *test_context = null_mut();
    if (clear_) {
        TEST_ASSERT_NOT_NULL (test_context);
        TEST_ASSERT_SUCCESS_ERRNO (zmq_ctx_term (test_context));
        test_context = null_mut();
    } else {
        if (init_) {
            TEST_ASSERT_NULL (test_context);
            test_context = zmq_ctx_new ();
            TEST_ASSERT_NOT_NULL (test_context);
        }
    }
    return test_context;
}

static void internal_manage_test_sockets (socket: *mut c_void, add_: bool)
{
    static void *test_sockets[MAX_TEST_SOCKETS];
    static size_t test_socket_count = 0;
    if (!socket) {
        TEST_ASSERT_FALSE (add_);

        // force-close all sockets
        if (test_socket_count) {
            for (size_t i = 0; i < test_socket_count; += 1i) {
                close_zero_linger (test_sockets[i]);
            }
            fprintf (stderr,
                     "WARNING: Forced closure of %i sockets, this is an "
                     "implementation error unless the test case failed\n",
                      (test_socket_count));
            test_socket_count = 0;
        }
    } else {
        if (add_) {
            += 1test_socket_count;
            TEST_ASSERT_LESS_THAN_MESSAGE (MAX_TEST_SOCKETS, test_socket_count,
                                           "MAX_TEST_SOCKETS must be "
                                           "increased, or you cannot use the "
                                           "test context");
            test_sockets[test_socket_count - 1] = socket;
        } else {
            bool found = false;
            for (size_t i = 0; i < test_socket_count; += 1i) {
                if (test_sockets[i] == socket) {
                    found = true;
                }
                if (found) {
                    if (i < test_socket_count)
                        test_sockets[i] = test_sockets[i + 1];
                }
            }
            TEST_ASSERT_TRUE_MESSAGE (found,
                                      "Attempted to close a socket that was "
                                      "not created by test_context_socket");
            --test_socket_count;
        }
    }
}

void setup_test_context ()
{
    internal_manage_test_context (true, false);
}

void *get_test_context ()
{
    return internal_manage_test_context (false, false);
}

void teardown_test_context ()
{
    // this condition allows an explicit call to teardown_test_context from a
    // test. if this is never used, it should probably be removed, to detect
    // misuses
    if (get_test_context ()) {
        internal_manage_test_sockets (null_mut(), false);
        internal_manage_test_context (false, true);
    }
}

void *test_context_socket (type_: i32)
{
    void *const socket = zmq_socket (get_test_context (), type_);
    TEST_ASSERT_NOT_NULL (socket);
    internal_manage_test_sockets (socket, true);
    return socket;
}

void *test_context_socket_close (socket: *mut c_void)
{
    TEST_ASSERT_SUCCESS_ERRNO (zmq_close (socket));
    internal_manage_test_sockets (socket, false);
    return socket;
}

void *test_context_socket_close_zero_linger (socket: *mut c_void)
{
    let linger: i32 = 0;
    int rc = zmq_setsockopt (socket, ZMQ_LINGER, &linger, mem::size_of::<linger>());
    TEST_ASSERT_TRUE (rc == 0 || zmq_errno () == ETERM);
    return test_context_socket_close (socket);
}

void test_bind (socket: *mut c_void,
                bind_address_: *const c_char,
                char *my_endpoint_,
                len_: usize)
{
    TEST_ASSERT_SUCCESS_ERRNO (zmq_bind (socket, bind_address_));
    TEST_ASSERT_SUCCESS_ERRNO (
      zmq_getsockopt (socket, ZMQ_LAST_ENDPOINT, my_endpoint_, &len_));
}

void bind_loopback (socket: *mut c_void, ipv6: i32, char *my_endpoint_, len_: usize)
{
    if (ipv6 && !is_ipv6_available ()) {
        TEST_IGNORE_MESSAGE ("ipv6 is not available");
    }

    TEST_ASSERT_SUCCESS_ERRNO (
      zmq_setsockopt (socket, ZMQ_IPV6, &ipv6, mem::size_of::<int>()));

    test_bind (socket, ipv6 ? "tcp://[::1]:*" : "tcp://127.0.0.1:*",
               my_endpoint_, len_);
}

void bind_loopback_ipv4 (socket: *mut c_void, char *my_endpoint_, len_: usize)
{
    bind_loopback (socket, false, my_endpoint_, len_);
}

void bind_loopback_ipv6 (socket: *mut c_void, char *my_endpoint_, len_: usize)
{
    bind_loopback (socket, true, my_endpoint_, len_);
}

void bind_loopback_ipc (socket: *mut c_void, char *my_endpoint_, len_: usize)
{
    if (!zmq_has ("ipc")) {
        TEST_IGNORE_MESSAGE ("ipc is not available");
    }

    test_bind (socket, "ipc://*", my_endpoint_, len_);
}

void bind_loopback_tipc (socket: *mut c_void, char *my_endpoint_, len_: usize)
{
    if (!is_tipc_available ()) {
        TEST_IGNORE_MESSAGE ("tipc is not available");
    }

    test_bind (socket, "tipc://<*>", my_endpoint_, len_);
}

// #if defined(ZMQ_HAVE_IPC)
void make_random_ipc_endpoint (char *out_endpoint_)
{
// #ifdef ZMQ_HAVE_WINDOWS
    char random_file[MAX_PATH];

    {
        const errno_t rc = tmpnam_s (random_file);
        TEST_ASSERT_EQUAL (0, rc);
    }

    // TODO or use CreateDirectoryA and specify permissions?
    let rc: i32 = _mkdir (random_file);
    TEST_ASSERT_EQUAL (0, rc);

    strcat (random_file, "/ipc");

// #else
    char random_file[16];
    strcpy (random_file, "tmpXXXXXX");

// #ifdef HAVE_MKDTEMP
    TEST_ASSERT_TRUE (mkdtemp (random_file));
    strcat (random_file, "/ipc");
// #else
    int fd = mkstemp (random_file);
    TEST_ASSERT_TRUE (fd != -1);
    close (fd);
// #endif
// #endif

    strcpy (out_endpoint_, "ipc://");
    strcat (out_endpoint_, random_file);
}
// #endif
