/*
    Copyright (c) 2007-2016 Contributors as noted in the AUTHORS file

    This file is part of libzmq, the ZeroMQ core engine in C++.

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

// #include "precompiled.hpp"
// #include "stream_connecter_base.hpp"
// #include "session_base.hpp"
// #include "address.hpp"
// #include "random.hpp"
// #include "zmtp_engine.hpp"
// #include "raw_engine.hpp"

// #ifndef ZMQ_HAVE_WINDOWS
// #include <unistd.h>
// #else
// #include <winsock2.h>
// #endif

// #include <limits>

zmq::stream_connecter_base_t::stream_connecter_base_t (
  zmq::io_thread_t *io_thread_,
  zmq::session_base_t *session_,
  const zmq::ZmqOptions &options_,
  zmq::Address *addr_,
  bool delayed_start_) :
    own_t (io_thread_, options_),
    io_object_t (io_thread_),
    _addr (addr_),
    _s (retired_fd),
    _handle (static_cast<handle_t> (NULL)),
    _socket (session_->get_socket ()),
    _delayed_start (delayed_start_),
    _reconnect_timer_started (false),
    _current_reconnect_ivl (options.reconnect_ivl),
    _session (session_)
{
    zmq_assert (_addr);
    _addr->to_string (_endpoint);
    // TODO the return value is unused! what if it fails? if this is impossible
    // or does not matter, change such that endpoint in initialized using an
    // initializer, and make endpoint const
}

zmq::stream_connecter_base_t::~stream_connecter_base_t ()
{
    zmq_assert (!_reconnect_timer_started);
    zmq_assert (!_handle);
    zmq_assert (_s == retired_fd);
}

void zmq::stream_connecter_base_t::process_plug ()
{
    if (_delayed_start)
        add_reconnect_timer ();
    else
        start_connecting ();
}

void zmq::stream_connecter_base_t::process_term (linger_: i32)
{
    if (_reconnect_timer_started) {
        cancel_timer (reconnect_timer_id);
        _reconnect_timer_started = false;
    }

    if (_handle) {
        rm_handle ();
    }

    if (_s != retired_fd)
        close ();

    own_t::process_term (linger_);
}

void zmq::stream_connecter_base_t::add_reconnect_timer ()
{
    if (options.reconnect_ivl > 0) {
        const int interval = get_new_reconnect_ivl ();
        add_timer (interval, reconnect_timer_id);
        _socket->event_connect_retried (
          make_unconnected_connect_endpoint_pair (_endpoint), interval);
        _reconnect_timer_started = true;
    }
}

int zmq::stream_connecter_base_t::get_new_reconnect_ivl ()
{
    //  TODO should the random jitter be really based on the configured initial
    //  reconnect interval options.reconnect_ivl, or better on the
    //  _current_reconnect_ivl?

    //  The new interval is the current interval + random value.
    const int random_jitter = generate_random () % options.reconnect_ivl;
    const int interval =
      _current_reconnect_ivl < std::numeric_limits<int>::max () - random_jitter
        ? _current_reconnect_ivl + random_jitter
        : std::numeric_limits<int>::max ();

    //  Only change the new current reconnect interval if the maximum reconnect
    //  interval was set and if it's larger than the reconnect interval.
    if (options.reconnect_ivl_max > 0
        && options.reconnect_ivl_max > options.reconnect_ivl) {
        //  Calculate the next interval
        _current_reconnect_ivl =
          _current_reconnect_ivl < std::numeric_limits<int>::max () / 2
            ? std::min (_current_reconnect_ivl * 2, options.reconnect_ivl_max)
            : options.reconnect_ivl_max;
    }

    return interval;
}

void zmq::stream_connecter_base_t::rm_handle ()
{
    rm_fd (_handle);
    _handle = static_cast<handle_t> (NULL);
}

void zmq::stream_connecter_base_t::close ()
{
    // TODO before, this was an assertion for _s != retired_fd, but this does not match usage of close
    if (_s != retired_fd) {
// #ifdef ZMQ_HAVE_WINDOWS
        const int rc = closesocket (_s);
        wsa_assert (rc != SOCKET_ERROR);
// #else
        const int rc = ::close (_s);
        errno_assert (rc == 0);
// #endif
        _socket->event_closed (
          make_unconnected_connect_endpoint_pair (_endpoint), _s);
        _s = retired_fd;
    }
}

void zmq::stream_connecter_base_t::in_event ()
{
    //  We are not polling for incoming data, so we are actually called
    //  because of error here. However, we can get error on out event as well
    //  on some platforms, so we'll simply handle both events in the same way.
    out_event ();
}

void zmq::stream_connecter_base_t::create_engine (
  fd_t fd_, local_address_: &str)
{
    const endpoint_uri_pair_t endpoint_pair (local_address_, _endpoint,
                                             endpoint_type_connect);

    //  Create the engine object for this connection.
    i_engine *engine;
    if (options.raw_socket)
        engine = new (std::nothrow) raw_engine_t (fd_, options, endpoint_pair);
    else
        engine = new (std::nothrow) zmtp_engine_t (fd_, options, endpoint_pair);
    alloc_assert (engine);

    //  Attach the engine to the corresponding session object.
    send_attach (_session, engine);

    //  Shut the connecter down.
    terminate ();

    _socket->event_connected (endpoint_pair, fd_);
}

void zmq::stream_connecter_base_t::timer_event (id_: i32)
{
    zmq_assert (id_ == reconnect_timer_id);
    _reconnect_timer_started = false;
    start_connecting ();
}
pub struct stream_connecter_base_t : public own_t, public io_object_t
{
// public:
    //  If 'delayed_start' is true connecter first waits for a while,
    //  then starts connection process.
    stream_connecter_base_t (zmq::io_thread_t *io_thread_,
                             zmq::session_base_t *session_,
                             const ZmqOptions &options_,
                             Address *addr_,
                             bool delayed_start_);

    ~stream_connecter_base_t () ZMQ_OVERRIDE;

  protected:
    //  Handlers for incoming commands.
    void process_plug () ZMQ_FINAL;
    void process_term (linger_: i32) ZMQ_OVERRIDE;

    //  Handlers for I/O events.
    void in_event () ZMQ_OVERRIDE;
    void timer_event (id_: i32) ZMQ_OVERRIDE;

    //  Internal function to create the engine after connection was established.
    virtual void create_engine (fd_t fd, local_address_: &str);

    //  Internal function to add a reconnect timer
    void add_reconnect_timer ();

    //  Removes the handle from the poller.
    void rm_handle ();

    //  Close the connecting socket.
    void close ();

    //  Address to connect to. Owned by session_base_t.
    //  It is non-const since some parts may change during opening.
    Address *const _addr;

    //  Underlying socket.
    fd_t _s;

    //  Handle corresponding to the listening socket, if file descriptor is
    //  registered with the poller, or NULL.
    handle_t _handle;

    // String representation of endpoint to connect to
    std::string _endpoint;

    // Socket
    zmq::ZmqSocketBase *const _socket;

  // private:
    //  ID of the timer used to delay the reconnection.
    enum
    {
        reconnect_timer_id = 1
    };

    //  Internal function to return a reconnect backoff delay.
    //  Will modify the current_reconnect_ivl used for next call
    //  Returns the currently used interval
    int get_new_reconnect_ivl ();

    virtual void start_connecting () = 0;

    //  If true, connecter is waiting a while before trying to connect.
    const bool _delayed_start;

    //  True iff a timer has been started.
    bool _reconnect_timer_started;

    //  Current reconnect ivl, updated for backoff strategy
    _current_reconnect_ivl: i32;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (stream_connecter_base_t)

  protected:
    //  Reference to the session we belong to.
    zmq::session_base_t *const _session;
};
