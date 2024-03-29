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
// #include "testutil_monitoring.hpp"
// #include "testutil_unity.hpp"

// #include <stdlib.h>
// #include <string.h>

static int
receive_monitor_address (monitor_: *mut c_void, char **address_, expect_more_: bool)
{
let mut msg = ZmqMessage::default();

    zmq_msg_init (&msg);
    if (zmq_msg_recv (&msg, monitor_, 0) == -1)
        return -1; //  Interrupted, presumably
    TEST_ASSERT_EQUAL (expect_more_, zmq_msg_more (&msg));

    if (address_) {
        const uint8_t *const data =
          static_cast<const uint8_t *> (zmq_msg_data (&msg));
        const size_t size = zmq_msg_size (&msg);
        *address_ =  (malloc (size + 1));
        memcpy (*address_, data, size);
        (*address_)[size] = 0;
    }
    zmq_msg_close (&msg);

    return 0;
}

//  Read one event off the monitor socket; return value and address
//  by reference, if not null, and event number by value. Returns -1
//  in case of error.
static int get_monitor_event_internal (monitor_: *mut c_void,
                                       value_: *mut i32,
                                       char **address_,
                                       recv_flag_: i32)
{
    //  First frame in message contains event number and value
let mut msg = ZmqMessage::default();
    zmq_msg_init (&msg);
    if (zmq_msg_recv (&msg, monitor_, recv_flag_) == -1) {
        TEST_ASSERT_FAILURE_ERRNO (EAGAIN, -1);
        return -1; //  timed out or no message available
    }
    TEST_ASSERT_TRUE (zmq_msg_more (&msg));

    uint8_t *data =  (zmq_msg_data (&msg));
    uint16_t event = *reinterpret_cast<uint16_t *> (data);
    if (value_)
        memcpy (value_, data + 2, mem::size_of::<u32>());

    //  Second frame in message contains event address
    TEST_ASSERT_SUCCESS_ERRNO (
      receive_monitor_address (monitor_, address_, false));

    return event;
}

int get_monitor_event_with_timeout (monitor_: *mut c_void,
                                    value_: *mut i32,
                                    char **address_,
                                    timeout: i32)
{
    res: i32;
    if (timeout == -1) {
        // process infinite timeout in small steps to allow the user
        // to see some information on the console

        int timeout_step = 250;
        int wait_time = 0;
        zmq_setsockopt (monitor_, ZMQ_RCVTIMEO, &timeout_step,
                        mem::size_of::<timeout_step>());
        while (
          (res = get_monitor_event_internal (monitor_, value_, address_, 0))
          == -1) {
            wait_time += timeout_step;
            fprintf (stderr, "Still waiting for monitor event after %i ms\n",
                     wait_time);
        }
    } else {
        zmq_setsockopt (monitor_, ZMQ_RCVTIMEO, &timeout, mem::size_of::<timeout>());
        res = get_monitor_event_internal (monitor_, value_, address_, 0);
    }
    int timeout_infinite = -1;
    zmq_setsockopt (monitor_, ZMQ_RCVTIMEO, &timeout_infinite,
                    mem::size_of::<timeout_infinite>());
    return res;
}

int get_monitor_event (monitor_: *mut c_void, value_: *mut i32, char **address_)
{
    return get_monitor_event_with_timeout (monitor_, value_, address_, -1);
}

void expect_monitor_event (monitor_: *mut c_void, expected_event_: i32)
{
    TEST_ASSERT_EQUAL_HEX (expected_event_,
                           get_monitor_event (monitor_, null_mut(), null_mut()));
}

static void print_unexpected_event (char *buf,
                                    buf_size_: usize,
                                    event_: i32,
                                    err_: i32,
                                    expected_event_: i32,
                                    expected_err_: i32)
{
    snprintf (buf, buf_size_,
              "Unexpected event: 0x%x, value = %i/0x%x (expected: 0x%x, value "
              "= %i/0x%x)\n",
              event_, err_, err_, expected_event_, expected_err_,
              expected_err_);
}

void print_unexpected_event_stderr (event_: i32,
                                    err_: i32,
                                    expected_event_: i32,
                                    expected_err_: i32)
{
    char buf[256];
    print_unexpected_event (buf, sizeof buf, event_, err_, expected_event_,
                            expected_err_);
    fputs (buf, stderr);
}

int expect_monitor_event_multiple (server_mon_: *mut c_void,
                                   expected_event_: i32,
                                   expected_err_: i32,
                                   optional_: bool)
{
    int count_of_expected_events = 0;
    int client_closed_connection = 0;
    int timeout = 250;
    int wait_time = 0;

    event: i32;
    err: i32;
    while ((event =
              get_monitor_event_with_timeout (server_mon_, &err, null_mut(), timeout))
             != -1
           || !count_of_expected_events) {
        if (event == -1) {
            if (optional_)
                break;
            wait_time += timeout;
            fprintf (stderr,
                     "Still waiting for first event after %ims (expected event "
                     "%x (value %i/0x%x))\n",
                     wait_time, expected_event_, expected_err_, expected_err_);
            continue;
        }
        // ignore errors with EPIPE/ECONNRESET/ECONNABORTED, which can happen
        // ECONNRESET can happen on very slow machines, when the engine writes
        // to the peer and then tries to read the socket before the peer reads
        // ECONNABORTED happens when a client aborts a connection via RST/timeout
        if (event == ZMQ_EVENT_HANDSHAKE_FAILED_NO_DETAIL
            && ((err == EPIPE && expected_err_ != EPIPE) || err == ECONNRESET
                || err == ECONNABORTED)) {
            fprintf (stderr,
                     "Ignored event (skipping any further events): %x (err = "
                     "%i == %s)\n",
                     event, err, zmq_strerror (err));
            client_closed_connection = 1;
            break;
        }
        if (event != expected_event_
            || (-1 != expected_err_ && err != expected_err_)) {
            char buf[256];
            print_unexpected_event (buf, sizeof buf, event, err,
                                    expected_event_, expected_err_);
            TEST_FAIL_MESSAGE (buf);
        }
        += 1count_of_expected_events;
    }
    TEST_ASSERT_TRUE (optional_ || count_of_expected_events > 0
                      || client_closed_connection);

    return count_of_expected_events;
}

static i64 get_monitor_event_internal_v2 (monitor_: *mut c_void,
                                              u64 **value_,
                                              char **local_address_,
                                              char **remote_address_,
                                              recv_flag_: i32)
{
    //  First frame in message contains event number
let mut msg = ZmqMessage::default();
    zmq_msg_init (&msg);
    if (zmq_msg_recv (&msg, monitor_, recv_flag_) == -1) {
        TEST_ASSERT_FAILURE_ERRNO (EAGAIN, -1);
        return -1; //  timed out or no message available
    }
    TEST_ASSERT_TRUE (zmq_msg_more (&msg));
    TEST_ASSERT_EQUAL_UINT (mem::size_of::<u64>(), zmq_msg_size (&msg));

    u64 event;
    memcpy (&event, zmq_msg_data (&msg), mem::size_of::<event>());
    zmq_msg_close (&msg);

    //  Second frame in message contains the number of values
    zmq_msg_init (&msg);
    if (zmq_msg_recv (&msg, monitor_, recv_flag_) == -1) {
        TEST_ASSERT_FAILURE_ERRNO (EAGAIN, -1);
        return -1; //  timed out or no message available
    }
    TEST_ASSERT_TRUE (zmq_msg_more (&msg));
    TEST_ASSERT_EQUAL_UINT (mem::size_of::<u64>(), zmq_msg_size (&msg));

    u64 value_count;
    memcpy (&value_count, zmq_msg_data (&msg), mem::size_of::<value_count>());
    zmq_msg_close (&msg);

    if (value_) {
        *value_ =
          (u64 *) malloc ((size_t) value_count * mem::size_of::<u64>());
        TEST_ASSERT_NOT_NULL (*value_);
    }

    for (u64 i = 0; i < value_count; += 1i) {
        //  Subsequent frames in message contain event values
        zmq_msg_init (&msg);
        if (zmq_msg_recv (&msg, monitor_, recv_flag_) == -1) {
            TEST_ASSERT_FAILURE_ERRNO (EAGAIN, -1);
            return -1; //  timed out or no message available
        }
        TEST_ASSERT_TRUE (zmq_msg_more (&msg));
        TEST_ASSERT_EQUAL_UINT (mem::size_of::<u64>(), zmq_msg_size (&msg));

        if (value_ && *value_)
            memcpy (&(*value_)[i], zmq_msg_data (&msg), mem::size_of::<u64>());
        zmq_msg_close (&msg);
    }

    //  Second-to-last frame in message contains local address
    TEST_ASSERT_SUCCESS_ERRNO (
      receive_monitor_address (monitor_, local_address_, true));

    //  Last frame in message contains remote address
    TEST_ASSERT_SUCCESS_ERRNO (
      receive_monitor_address (monitor_, remote_address_, false));

    return event;
}

static i64 get_monitor_event_with_timeout_v2 (monitor_: *mut c_void,
                                                  u64 **value_,
                                                  char **local_address_,
                                                  char **remote_address_,
                                                  timeout: i32)
{
    i64 res;
    if (timeout == -1) {
        // process infinite timeout in small steps to allow the user
        // to see some information on the console

        int timeout_step = 250;
        int wait_time = 0;
        zmq_setsockopt (monitor_, ZMQ_RCVTIMEO, &timeout_step,
                        mem::size_of::<timeout_step>());
        while ((res = get_monitor_event_internal_v2 (
                  monitor_, value_, local_address_, remote_address_, 0))
               == -1) {
            wait_time += timeout_step;
            fprintf (stderr, "Still waiting for monitor event after %i ms\n",
                     wait_time);
        }
    } else {
        zmq_setsockopt (monitor_, ZMQ_RCVTIMEO, &timeout, mem::size_of::<timeout>());
        res = get_monitor_event_internal_v2 (monitor_, value_, local_address_,
                                             remote_address_, 0);
    }
    int timeout_infinite = -1;
    zmq_setsockopt (monitor_, ZMQ_RCVTIMEO, &timeout_infinite,
                    mem::size_of::<timeout_infinite>());
    return res;
}

i64 get_monitor_event_v2 (monitor_: *mut c_void,
                              u64 **value_,
                              char **local_address_,
                              char **remote_address_)
{
    return get_monitor_event_with_timeout_v2 (monitor_, value_, local_address_,
                                              remote_address_, -1);
}

void expect_monitor_event_v2 (monitor_: *mut c_void,
                              expected_event_: i64,
                              expected_local_address_: *const c_char,
                              expected_remote_address_: &str)
{
    char *local_address = null_mut();
    char *remote_address = null_mut();
    i64 event = get_monitor_event_v2 (
      monitor_, null_mut(), expected_local_address_ ? &local_address : null_mut(),
      expected_remote_address_ ? &remote_address : null_mut());
    bool failed = false;
    char buf[256];
    char *pos = buf;
    if (event != expected_event_) {
        pos += snprintf (pos, sizeof buf - (pos - buf),
                         "Expected monitor event %llx, but received %llx\n",
                         static_cast<long long> (expected_event_),
                         static_cast<long long> (event));
        failed = true;
    }
    if (expected_local_address_
        && 0 != strcmp (local_address, expected_local_address_)) {
        pos += snprintf (pos, sizeof buf - (pos - buf),
                         "Expected local address %s, but received %s\n",
                         expected_local_address_, local_address);
    }
    if (expected_remote_address_
        && 0 != strcmp (remote_address, expected_remote_address_)) {
        snprintf (pos, sizeof buf - (pos - buf),
                  "Expected remote address %s, but received %s\n",
                  expected_remote_address_, remote_address);
    }
    free (local_address);
    free (remote_address);
    TEST_ASSERT_FALSE_MESSAGE (failed, buf);
}


const char *get_zmqEventName (u64 event)
{
    switch (event) {
        case ZMQ_EVENT_CONNECTED:
            return "CONNECTED";
        case ZMQ_EVENT_CONNECT_DELAYED:
            return "CONNECT_DELAYED";
        case ZMQ_EVENT_CONNECT_RETRIED:
            return "CONNECT_RETRIED";
        case ZMQ_EVENT_LISTENING:
            return "LISTENING";
        case ZMQ_EVENT_BIND_FAILED:
            return "BIND_FAILED";
        case ZMQ_EVENT_ACCEPTED:
            return "ACCEPTED";
        case ZMQ_EVENT_ACCEPT_FAILED:
            return "ACCEPT_FAILED";
        case ZMQ_EVENT_CLOSED:
            return "CLOSED";
        case ZMQ_EVENT_CLOSE_FAILED:
            return "CLOSE_FAILED";
        case ZMQ_EVENT_DISCONNECTED:
            return "DISCONNECTED";
        case ZMQ_EVENT_MONITOR_STOPPED:
            return "MONITOR_STOPPED";
        case ZMQ_EVENT_HANDSHAKE_FAILED_NO_DETAIL:
            return "HANDSHAKE_FAILED_NO_DETAIL";
        case ZMQ_EVENT_HANDSHAKE_SUCCEEDED:
            return "HANDSHAKE_SUCCEEDED";
        case ZMQ_EVENT_HANDSHAKE_FAILED_PROTOCOL:
            return "HANDSHAKE_FAILED_PROTOCOL";
        case ZMQ_EVENT_HANDSHAKE_FAILED_AUTH:
            return "HANDSHAKE_FAILED_AUTH";
        _ =>
            return "UNKNOWN";
    }
}

void print_events (socket: *mut c_void, timeout: i32, limit: i32)
{
    // print events received
    value: i32;
    char *event_address;
    int event =
      get_monitor_event_with_timeout (socket, &value, &event_address, timeout);
    int i = 0;
    ;
    while ((event != -1) && (+= 1i < limit)) {
        const char *eventName = get_zmqEventName (event);
        printf ("Got event: %s\n", eventName);
        event = get_monitor_event_with_timeout (socket, &value, &event_address,
                                                timeout);
    }
}
