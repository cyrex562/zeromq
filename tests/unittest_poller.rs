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

// #include "../tests/testutil.hpp"

// #include <poller.hpp>
// #include <i_poll_events.hpp>
// #include <ip.hpp>

// #include <unity.h>

// #ifndef _WIN32
// #include <unistd.h>
// #define closesocket close
// #endif

void setUp ()
{
}
void tearDown ()
{
}

void test_create ()
{
    ThreadCtx thread_ctx;
    Poller poller (thread_ctx);
}

#if 0
// TODO this triggers an assertion. should it be a valid use case?
void test_start_empty ()
{
    ThreadCtx thread_ctx;
    Poller poller (thread_ctx);
    poller.start ();
    msleep (SETTLE_TIME);
}
// #endif

struct test_events_t : i_poll_events
{
    test_events_t (fd: ZmqFileDesc, Poller &poller_) :
        _fd (fd),
        poller (poller_)
    {
        (void) _fd;
    }

    void in_event ()
    {
        poller.rm_fd (_handle);
        _handle = (Poller::handle_t) null_mut();

        // this must only be incremented after rm_fd
        in_events.add (1);
    }


    void out_event ()
    {
        // TODO
    }


    void timer_event (id_: i32)
    {
        LIBZMQ_UNUSED (id_);
        poller.rm_fd (_handle);
        _handle = (Poller::handle_t) null_mut();

        // this must only be incremented after rm_fd
        timer_events.add (1);
    }

    void set_handle (Poller::handle_t handle_) { _handle = handle_; }

    AtomicCounter in_events, timer_events;

  //
    ZmqFileDesc _fd;
    Poller &poller;
    Poller::handle_t _handle;
};

void wait_in_events (test_events_t &events_)
{
    void *watch = zmq_stopwatch_start ();
    while (events_.in_events.get () < 1) {
        msleep (1);
// #ifdef ZMQ_BUILD_DRAFT
        TEST_ASSERT_LESS_OR_EQUAL_MESSAGE (SETTLE_TIME,
                                           zmq_stopwatch_intermediate (watch),
                                           "Timeout waiting for in event");
// #endif
    }
    zmq_stopwatch_stop (watch);
}

void wait_timer_events (test_events_t &events_)
{
    void *watch = zmq_stopwatch_start ();
    while (events_.timer_events.get () < 1) {
        msleep (1);
// #ifdef ZMQ_BUILD_DRAFT
        TEST_ASSERT_LESS_OR_EQUAL_MESSAGE (SETTLE_TIME,
                                           zmq_stopwatch_intermediate (watch),
                                           "Timeout waiting for timer event");
// #endif
    }
    zmq_stopwatch_stop (watch);
}

void create_nonblocking_fdpair (ZmqFileDesc *r_, ZmqFileDesc *w_)
{
    int rc = make_fdpair (r_, w_);
    TEST_ASSERT_EQUAL_INT (0, rc);
    TEST_ASSERT_NOT_EQUAL (retired_fd, *r_);
    TEST_ASSERT_NOT_EQUAL (retired_fd, *w_);
    unblock_socket (*r_);
    unblock_socket (*w_);
}

void send_signal (ZmqFileDesc w_)
{
// #if defined ZMQ_HAVE_EVENTFD
    const u64 inc = 1;
    ssize_t sz = write (w_, &inc, mem::size_of::<inc>());
    assert (sz == mem::size_of::<inc>());
// #else
    {
        char msg[] = "test";
        int rc = send (w_, msg, mem::size_of::<msg>(), 0);
        assert (rc == mem::size_of::<msg>());
    }
// #endif
}

void close_fdpair (ZmqFileDesc w_, ZmqFileDesc r_)
{
    int rc = closesocket (w_);
    TEST_ASSERT_EQUAL_INT (0, rc);
// #if !defined ZMQ_HAVE_EVENTFD
    rc = closesocket (r_);
    TEST_ASSERT_EQUAL_INT (0, rc);
// #else
    LIBZMQ_UNUSED (r_);
// #endif
}

void test_add_fd_and_start_and_receive_data ()
{
    ThreadCtx thread_ctx;
    Poller poller (thread_ctx);

    ZmqFileDesc r, w;
    create_nonblocking_fdpair (&r, &w);

    test_events_t events (r, poller);

    Poller::handle_t handle = poller.add_fd (r, &events);
    events.set_handle (handle);
    poller.set_pollin (handle);
    poller.start ();

    send_signal (w);

    wait_in_events (events);

    // required cleanup
    close_fdpair (w, r);
}

void test_add_fd_and_remove_by_timer ()
{
    ZmqFileDesc r, w;
    create_nonblocking_fdpair (&r, &w);

    ThreadCtx thread_ctx;
    Poller poller (thread_ctx);

    test_events_t events (r, poller);

    Poller::handle_t handle = poller.add_fd (r, &events);
    events.set_handle (handle);

    poller.add_timer (50, &events, 0);
    poller.start ();

    wait_timer_events (events);

    // required cleanup
    close_fdpair (w, r);
}

// #ifdef _WIN32
void test_add_fd_with_pending_failing_connect ()
{
    ThreadCtx thread_ctx;
    Poller poller (thread_ctx);

    ZmqFileDesc bind_socket = socket (AF_INET, SOCK_STREAM, 0);
    sockaddr_in addr = {0};
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = htonl (INADDR_LOOPBACK);
    addr.sin_port = 0;
    TEST_ASSERT_EQUAL_INT (0, bind (bind_socket,
                                    reinterpret_cast<const sockaddr *> (&addr),
                                    mem::size_of::<addr>()));

    int addr_len = static_cast<int> (mem::size_of::<addr>());
    TEST_ASSERT_EQUAL_INT (0, getsockname (bind_socket,
                                           reinterpret_cast<sockaddr *> (&addr),
                                           &addr_len));

    ZmqFileDesc connect_socket = socket (AF_INET, SOCK_STREAM, 0);
    unblock_socket (connect_socket);

    TEST_ASSERT_EQUAL_INT (
      -1, connect (connect_socket, reinterpret_cast<const sockaddr *> (&addr),
                   mem::size_of::<addr>()));
    TEST_ASSERT_EQUAL_INT (WSAEWOULDBLOCK, WSAGetLastError ());

    test_events_t events (connect_socket, poller);

    Poller::handle_t handle = poller.add_fd (connect_socket, &events);
    events.set_handle (handle);
    poller.set_pollin (handle);
    poller.start ();

    wait_in_events (events);

    value: i32;
    int value_len = mem::size_of::<value>();
    TEST_ASSERT_EQUAL_INT (0, getsockopt (connect_socket, SOL_SOCKET, SO_ERROR,
                                          reinterpret_cast<char *> (&value),
                                          &value_len));
    TEST_ASSERT_EQUAL_INT (WSAECONNREFUSED, value);

    // required cleanup
    close (connect_socket);
    close (bind_socket);
}
// #endif

int main (void)
{
    UNITY_BEGIN ();

    initialize_network ();
    setup_test_environment ();

    RUN_TEST (test_create);
    RUN_TEST (test_add_fd_and_start_and_receive_data);
    RUN_TEST (test_add_fd_and_remove_by_timer);

// #if defined _WIN32
    RUN_TEST (test_add_fd_with_pending_failing_connect);
// #endif

    shutdown_network ();

    return UNITY_END ();
}
