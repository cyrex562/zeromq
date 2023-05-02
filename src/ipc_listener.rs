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
// #include "ipc_listener.hpp"

// #if defined ZMQ_HAVE_IPC

// #include <new>

// #include <string.h>

// #include "ipc_address.hpp"
// #include "io_thread.hpp"
// #include "config.hpp"
// #include "err.hpp"
// #include "ip.hpp"
// #include "socket_base.hpp"
// #include "address.hpp"
pub struct ipc_listener_t  : public stream_listener_base_t
{
// public:
    ipc_listener_t (ZmqThread *io_thread_,
                    socket: *mut ZmqSocketBase,
                    options: &ZmqOptions);

    //  Set address to listen on.
    int set_local_address (addr_: &str);

  protected:
    std::string get_socket_name (fd: ZmqFileDesc, SocketEnd socket_end_) const;

  // private:
    //  Handlers for I/O events.
    void in_event ();

    //  Filter new connections if the OS provides a mechanism to get
    //  the credentials of the peer process.  Called from accept().
// #if defined ZMQ_HAVE_SO_PEERCRED || defined ZMQ_HAVE_LOCAL_PEERCRED
    bool filter (ZmqFileDesc sock_);
// #endif

    int close ();

    //  Accept the new connection. Returns the file descriptor of the
    //  newly created connection. The function may return retired_fd
    //  if the connection was dropped while waiting in the listen backlog.
    ZmqFileDesc accept ();

    //  True, if the underlying file for UNIX domain socket exists.
    _has_file: bool

    //  Name of the temporary directory (if any) that has the
    //  UNIX domain socket
    _tmp_socket_dirname: String;

    //  Name of the file associated with the UNIX domain address.
    _filename: String;

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ipc_listener_t)
};

// #ifdef _MSC_VER
// #ifdef ZMQ_IOTHREAD_POLLER_USE_SELECT
#error On Windows, IPC does not work with POLLER=select, use POLLER=epoll instead, or disable IPC transport
// #endif

// #include <afunix.h>
// #include <direct.h>

// #define rmdir _rmdir
// #define unlink _unlink

// #else
// #include <unistd.h>
// #include <sys/socket.h>
// #include <fcntl.h>
// #include <sys/un.h>
// #endif

// #ifdef ZMQ_HAVE_LOCAL_PEERCRED
// #include <sys/types.h>
// #include <sys/ucred.h>
// #endif
// #ifdef ZMQ_HAVE_SO_PEERCRED
// #include <sys/types.h>
// #include <pwd.h>
// #include <grp.h>
// #if defined ZMQ_HAVE_OPENBSD
// #define ucred sockpeercred
// #endif
// #endif

ipc_listener_t::ipc_listener_t (ZmqThread *io_thread_,
                                     ZmqSocketBase *socket,
                                     options: &ZmqOptions) :
    stream_listener_base_t (io_thread_, socket, options_), _has_file (false)
{
}

void ipc_listener_t::in_event ()
{
    const ZmqFileDesc fd = accept ();

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
ipc_listener_t::get_socket_name (fd: ZmqFileDesc,
                                      SocketEnd socket_end_) const
{
    return get_socket_name<IpcAddress> (fd, socket_end_);
}

int ipc_listener_t::set_local_address (addr_: &str)
{
    //  Create addr on stack for auto-cleanup
    std::string addr (addr_);

    //  Allow wildcard file
    if (options.use_fd == -1 && addr[0] == '*') {
        if (create_ipc_wildcard_address (_tmp_socket_dirname, addr) < 0) {
            return -1;
        }
    }

    //  Get rid of the file associated with the UNIX domain socket that
    //  may have been left behind by the previous run of the application.
    //  MUST NOT unlink if the FD is managed by the user, or it will stop
    //  working after the first client connects. The user will take care of
    //  cleaning up the file after the service is stopped.
    if (options.use_fd == -1) {
        ::unlink (addr.c_str ());
    }
    _filename.clear ();

    //  Initialise the address structure.
    IpcAddress address;
    int rc = address.resolve (addr.c_str ());
    if (rc != 0) {
        if (!_tmp_socket_dirname.empty ()) {
            // We need to preserve errno to return to the user
            let tmp_errno: i32 = errno;
            ::rmdir (_tmp_socket_dirname.c_str ());
            _tmp_socket_dirname.clear ();
            errno = tmp_errno;
        }
        return -1;
    }

    address.to_string (_endpoint);

    if (options.use_fd != -1) {
        _s = options.use_fd;
    } else {
        //  Create a listening socket.
        _s = open_socket (AF_UNIX, SOCK_STREAM, 0);
        if (_s == retired_fd) {
            if (!_tmp_socket_dirname.empty ()) {
                // We need to preserve errno to return to the user
                let tmp_errno: i32 = errno;
                ::rmdir (_tmp_socket_dirname.c_str ());
                _tmp_socket_dirname.clear ();
                errno = tmp_errno;
            }
            return -1;
        }

        //  Bind the socket to the file path.
        rc = bind (_s, const_cast<sockaddr *> (address.addr ()),
                   address.addrlen ());
        if (rc != 0)
            goto error;

        //  Listen for incoming connections.
        rc = listen (_s, options.backlog);
        if (rc != 0)
            goto error;
    }

    _filename = ZMQ_MOVE (addr);
    _has_file = true;

    _socket.event_listening (make_unconnected_bind_endpoint_pair (_endpoint),
                              _s);
    return 0;

error:
    let err: i32 = errno;
    close ();
    errno = err;
    return -1;
}

int ipc_listener_t::close ()
{
    zmq_assert (_s != retired_fd);
    const ZmqFileDesc fd_for_event = _s;
// #ifdef ZMQ_HAVE_WINDOWS
    int rc = closesocket (_s);
    wsa_assert (rc != SOCKET_ERROR);
// #else
    int rc = ::close (_s);
    errno_assert (rc == 0);
// #endif

    _s = retired_fd;

    if (_has_file && options.use_fd == -1) {
        if (!_tmp_socket_dirname.empty ()) {
            //  TODO review this behaviour, it is inconsistent with the use of
            //  unlink in open since 656cdb959a7482c45db979c1d08ede585d12e315;
            //  however, we must at least remove the file before removing the
            //  directory, otherwise it will always fail
            rc = ::unlink (_filename.c_str ());

            if (rc == 0) {
                rc = ::rmdir (_tmp_socket_dirname.c_str ());
                _tmp_socket_dirname.clear ();
            }
        }

        if (rc != 0) {
            _socket.event_close_failed (
              make_unconnected_bind_endpoint_pair (_endpoint), zmq_errno ());
            return -1;
        }
    }

    _socket.event_closed (make_unconnected_bind_endpoint_pair (_endpoint),
                           fd_for_event);
    return 0;
}

// #if defined ZMQ_HAVE_SO_PEERCRED

bool ipc_listener_t::filter (ZmqFileDesc sock_)
{
    if (options.ipc_uid_accept_filters.is_empty()
        && options.ipc_pid_accept_filters.is_empty()
        && options.ipc_gid_accept_filters.empty ())
        return true;

    struct ucred cred;
    socklen_t size = mem::size_of::<cred>();

    if (getsockopt (sock_, SOL_SOCKET, SO_PEERCRED, &cred, &size))
        return false;
    if (options.ipc_uid_accept_filters.find (cred.uid)
          != options.ipc_uid_accept_filters.end ()
        || options.ipc_gid_accept_filters.find (cred.gid)
             != options.ipc_gid_accept_filters.end ()
        || options.ipc_pid_accept_filters.find (cred.pid)
             != options.ipc_pid_accept_filters.end ())
        return true;

    const struct passwd *pw;
    const struct group *gr;

    if (!(pw = getpwuid (cred.uid)))
        return false;
    for (ZmqOptions::ipc_gid_accept_filters_t::const_iterator
           it = options.ipc_gid_accept_filters.begin (),
           end = options.ipc_gid_accept_filters.end ();
         it != end; it+= 1) {
        if (!(gr = getgrgid (*it)))
            continue;
        for (char **mem = gr.gr_mem; *mem; mem+= 1) {
            if (!strcmp (*mem, pw.pw_name))
                return true;
        }
    }
    return false;
}

#elif defined ZMQ_HAVE_LOCAL_PEERCRED

bool ipc_listener_t::filter (ZmqFileDesc sock_)
{
    if (options.ipc_uid_accept_filters.is_empty()
        && options.ipc_gid_accept_filters.empty ())
        return true;

    struct xucred cred;
    socklen_t size = mem::size_of::<cred>();

    if (getsockopt (sock_, 0, LOCAL_PEERCRED, &cred, &size))
        return false;
    if (cred.cr_version != XUCRED_VERSION)
        return false;
    if (options.ipc_uid_accept_filters.find (cred.cr_uid)
        != options.ipc_uid_accept_filters.end ())
        return true;
    for (int i = 0; i < cred.cr_ngroups; i+= 1) {
        if (options.ipc_gid_accept_filters.find (cred.cr_groups[i])
            != options.ipc_gid_accept_filters.end ())
            return true;
    }

    return false;
}

// #endif

ZmqFileDesc ipc_listener_t::accept ()
{
    //  Accept one connection and deal with different failure modes.
    //  The situation where connection cannot be accepted due to insufficient
    //  resources is considered valid and treated by ignoring the connection.
    zmq_assert (_s != retired_fd);
// #if defined ZMQ_HAVE_SOCK_CLOEXEC && defined HAVE_ACCEPT4
    ZmqFileDesc sock = ::accept4 (_s, null_mut(), null_mut(), SOCK_CLOEXEC);
// #else
    struct sockaddr_storage ss;
    memset (&ss, 0, mem::size_of::<ss>());
// #if defined ZMQ_HAVE_HPUX || defined ZMQ_HAVE_VXWORKS
    int ss_len = mem::size_of::<ss>();
// #else
    socklen_t ss_len = mem::size_of::<ss>();
// #endif

    const ZmqFileDesc sock =
      ::accept (_s, reinterpret_cast<struct sockaddr *> (&ss), &ss_len);
// #endif
    if (sock == retired_fd) {
// #if defined ZMQ_HAVE_WINDOWS
        let last_error: i32 = WSAGetLastError ();
        wsa_assert (last_error == WSAEWOULDBLOCK || last_error == WSAECONNRESET
                    || last_error == WSAEMFILE || last_error == WSAENOBUFS);
// #else
        errno_assert (errno == EAGAIN || errno == EWOULDBLOCK || errno == EINTR
                      || errno == ECONNABORTED || errno == EPROTO
                      || errno == ENFILE);
// #endif
        return retired_fd;
    }

    make_socket_noninheritable (sock);

    // IPC accept() filters
// #if defined ZMQ_HAVE_SO_PEERCRED || defined ZMQ_HAVE_LOCAL_PEERCRED
    if (!filter (sock)) {
        int rc = ::close (sock);
        errno_assert (rc == 0);
        return retired_fd;
    }
// #endif

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

    return sock;
}

// #endif
