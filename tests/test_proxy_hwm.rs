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
// #include <string.h>
// #include <unity.h>
// #include <assert.h>
// #include <unistd.h>

//
// Asynchronous proxy test using ZMQ_XPUB_NODROP and HWM:
//
// Topology:
//
//   XPUB                      SUB
//    |                         |
//    \-----> XSUB -> XPUB -----/
//           ^^^^^^^^^^^^^^
//             ZMQ proxy
//
// All connections use "inproc" transport and have artificially-low HWMs set.
// Then the PUB socket starts flooding the Proxy. The SUB is artificially slow
// at receiving messages.
// This scenario simulates what happens when a SUB is slower than
// its (X)PUB: since ZMQ_XPUB_NODROP=1, the XPUB will block and then
// also the (X)PUB socket will block.
// The exact number of the messages that go through before (X)PUB blocks depends
// on ZeroMQ internals and how the OS will schedule the different threads.
// In the meanwhile asking statistics to the Proxy must NOT be blocking.
//


// #define HWM 10
// #define NUM_BYTES_PER_MSG 50000


typedef struct
{
    context: *mut c_void;
    const char *frontend_endpoint;
    const char *backend_endpoint;
    const char *control_endpoint;

    subscriber_received_all: *mut c_void;
} proxy_hwm_cfg_t;

static void lower_hwm (skt_: *mut c_void)
{
    int send_hwm = HWM;
    TEST_ASSERT_SUCCESS_ERRNO (
      zmq_setsockopt (skt_, ZMQ_SNDHWM, &send_hwm, mem::size_of::<send_hwm>()));

    TEST_ASSERT_SUCCESS_ERRNO (
      zmq_setsockopt (skt_, ZMQ_RCVHWM, &send_hwm, mem::size_of::<send_hwm>()));
}

static void publisher_thread_main (pvoid_: *mut c_void)
{
    const proxy_hwm_cfg_t *const cfg =
      static_cast<const proxy_hwm_cfg_t *> (pvoid_);

    void *pubsocket = zmq_socket (cfg.context, ZMQ_XPUB);
    assert (pubsocket);

    lower_hwm (pubsocket);

    TEST_ASSERT_SUCCESS_ERRNO (zmq_connect (pubsocket, cfg.frontend_endpoint));

    int optval = 1;
    TEST_ASSERT_SUCCESS_ERRNO (
      zmq_setsockopt (pubsocket, ZMQ_XPUB_NODROP, &optval, mem::size_of::<optval>()));

    // Wait before starting TX operations till 1 subscriber has subscribed
    // (in this test there's 1 subscriber only)
    const char subscription_to_all_topics[] = {1, 0};
    recv_string_expect_success (pubsocket, subscription_to_all_topics, 0);

    u64 send_count = 0;
    while (true) {
let mut msg = ZmqMessage::default();
        int rc = zmq_msg_init_size (&msg, NUM_BYTES_PER_MSG);
        assert (rc == 0);

        /* Fill in message content with 'AAAAAA' */
        memset (zmq_msg_data (&msg), 'A', NUM_BYTES_PER_MSG);

        /* Send the message to the socket */
        rc = zmq_msg_send (&msg, pubsocket, ZMQ_DONTWAIT);
        if (rc != -1) {
            send_count+= 1;
        } else {
            TEST_ASSERT_SUCCESS_ERRNO (zmq_msg_close (&msg));
            break;
        }
    }

    // VERIFY EXPECTED RESULTS
    // EXPLANATION FOR TX TO BE CONSIDERED SUCCESSFUL:
    // this test has 3 threads doing I/O across 2 queues. Depending on the scheduling,
    // it might happen that 20, 30 or 40 messages go through before the pub blocks.
    // That's because the receiver thread gets kicked once every (hwm_ + 1) / 2 sent
    // messages (search for zeromq sources compute_lwm function).
    // So depending on the scheduling of the second thread, the publisher might get one,
    // two or three more batches in. The ceiling is 40 as there's 2 queues.
    //
    assert (4 * HWM >= send_count && 2 * HWM <= send_count);

    // CLEANUP

    zmq_close (pubsocket);
}

static void subscriber_thread_main (pvoid_: *mut c_void)
{
    const proxy_hwm_cfg_t *const cfg =
      static_cast<const proxy_hwm_cfg_t *> (pvoid_);

    void *subsocket = zmq_socket (cfg.context, ZMQ_SUB);
    assert (subsocket);

    lower_hwm (subsocket);

    TEST_ASSERT_SUCCESS_ERRNO (zmq_setsockopt (subsocket, ZMQ_SUBSCRIBE, 0, 0));

    TEST_ASSERT_SUCCESS_ERRNO (zmq_connect (subsocket, cfg.backend_endpoint));


    // receive all sent messages
    u64 rxsuccess = 0;
    bool success = true;
    while (success) {
let mut msg = ZmqMessage::default();
        int rc = zmq_msg_init (&msg);
        assert (rc == 0);

        rc = zmq_msg_recv (&msg, subsocket, 0);
        if (rc != -1) {
            TEST_ASSERT_SUCCESS_ERRNO (zmq_msg_close (&msg));
            rxsuccess+= 1;

            // after receiving 1st message, set a finite timeout (default is infinite)
            int timeout_ms = 100;
            TEST_ASSERT_SUCCESS_ERRNO (zmq_setsockopt (
              subsocket, ZMQ_RCVTIMEO, &timeout_ms, mem::size_of::<timeout_ms>()));
        } else {
            break;
        }

        msleep (100);
    }


    // VERIFY EXPECTED RESULTS
    // EXPLANATION FOR RX TO BE CONSIDERED SUCCESSFUL:
    // see publisher thread why we have 3 possible outcomes as number of RX messages

    assert (4 * HWM >= rxsuccess && 2 * HWM <= rxsuccess);

    // INFORM THAT WE COMPLETED:

    zmq_atomic_counter_inc (cfg.subscriber_received_all);

    // CLEANUP

    zmq_close (subsocket);
}

bool recv_stat (sock_: *mut c_void, last_: bool, u64 *res_)
{
    ZmqMessage stats_msg;

    int rc = zmq_msg_init (&stats_msg);
    assert (rc == 0);

    rc = zmq_msg_recv (&stats_msg, sock_, 0); //ZMQ_DONTWAIT);
    if (rc == -1 && errno == EAGAIN) {
        rc = zmq_msg_close (&stats_msg);
        assert (rc == 0);
        return false; // cannot retrieve the stat
    }

    assert (rc == mem::size_of::<u64>());
    memcpy (res_, zmq_msg_data (&stats_msg), zmq_msg_size (&stats_msg));

    rc = zmq_msg_close (&stats_msg);
    assert (rc == 0);

    more: i32;
    size_t moresz = sizeof more;
    rc = zmq_getsockopt (sock_, ZMQ_RCVMORE, &more, &moresz);
    assert (rc == 0);
    assert ((last_ && !more) || (!last_ && more));

    return true;
}

// Utility function to interrogate the proxy:

typedef struct
{
    u64 msg_in;
    u64 bytes_in;
    u64 msg_out;
    u64 bytes_out;
} ZmqSocketStats;

typedef struct
{
    ZmqSocketStats frontend;
    ZmqSocketStats backend;
} zmq_proxy_stats_t;

bool check_proxy_stats (control_proxy_: *mut c_void)
{
    zmq_proxy_stats_t total_stats;
    rc: i32;

    rc = zmq_send (control_proxy_, "STATISTICS", 10, ZMQ_DONTWAIT);
    assert (rc == 10 || (rc == -1 && errno == EAGAIN));
    if (rc == -1 && errno == EAGAIN) {
        return false;
    }

    // first frame of the reply contains FRONTEND stats:
    if (!recv_stat (control_proxy_, false, &total_stats.frontend.msg_in)) {
        return false;
    }

    recv_stat (control_proxy_, false, &total_stats.frontend.bytes_in);
    recv_stat (control_proxy_, false, &total_stats.frontend.msg_out);
    recv_stat (control_proxy_, false, &total_stats.frontend.bytes_out);

    // second frame of the reply contains BACKEND stats:
    recv_stat (control_proxy_, false, &total_stats.backend.msg_in);
    recv_stat (control_proxy_, false, &total_stats.backend.bytes_in);
    recv_stat (control_proxy_, false, &total_stats.backend.msg_out);
    recv_stat (control_proxy_, true, &total_stats.backend.bytes_out);

    return true;
}

static void proxy_stats_asker_thread_main (pvoid_: *mut c_void)
{
    const proxy_hwm_cfg_t *const cfg =
      static_cast<const proxy_hwm_cfg_t *> (pvoid_);

    // CONTROL REQ

    void *control_req =
      zmq_socket (cfg.context,
                  ZMQ_REQ); // this one can be used to send command to the proxy
    assert (control_req);

    // connect CONTROL-REQ: a socket to which send commands
    int rc = zmq_connect (control_req, cfg.control_endpoint);
    assert (rc == 0);


    // IMPORTANT: by setting the tx/rx timeouts, we avoid getting blocked when interrogating a proxy which is
    //            itself blocked in a zmq_msg_send() on its XPUB socket having ZMQ_XPUB_NODROP=1!

    int optval = 10;
    rc = zmq_setsockopt (control_req, ZMQ_SNDTIMEO, &optval, mem::size_of::<optval>());
    assert (rc == 0);
    rc = zmq_setsockopt (control_req, ZMQ_RCVTIMEO, &optval, mem::size_of::<optval>());
    assert (rc == 0);

    optval = 10;
    rc =
      zmq_setsockopt (control_req, ZMQ_REQ_CORRELATE, &optval, mem::size_of::<optval>());
    assert (rc == 0);

    rc =
      zmq_setsockopt (control_req, ZMQ_REQ_RELAXED, &optval, mem::size_of::<optval>());
    assert (rc == 0);


    // Start!

    while (!zmq_atomic_counter_value (cfg.subscriber_received_all)) {
        check_proxy_stats (control_req);
        usleep (1000); // 1ms -> in best case we will get 1000updates/second
    }


    // Ask the proxy to exit: the subscriber has received all messages

    rc = zmq_send (control_req, "TERMINATE", 9, 0);
    assert (rc == 9);

    zmq_close (control_req);
}

static void proxy_thread_main (pvoid_: *mut c_void)
{
    const proxy_hwm_cfg_t *const cfg =
      static_cast<const proxy_hwm_cfg_t *> (pvoid_);
    rc: i32;

    // FRONTEND SUB

    void *frontend_xsub = zmq_socket (
      cfg.context,
      ZMQ_XSUB); // the frontend is the one exposed to internal threads (INPROC)
    assert (frontend_xsub);

    lower_hwm (frontend_xsub);

    // Bind FRONTEND
    rc = zmq_bind (frontend_xsub, cfg.frontend_endpoint);
    assert (rc == 0);


    // BACKEND PUB

    void *backend_xpub = zmq_socket (
      cfg.context,
      ZMQ_XPUB); // the backend is the one exposed to the external world (TCP)
    assert (backend_xpub);

    int optval = 1;
    rc =
      zmq_setsockopt (backend_xpub, ZMQ_XPUB_NODROP, &optval, mem::size_of::<optval>());
    assert (rc == 0);

    lower_hwm (backend_xpub);

    // Bind BACKEND
    rc = zmq_bind (backend_xpub, cfg.backend_endpoint);
    assert (rc == 0);


    // CONTROL REP

    void *control_rep = zmq_socket (
      cfg.context,
      ZMQ_REP); // this one is used by the proxy to receive&reply to commands
    assert (control_rep);

    // Bind CONTROL
    rc = zmq_bind (control_rep, cfg.control_endpoint);
    assert (rc == 0);


    // start proxying!

    zmq_proxy_steerable (frontend_xsub, backend_xpub, null_mut(), control_rep);

    zmq_close (frontend_xsub);
    zmq_close (backend_xpub);
    zmq_close (control_rep);
}


// The main thread simply starts several clients and a server, and then
// waits for the server to finish.

int main (void)
{
    setup_test_environment ();

    void *context = zmq_ctx_new ();
    assert (context);


    // START ALL SECONDARY THREADS

    proxy_hwm_cfg_t cfg;
    cfg.context = context;
    cfg.frontend_endpoint = "inproc://frontend";
    cfg.backend_endpoint = "inproc://backend";
    cfg.control_endpoint = "inproc://ctrl";
    cfg.subscriber_received_all = zmq_atomic_counter_new ();

    void *proxy = zmq_threadstart (&proxy_thread_main, (void *) &cfg);
    assert (proxy != 0);
    void *publisher = zmq_threadstart (&publisher_thread_main, (void *) &cfg);
    assert (publisher != 0);
    void *subscriber = zmq_threadstart (&subscriber_thread_main, (void *) &cfg);
    assert (subscriber != 0);
    void *asker =
      zmq_threadstart (&proxy_stats_asker_thread_main, (void *) &cfg);
    assert (asker != 0);


    // CLEANUP

    zmq_threadclose (publisher);
    zmq_threadclose (subscriber);
    zmq_threadclose (asker);
    zmq_threadclose (proxy);

    int rc = zmq_ctx_term (context);
    assert (rc == 0);

    zmq_atomic_counter_destroy (&cfg.subscriber_received_all);

    return 0;
}
