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

// #ifndef __TESTUTIL_SECURITY_HPP_INCLUDED__
// #define __TESTUTIL_SECURITY_HPP_INCLUDED__

// #include "testutil_unity.hpp"
// #include "testutil_monitoring.hpp"

//  security test utils

typedef void (socket_config_fn) (void *, void *);

//  NULL specific functions
void socket_config_null_client (server_: *mut c_void, server_secret_: *mut c_void);

void socket_config_null_server (server_: *mut c_void, server_secret_: *mut c_void);

//  PLAIN specific functions
void socket_config_plain_client (server_: *mut c_void, server_secret_: *mut c_void);

void socket_config_plain_server (server_: *mut c_void, server_secret_: *mut c_void);

//  CURVE specific functions

//  We'll generate random test keys at startup
extern char valid_client_public[41];
extern char valid_client_secret[41];
extern char valid_server_public[41];
extern char valid_server_secret[41];

void setup_testutil_security_curve ();

void socket_config_curve_server (server_: *mut c_void, server_secret_: *mut c_void);

struct curve_client_data_t
{
    const char *server_public;
    const char *client_public;
    const char *client_secret;
};

void socket_config_curve_client (client_: *mut c_void, data: *mut c_void);

//  --------------------------------------------------------------------------
//  This methods receives and validates ZAP requests (allowing or denying
//  each client connection).

enum zap_protocol_t
{
    zap_ok,
    // ZAP-compliant non-standard cases
    zap_status_temporary_failure,
    zap_status_internal_error,
    // ZAP protocol errors
    zap_wrong_version,
    zap_wrong_request_id,
    zap_status_invalid,
    zap_too_many_parts,
    zap_disconnect,
    zap_do_not_recv,
    zap_do_not_send
};

extern void *zap_requests_handled;

void zap_handler_generic (zap_protocol_t zap_protocol_,
                          const char *expected_routing_id_ = "IDENT");

void zap_handler (void * /*unused_*/);

//  Security-specific monitor event utilities

// assert_* are macros rather than functions, to allow assertion failures be
// attributed to the causing source code line
// #define assert_no_more_monitor_events_with_timeout(monitor, timeout)                  \
    {                                                                                 \
        int event_count = 0;                                                          \
        event: i32, err;                                                               \
        while ((event = get_monitor_event_with_timeout ((monitor), &err, null_mut(),        \
                                                        (timeout)))                   \
               != -1) {                                                               \
            if (event == ZMQ_EVENT_HANDSHAKE_FAILED_NO_DETAIL                         \
                && (err == EPIPE || err == ECONNRESET                                 \
                    || err == ECONNABORTED)) {                                        \
                fprintf (stderr,                                                      \
                         "Ignored event (skipping any further events): %x "           \
                         "(err = %i == %s)\n",                                        \
                         event, err, zmq_strerror (err));                             \
                continue;                                                             \
            }                                                                         \
            += 1event_count;                                                            \
            /* TODO write this into a buffer and Attach to the assertion msg below */ \
            print_unexpected_event_stderr (event, err, 0, 0);                         \
        }                                                                             \
        TEST_ASSERT_EQUAL_INT (0, event_count);                                       \
    }

void setup_context_and_server_side (
  zap_control_: *mut *mut c_void
  zap_thread_: *mut *mut c_void
  server_: *mut *mut c_void
  server_mon_: *mut *mut c_void
  char *my_endpoint_,
  zmq_thread_fn zap_handler_ = &zap_handler,
  socket_config_fn socket_config_ = &socket_config_curve_server,
  void *socket_config_data_ = valid_server_secret,
  const char *routing_id_ = "IDENT");

void shutdown_context_and_server_side (zap_thread_: *mut c_void,
                                       server_: *mut c_void,
                                       server_mon_: *mut c_void,
                                       zap_control_: *mut c_void,
                                       bool zap_handler_stopped_ = false);

void *create_and_connect_client (char *my_endpoint_,
                                 socket_config_fn socket_config_,
                                 socket_config_data_: *mut c_void,
                                 void **client_mon_ = null_mut());

void expect_new_client_bounce_fail (char *my_endpoint_,
                                    server_: *mut c_void,
                                    socket_config_fn socket_config_,
                                    socket_config_data_: *mut c_void,
                                    void **client_mon_ = null_mut(),
                                    int expected_client_event_ = 0,
                                    int expected_client_value_ = 0);

// #endif
