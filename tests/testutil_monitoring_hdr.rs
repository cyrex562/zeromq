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

// #ifndef __TESTUTIL_MONITORING_HPP_INCLUDED__
// #define __TESTUTIL_MONITORING_HPP_INCLUDED__

// #include "../include/zmq.h"
// #include "../src/stdint.hpp"

// #include <stddef.h>

//  General, i.e. non-security specific, monitor utilities

int get_monitor_event_with_timeout (monitor_: *mut c_void,
                                    value_: *mut i32,
                                    char **address_,
                                    timeout: i32);

//  Read one event off the monitor socket; return value and address
//  by reference, if not null, and event number by value. Returns -1
//  in case of error.
int get_monitor_event (monitor_: *mut c_void, value_: *mut i32, char **address_);

void expect_monitor_event (monitor_: *mut c_void, expected_event_: i32);

void print_unexpected_event_stderr (event_: i32,
                                    err_: i32,
                                    expected_event_: i32,
                                    expected_err_: i32);

//  expects that one or more occurrences of the expected event are received
//  via the specified socket monitor
//  returns the number of occurrences of the expected event
//  interrupts, if a ZMQ_EVENT_HANDSHAKE_FAILED_NO_DETAIL with EPIPE, ECONNRESET
//  or ECONNABORTED occurs; in this case, 0 is returned
//  this should be investigated further, see
//  https://github.com/zeromq/libzmq/issues/2644
int expect_monitor_event_multiple (server_mon_: *mut c_void,
                                   expected_event_: i32,
                                   int expected_err_ = -1,
                                   bool optional_ = false);

i64 get_monitor_event_v2 (monitor_: *mut c_void,
                              u64 **value_,
                              char **local_address_,
                              char **remote_address_);

void expect_monitor_event_v2 (monitor_: *mut c_void,
                              expected_event_: i64,
                              const char *expected_local_address_ = null_mut(),
                              const char *expected_remote_address_ = null_mut());


const char *get_zmqEventName (u64 event);
void print_events (socket: *mut c_void, timeout: i32, limit: i32);

// #endif
