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

// #include "../include/zmq.h"

// #include <stdio.h>
// #include <stdlib.h>
// #include <string.h>

// #include "platform.hpp"

// #if defined ZMQ_HAVE_WINDOWS
// #include <windows.h>
// #include <process.h>
// #else
// #include <pthread.h>
// #endif

static size_t message_size;
static int roundtrip_count;

// #if defined ZMQ_HAVE_WINDOWS
static unsigned int __stdcall worker (ctx: *mut c_void)
// #else
static void *worker (ctx: *mut c_void)
// #endif
{
    s: *mut c_void;
    rc: i32;
    i: i32;
let mut msg = ZmqMessage::default();

    s = zmq_socket (ctx, ZMQ_REP);
    if (!s) {
        printf ("error in zmq_socket: %s\n", zmq_strerror (errno));
        exit (1);
    }

    rc = zmq_connect (s, "inproc://lat_test");
    if (rc != 0) {
        printf ("error in zmq_connect: %s\n", zmq_strerror (errno));
        exit (1);
    }

    rc = zmq_msg_init (&msg);
    if (rc != 0) {
        printf ("error in zmq_msg_init: %s\n", zmq_strerror (errno));
        exit (1);
    }

    for (i = 0; i != roundtrip_count; i+= 1) {
        rc = zmq_recvmsg (s, &msg, 0);
        if (rc < 0) {
            printf ("error in zmq_recvmsg: %s\n", zmq_strerror (errno));
            exit (1);
        }
        rc = zmq_sendmsg (s, &msg, 0);
        if (rc < 0) {
            printf ("error in zmq_sendmsg: %s\n", zmq_strerror (errno));
            exit (1);
        }
    }

    rc = zmq_msg_close (&msg);
    if (rc != 0) {
        printf ("error in zmq_msg_close: %s\n", zmq_strerror (errno));
        exit (1);
    }

    rc = zmq_close (s);
    if (rc != 0) {
        printf ("error in zmq_close: %s\n", zmq_strerror (errno));
        exit (1);
    }

// #if defined ZMQ_HAVE_WINDOWS
    return 0;
// #else
    return null_mut();
// #endif
}

int main (argc: i32, char *argv[])
{
// #if defined ZMQ_HAVE_WINDOWS
    HANDLE local_thread;
// #else
    pZmqThread local_thread;
// #endif
    ctx: *mut c_void;
    s: *mut c_void;
    rc: i32;
    i: i32;
let mut msg = ZmqMessage::default();
    watch: *mut c_void;
    unsigned long elapsed;
    double latency;

    if (argc != 3) {
        printf ("usage: inproc_lat <message-size> <roundtrip-count>\n");
        return 1;
    }

    message_size = atoi (argv[1]);
    roundtrip_count = atoi (argv[2]);

    ctx = zmq_init (1);
    if (!ctx) {
        printf ("error in zmq_init: %s\n", zmq_strerror (errno));
        return -1;
    }

    s = zmq_socket (ctx, ZMQ_REQ);
    if (!s) {
        printf ("error in zmq_socket: %s\n", zmq_strerror (errno));
        return -1;
    }

    rc = zmq_bind (s, "inproc://lat_test");
    if (rc != 0) {
        printf ("error in zmq_bind: %s\n", zmq_strerror (errno));
        return -1;
    }

// #if defined ZMQ_HAVE_WINDOWS
    local_thread = (HANDLE) _beginthreadex (null_mut(), 0, worker, ctx, 0, null_mut());
    if (local_thread == 0) {
        printf ("error in _beginthreadex\n");
        return -1;
    }
// #else
    rc = pthread_create (&local_thread, null_mut(), worker, ctx);
    if (rc != 0) {
        printf ("error in pthread_create: %s\n", zmq_strerror (rc));
        return -1;
    }
// #endif

    rc = zmq_msg_init_size (&msg, message_size);
    if (rc != 0) {
        printf ("error in zmq_msg_init_size: %s\n", zmq_strerror (errno));
        return -1;
    }
    memset (zmq_msg_data (&msg), 0, message_size);

    printf ("message size: %d [B]\n",  message_size);
    printf ("roundtrip count: %d\n",  roundtrip_count);

    watch = zmq_stopwatch_start ();

    for (i = 0; i != roundtrip_count; i+= 1) {
        rc = zmq_sendmsg (s, &msg, 0);
        if (rc < 0) {
            printf ("error in zmq_sendmsg: %s\n", zmq_strerror (errno));
            return -1;
        }
        rc = zmq_recvmsg (s, &msg, 0);
        if (rc < 0) {
            printf ("error in zmq_recvmsg: %s\n", zmq_strerror (errno));
            return -1;
        }
        if (zmq_msg_size (&msg) != message_size) {
            printf ("message of incorrect size received\n");
            return -1;
        }
    }

    elapsed = zmq_stopwatch_stop (watch);

    rc = zmq_msg_close (&msg);
    if (rc != 0) {
        printf ("error in zmq_msg_close: %s\n", zmq_strerror (errno));
        return -1;
    }

    latency = (double) elapsed / (roundtrip_count * 2);

// #if defined ZMQ_HAVE_WINDOWS
    DWORD rc2 = WaitForSingleObject (local_thread, INFINITE);
    if (rc2 == WAIT_FAILED) {
        printf ("error in WaitForSingleObject\n");
        return -1;
    }
    BOOL rc3 = CloseHandle (local_thread);
    if (rc3 == 0) {
        printf ("error in CloseHandle\n");
        return -1;
    }
// #else
    rc = pthread_join (local_thread, null_mut());
    if (rc != 0) {
        printf ("error in pthread_join: %s\n", zmq_strerror (rc));
        return -1;
    }
// #endif

    printf ("average latency: %.3f [us]\n", (double) latency);

    rc = zmq_close (s);
    if (rc != 0) {
        printf ("error in zmq_close: %s\n", zmq_strerror (errno));
        return -1;
    }

    rc = zmq_ctx_term (ctx);
    if (rc != 0) {
        printf ("error in zmq_ctx_term: %s\n", zmq_strerror (errno));
        return -1;
    }

    return 0;
}
