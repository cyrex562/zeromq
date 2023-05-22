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

// #include "precompiled.hpp"
// #include "stream_listener_base.hpp"
// #include "session_base.hpp"
// #include "socket_base.hpp"
// #include "zmtp_engine.hpp"
// #include "raw_engine.hpp"

// #ifndef ZMQ_HAVE_WINDOWS
// #include <unistd.h>
// #else
// #include <winsock2.h>
// #endif
pub struct stream_listener_base_t : public ZmqOwn, public ZmqIoObject
{
//
    stream_listener_base_t (ZmqIoThread *io_thread_,
                            socket: *mut ZmqSocketBase,
                            options: &ZmqOptions);
    ~stream_listener_base_t () ;

    // Get the bound address for use with wildcards
    int get_local_address (std::string &addr_) const;


    virtual std::string get_socket_name (fd: ZmqFileDesc,
                                         SocketEnd socket_end_) const = 0;

  //
    //  Handlers for incoming commands.
    void process_plug () ;
    void process_term (linger: i32) ;


    //  Close the listening socket.
    virtual int close ();

    virtual void create_engine (ZmqFileDesc fd);

    //  Underlying socket.
    ZmqFileDesc _s;

    //  Handle corresponding to the listening socket.
    let mut _handle: ZmqHandle;

    //  Socket the listener belongs to.
    ZmqSocketBase *_socket;

    // String representation of endpoint to bind to
    _endpoint: String;

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (stream_listener_base_t)
};

stream_listener_base_t::stream_listener_base_t (
  ZmqIoThread *io_thread_,
  socket: *mut ZmqSocketBase,
  options: &ZmqOptions) :
    ZmqOwn (io_thread_, options_),
    ZmqIoObject (io_thread_),
    _s (retired_fd),
    _handle ( (null_mut())),
    self._socket (socket)
{
}

stream_listener_base_t::~stream_listener_base_t ()
{
    // zmq_assert (_s == retired_fd);
    // zmq_assert (!_handle);
}

int stream_listener_base_t::get_local_address (std::string &addr_) const
{
    addr_ = get_socket_name (_s, SocketEndLocal);
    return addr_.is_empty() ? -1 : 0;
}

void stream_listener_base_t::process_plug ()
{
    //  Start polling for incoming connections.
    _handle = add_fd (_s);
    set_pollin (_handle);
}

void stream_listener_base_t::process_term (linger: i32)
{
    rm_fd (_handle);
    _handle =  (null_mut());
    close ();
    ZmqOwn::process_term (linger);
}

int stream_listener_base_t::close ()
{
    // TODO this is identical to stream_connector_base_t::close

    // zmq_assert (_s != retired_fd);
// #ifdef ZMQ_HAVE_WINDOWS
    let rc: i32 = closesocket (_s);
    wsa_assert (rc != SOCKET_ERROR);
// #else
    let rc: i32 = ::close (_s);
    // errno_assert (rc == 0);
// #endif
    self._socket.event_closed (make_unconnected_bind_endpoint_pair (_endpoint), _s);
    _s = retired_fd;

    return 0;
}

void stream_listener_base_t::create_engine (ZmqFileDesc fd)
{
    const endpoint_uri_ZmqPair endpoint_pair (
      get_socket_name (fd, SocketEndLocal),
      get_socket_name (fd, SocketEndRemote), endpoint_type_bind);

    ZmqEngineInterface *engine;
    if (options.raw_socket)
        engine =  RawEngine (fd, options, endpoint_pair);
    else
        engine =  ZmqZmtpEngine (fd, options, endpoint_pair);
    // alloc_assert (engine);

    //  Choose I/O thread to run connecter in. Given that we are already
    //  running in an I/O thread, there must be at least one available.
    ZmqIoThread *io_thread = choose_io_thread (options.affinity);
    // zmq_assert (io_thread);

    //  Create and launch a session object.
    ZmqSessionBase *session =
      ZmqSessionBase::create (io_thread, false, self._socket, options, null_mut());
    // errno_assert (session);
    session.inc_seqnum ();
    launch_child (session);
    send_attach (session, engine, false);

    self._socket.event_accepted (endpoint_pair, fd);
}
