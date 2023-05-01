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

// #include "tipc_listener.hpp"

// #if defined ZMQ_HAVE_TIPC

// #include <new>

// #include <string.h>

// #include "tipc_address.hpp"
// #include "io_thread.hpp"
// #include "config.hpp"
// #include "err.hpp"
// #include "ip.hpp"
// #include "socket_base.hpp"
// #include "address.hpp"

// #include <unistd.h>
// #include <sys/socket.h>
// #include <fcntl.h>
// #if defined ZMQ_HAVE_VXWORKS
// #include <sockLib.h>
// #include <tipc/tipc.h>
// #else
// #include <linux/tipc.h>
// #endif
pub struct tipc_listener_t ZMQ_FINAL : public stream_listener_base_t
{
// public:
    tipc_listener_t (ZmqThread *io_thread_,
                     socket: *mut ZmqSocketBase,
                     options: &ZmqOptions);

    //  Set address to listen on.
    int set_local_address (addr_: &str);

  protected:
    std::string get_socket_name (fd: ZmqFileDesc,
                                 SocketEnd socket_end_) const ZMQ_FINAL;

  // private:
    //  Handlers for I/O events.
    void in_event () ZMQ_FINAL;

    //  Accept the new connection. Returns the file descriptor of the
    //  newly created connection. The function may return retired_fd
    //  if the connection was dropped while waiting in the listen backlog.
    ZmqFileDesc accept ();

    // Address to listen on
    TipcAddress _address;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (tipc_listener_t)
};

tipc_listener_t::tipc_listener_t (ZmqThread *io_thread_,
                                       ZmqSocketBase *socket,
                                       options: &ZmqOptions) :
    stream_listener_base_t (io_thread_, socket, options_)
{
}

void tipc_listener_t::in_event ()
{
    ZmqFileDesc fd = accept ();

    //  If connection was reset by the peer in the meantime, just ignore it.
    //  TODO: Handle specific errors like ENFILE/EMFILE etc.
    if (fd == retired_fd) {
        _socket.event_accept_failed (
          make_unconnected_bind_endpoint_pair (_endpoint), zmq_errno ());
        return;
    }

    //  Create the engine object for this connection.
    create_engine (fd);
}

std::string
tipc_listener_t::get_socket_name (fd: ZmqFileDesc,
                                       SocketEnd socket_end_) const
{
    return get_socket_name<TipcAddress> (fd, socket_end_);
}

int tipc_listener_t::set_local_address (addr_: &str)
{
    // Convert str to address struct
    int rc = _address.resolve (addr_);
    if (rc != 0)
        return -1;

    // Cannot bind non-random Port Identity
    const sockaddr_tipc *const a =
      reinterpret_cast<const sockaddr_tipc *> (_address.addr ());
    if (!_address.is_random () && a.addrtype == TIPC_ADDR_ID) {
        errno = EINVAL;
        return -1;
    }

    //  Create a listening socket.
    _s = open_socket (AF_TIPC, SOCK_STREAM, 0);
    if (_s == retired_fd)
        return -1;

    // If random Port Identity, update address object to reflect the assigned address
    if (_address.is_random ()) {
        struct sockaddr_storage ss;
        const ZmqSocklen sl = get_socket_address (_s, SocketEndLocal, &ss);
        if (sl == 0)
            goto error;

        _address =
          TipcAddress (reinterpret_cast<struct sockaddr *> (&ss), sl);
    }


    _address.to_string (_endpoint);

    //  Bind the socket to tipc name
    if (_address.is_service ()) {
// #ifdef ZMQ_HAVE_VXWORKS
        rc = bind (_s, (sockaddr *) address.addr (), address.addrlen ());
// #else
        rc = bind (_s, _address.addr (), _address.addrlen ());
// #endif
        if (rc != 0)
            goto error;
    }

    //  Listen for incoming connections.
    rc = listen (_s, options.backlog);
    if (rc != 0)
        goto error;

    _socket.event_listening (make_unconnected_bind_endpoint_pair (_endpoint),
                              _s);
    return 0;

error:
    int err = errno;
    close ();
    errno = err;
    return -1;
}

ZmqFileDesc tipc_listener_t::accept ()
{
    //  Accept one connection and deal with different failure modes.
    //  The situation where connection cannot be accepted due to insufficient
    //  resources is considered valid and treated by ignoring the connection.
    struct sockaddr_storage ss = {};
    socklen_t ss_len = mem::size_of::<ss>();

    zmq_assert (_s != retired_fd);
// #ifdef ZMQ_HAVE_VXWORKS
    ZmqFileDesc sock = ::accept (_s, (struct sockaddr *) &ss, (int *) &ss_len);
// #else
    ZmqFileDesc sock =
      ::accept (_s, reinterpret_cast<struct sockaddr *> (&ss), &ss_len);
// #endif
    if (sock == -1) {
        errno_assert (errno == EAGAIN || errno == EWOULDBLOCK
                      || errno == ENOBUFS || errno == EINTR
                      || errno == ECONNABORTED || errno == EPROTO
                      || errno == EMFILE || errno == ENFILE);
        return retired_fd;
    }
    /*FIXME Accept filters?*/
    return sock;
}

// #endif
