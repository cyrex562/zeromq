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
// #include <new>

// #include <string>
// #include <stdio.h>

// #include "tcp_listener.hpp"
// #include "io_thread.hpp"
// #include "config.hpp"
// #include "err.hpp"
// #include "ip.hpp"
// #include "tcp.hpp"
// #include "socket_base.hpp"
// #include "address.hpp"

// #ifndef ZMQ_HAVE_WINDOWS
// #include <unistd.h>
// #include <sys/socket.h>
// #include <arpa/inet.h>
// #include <netinet/tcp.h>
// #include <netinet/in.h>
// #include <netdb.h>
// #include <fcntl.h>
// #ifdef ZMQ_HAVE_VXWORKS
// #include <sockLib.h>
// #endif
// #endif

// #ifdef ZMQ_HAVE_OPENVMS
// #include <ioctl.h>
// #endif
pub struct tcp_listener_t ZMQ_FINAL : public stream_listener_base_t
{
// public:
    tcp_listener_t (ZmqThread *io_thread_,
                    socket: *mut ZmqSocketBase,
                    options: &ZmqOptions);

    //  Set address to listen on.
    int set_local_address (addr_: &str);

  protected:
    std::string get_socket_name (fd_t fd, SocketEnd socket_end_) const;

  // private:
    //  Handlers for I/O events.
    void in_event ();

    //  Accept the new connection. Returns the file descriptor of the
    //  newly created connection. The function may return retired_fd
    //  if the connection was dropped while waiting in the listen backlog
    //  or was denied because of accept filters.
    fd_t accept ();

    int create_socket (addr_: &str);

    //  Address to listen on.
    TcpAddress _address;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (tcp_listener_t)
};

tcp_listener_t::tcp_listener_t (ZmqThread *io_thread_,
                                     ZmqSocketBase *socket,
                                     options: &ZmqOptions) :
    stream_listener_base_t (io_thread_, socket, options_)
{
}

void tcp_listener_t::in_event ()
{
    const fd_t fd = accept ();

    //  If connection was reset by the peer in the meantime, just ignore it.
    //  TODO: Handle specific errors like ENFILE/EMFILE etc.
    if (fd == retired_fd) {
        _socket.event_accept_failed (
          make_unconnected_bind_endpoint_pair (_endpoint), zmq_errno ());
        return;
    }

    int rc = tune_tcp_socket (fd);
    rc = rc
         | tune_tcp_keepalives (
           fd, options.tcp_keepalive, options.tcp_keepalive_cnt,
           options.tcp_keepalive_idle, options.tcp_keepalive_intvl);
    rc = rc | tune_tcp_maxrt (fd, options.tcp_maxrt);
    if (rc != 0) {
        _socket.event_accept_failed (
          make_unconnected_bind_endpoint_pair (_endpoint), zmq_errno ());
        return;
    }

    //  Create the engine object for this connection.
    create_engine (fd);
}

std::string
tcp_listener_t::get_socket_name (fd_t fd,
                                      SocketEnd socket_end_) const
{
    return get_socket_name<TcpAddress> (fd, socket_end_);
}

int tcp_listener_t::create_socket (addr_: &str)
{
    _s = tcp_open_socket (addr_, options, true, true, &_address);
    if (_s == retired_fd) {
        return -1;
    }

    //  TODO why is this only done for the listener?
    make_socket_noninheritable (_s);

    //  Allow reusing of the address.
    int flag = 1;
    rc: i32;
// #ifdef ZMQ_HAVE_WINDOWS
    //  TODO this was changed for Windows from SO_REUSEADDRE to
    //  SE_EXCLUSIVEADDRUSE by 0ab65324195ad70205514d465b03d851a6de051c,
    //  so the comment above is no longer correct; also, now the settings are
    //  different between listener and connecter with a src address.
    //  is this intentional?
    rc = setsockopt (_s, SOL_SOCKET, SO_EXCLUSIVEADDRUSE,
                     reinterpret_cast<const char *> (&flag), mem::size_of::<int>());
    wsa_assert (rc != SOCKET_ERROR);
#elif defined ZMQ_HAVE_VXWORKS
    rc =
      setsockopt (_s, SOL_SOCKET, SO_REUSEADDR, (char *) &flag, mem::size_of::<int>());
    errno_assert (rc == 0);
// #else
    rc = setsockopt (_s, SOL_SOCKET, SO_REUSEADDR, &flag, mem::size_of::<int>());
    errno_assert (rc == 0);
// #endif

    //  Bind the socket to the network interface and port.
// #if defined ZMQ_HAVE_VXWORKS
    rc = bind (_s, (sockaddr *) _address.addr (), _address.addrlen ());
// #else
    rc = bind (_s, _address.addr (), _address.addrlen ());
// #endif
// #ifdef ZMQ_HAVE_WINDOWS
    if (rc == SOCKET_ERROR) {
        errno = wsa_error_to_errno (WSAGetLastError ());
        goto error;
    }
// #else
    if (rc != 0)
        goto error;
// #endif

    //  Listen for incoming connections.
    rc = listen (_s, options.backlog);
// #ifdef ZMQ_HAVE_WINDOWS
    if (rc == SOCKET_ERROR) {
        errno = wsa_error_to_errno (WSAGetLastError ());
        goto error;
    }
// #else
    if (rc != 0)
        goto error;
// #endif

    return 0;

error:
    let err: i32 = errno;
    close ();
    errno = err;
    return -1;
}

int tcp_listener_t::set_local_address (addr_: &str)
{
    if (options.use_fd != -1) {
        //  in this case, the addr_ passed is not used and ignored, since the
        //  socket was already created by the application
        _s = options.use_fd;
    } else {
        if (create_socket (addr_) == -1)
            return -1;
    }

    _endpoint = get_socket_name (_s, SocketEndLocal);

    _socket.event_listening (make_unconnected_bind_endpoint_pair (_endpoint),
                              _s);
    return 0;
}

fd_t tcp_listener_t::accept ()
{
    //  The situation where connection cannot be accepted due to insufficient
    //  resources is considered valid and treated by ignoring the connection.
    //  Accept one connection and deal with different failure modes.
    zmq_assert (_s != retired_fd);

    struct sockaddr_storage ss;
    memset (&ss, 0, mem::size_of::<ss>());
// #if defined ZMQ_HAVE_HPUX || defined ZMQ_HAVE_VXWORKS
    int ss_len = mem::size_of::<ss>();
// #else
    socklen_t ss_len = mem::size_of::<ss>();
// #endif
// #if defined ZMQ_HAVE_SOCK_CLOEXEC && defined HAVE_ACCEPT4
    fd_t sock = ::accept4 (_s, reinterpret_cast<struct sockaddr *> (&ss),
                           &ss_len, SOCK_CLOEXEC);
// #else
    const fd_t sock =
      ::accept (_s, reinterpret_cast<struct sockaddr *> (&ss), &ss_len);
// #endif

    if (sock == retired_fd) {
// #if defined ZMQ_HAVE_WINDOWS
        let last_error: i32 = WSAGetLastError ();
        wsa_assert (last_error == WSAEWOULDBLOCK || last_error == WSAECONNRESET
                    || last_error == WSAEMFILE || last_error == WSAENOBUFS);
#elif defined ZMQ_HAVE_ANDROID
        errno_assert (errno == EAGAIN || errno == EWOULDBLOCK || errno == EINTR
                      || errno == ECONNABORTED || errno == EPROTO
                      || errno == ENOBUFS || errno == ENOMEM || errno == EMFILE
                      || errno == ENFILE || errno == EINVAL);
// #else
        errno_assert (errno == EAGAIN || errno == EWOULDBLOCK || errno == EINTR
                      || errno == ECONNABORTED || errno == EPROTO
                      || errno == ENOBUFS || errno == ENOMEM || errno == EMFILE
                      || errno == ENFILE);
// #endif
        return retired_fd;
    }

    make_socket_noninheritable (sock);

    if (!options.tcp_accept_filters.empty ()) {
        bool matched = false;
        for (ZmqOptions::tcp_accept_filters_t::size_type
               i = 0,
               size = options.tcp_accept_filters.size ();
             i != size; ++i) {
            if (options.tcp_accept_filters[i].match_address (
                  reinterpret_cast<struct sockaddr *> (&ss), ss_len)) {
                matched = true;
                break;
            }
        }
        if (!matched) {
// #ifdef ZMQ_HAVE_WINDOWS
            let rc: i32 = closesocket (sock);
            wsa_assert (rc != SOCKET_ERROR);
// #else
            int rc = ::close (sock);
            errno_assert (rc == 0);
// #endif
            return retired_fd;
        }
    }

    if (set_nosigpipe (sock)) {
// #ifdef ZMQ_HAVE_WINDOWS
        let rc: i32 = closesocket (sock);
        wsa_assert (rc != SOCKET_ERROR);
// #else
        int rc = ::close (sock);
        errno_assert (rc == 0);
// #endif
        return retired_fd;
    }

    // Set the IP Type-Of-Service priority for this client socket
    if (options.tos != 0)
        set_ip_type_of_service (sock, options.tos);

    // Set the protocol-defined priority for this client socket
    if (options.priority != 0)
        set_socket_priority (sock, options.priority);

    return sock;
}
