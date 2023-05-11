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

use std::mem;
use std::ptr::null_mut;
use libc::{accept, bind, c_int, close, getsockopt, listen, rmdir, unlink};
use windows::Win32::Networking::WinSock::{closesocket, SOCK_STREAM, SOCKET_ERROR, SOL_SOCKET, WSA_ERROR, WSAECONNRESET, WSAEMFILE, WSAENOBUFS, WSAEWOULDBLOCK, WSAGetLastError};
use crate::address::{get_socket_name, SocketEnd};
use crate::address_family::AF_UNIX;
use crate::dish::DishSessionState::group;
use crate::endpoint::make_unconnected_bind_endpoint_pair;
use crate::fd::ZmqFileDesc;
use crate::io_thread::ZmqIoThread;
use crate::ip::{create_ipc_wildcard_address, make_socket_noninheritable, open_socket, set_nosigpipe};
use crate::ipc_address::IpcAddress;
use crate::ops::zmq_errno;
use crate::options::ZmqOptions;
use crate::socket_base::ZmqSocketBase;

// #include "ipc_address.hpp"
// #include "io_thread.hpp"
// #include "config.hpp"
// #include "err.hpp"
// #include "ip.hpp"
// #include "socket_base.hpp"
// #include "address.hpp"
#[derive(Debug, Clone, Default)]
pub struct IpcListener<'a>
{
    // : public stream_listener_base_t
    pub stream_listener_base: StreamListenerBase,

    //  True, if the underlying file for UNIX domain socket exists.
    pub _has_file: bool,

    //  Name of the temporary directory (if any) that has the
    //  UNIX domain socket
    pub _tmp_socket_dirname: String,

    //  Name of the file associated with the UNIX domain address.
    pub _filename: String,

    pub options: &'a ZmqOptions,

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (IpcListener)
}

impl IpcListener {
    //
    // IpcListener (ZmqIoThread *io_thread_,
    //             socket: *mut ZmqSocketBase,
    //             options: &ZmqOptions);
    pub fn new(io_thread: &mut ZmqIoThread,
               socket: &mut ZmqSocketBase,
               options: &ZmqOptions) -> Self
    {
        // :
        //     stream_listener_base_t (io_thread_, socket, options_), _has_file (false)
        Self {
            stream_listener_base: StreamListenerBase::new(io_thread, socket, options),
            _has_file: false,
            _tmp_socket_dirname: String::new(),
            _filename: String::new(),
            options: options,
        }
    }

    //  Set address to listen on.
    // int set_local_address (addr_: &str);

    pub fn set_local_address(&mut self, addr_: &str) -> i32
    {
        //  Create addr on stack for auto-cleanup
        // std::string addr (addr_);

        //  Allow wildcard file
        if (self.options.use_fd == -1 && addr[0] == '*') {
            if (create_ipc_wildcard_address(_tmp_socket_dirname, addr) < 0) {
                return -1;
            }
        }

        //  Get rid of the file associated with the UNIX domain socket that
        //  may have been left behind by the previous run of the application.
        //  MUST NOT unlink if the FD is managed by the user, or it will stop
        //  working after the first client connects. The user will take care of
        //  cleaning up the file after the service is stopped.
        if (self.options.use_fd == -1) {
            unsafe { unlink(addr.c_str()) };
        }
        _filename.clear();

        //  Initialise the address structure.
        let mut address = IpcAddress::new();
        let mut rc = address.resolve(addr.c_str());
        if (rc != 0) {
            if (!_tmp_socket_dirname.empty()) {
                // We need to preserve errno to return to the user
                let tmp_errno: i32 = errno;
                unsafe { rmdir(_tmp_socket_dirname.c_str()) };
                _tmp_socket_dirname.clear();
                errno = tmp_errno;
            }
            return -1;
        }

        address.to_string(_endpoint);

        if (self.options.use_fd != -1) {
            _s = self.options.use_fd;
        } else {
            //  Create a listening socket.
            _s = open_socket(AF_UNIX as i32, SOCK_STREAM as i32, 0);
            if (_s == retired_fd) {
                if (!_tmp_socket_dirname.empty()) {
                    // We need to preserve errno to return to the user
                    let tmp_errno: i32 = errno;
                    ::rmdir(_tmp_socket_dirname.c_str());
                    _tmp_socket_dirname.clear();
                    errno = tmp_errno;
                }
                return -1;
            }

            //  Bind the socket to the file path.
            rc = unsafe {
                bind(_s, (address.addr()),
                     address.addrlen())
            };
            if (rc != 0) {
                // goto
                // error;
            }

            //  Listen for incoming connections.
            rc = unsafe { listen(self._s, self.options.backlog) };
            if (rc != 0) {
                // goto
                // error;
            }
        }

        self._filename = addr.clone();
        self._has_file = true;

        self._socket.event_listening(make_unconnected_bind_endpoint_pair(self._endpoint),
                                     self._s);
        return 0;

        // error:
        // let err: i32 = errno;
        // close ();
        // errno = err;
        // return -1;
    }


    // std::string get_socket_name (fd: ZmqFileDesc, SocketEnd socket_end_) const;
    pub fn get_socket_name(&mut self, fd: ZmqFileDesc, socket_end_: SocketEnd) -> String
    {
        return get_socket_name(fd, socket_end_).unwrap();
    }

    //
    //  Handlers for I/O events.
    // void in_event ();

    pub fn in_event(&mut self)
    {
        let fd = self.accept();

        //  If connection was reset by the peer in the meantime, just ignore it.
        //  TODO: Handle specific errors like ENFILE/EMFILE etc.
        if (fd == retired_fd) {
            self._socket.event_accept_failed(
                make_unconnected_bind_endpoint_pair(_endpoint), zmq_errno());
            return;
        }

        //  Create the engine object for this connection.
        create_engine(fd);
    }

    //  Filter new connections if the OS provides a mechanism to get
    //  the credentials of the peer process.  Called from accept().
// #if defined ZMQ_HAVE_SO_PEERCRED || defined ZMQ_HAVE_LOCAL_PEERCRED
//     bool filter (ZmqFileDesc sock_);
// #endif

    pub fn filter(&mut self, sock_: ZmqFileDesc) -> bool
    {
        if (self.options.ipc_uid_accept_filters.is_empty()
            && self.options.ipc_pid_accept_filters.is_empty()
            && self.options.ipc_gid_accept_filters.empty()) {
            return true;
        }

        // struct ucred cred;
        let mut cred: ucred = ucred::new();
        let mut size = mem::size_of::<cred>() as c_int;

        unsafe {
            if (getsockopt(sock_, SOL_SOCKET, SO_PEERCRED, &mut cred, &mut size)) {
                return false;
            }
        }
        if (self.options.ipc_uid_accept_filters.find(cred.uid)
            != self.options.ipc_uid_accept_filters.end()
            || self.options.ipc_gid_accept_filters.find(cred.gid)
            != self.options.ipc_gid_accept_filters.end()
            || self.options.ipc_pid_accept_filters.find(cred.pid)
            != self.options.ipc_pid_accept_filters.end()) {
            return true;
        }

        // const struct passwd *pw;
        let pw: passwd = passwd::new();
        // const struct group *gr;
        let gr: group = group::new();

        if (!(pw = getpwuid(cred.uid))) {
            return false;
        }

        // for (ZmqOptions::ipc_gid_accept_filters_t::const_iterator
        //     it = options.ipc_gid_accept_filters.begin (),
        //     end = options.ipc_gid_accept_filters.end ();
        //     it != end; it+= 1)
        for opt in self.options.ipc_gid_accept_filters.iter()
        {
            if (!(gr = getgrgid(opt))) {
                continue;
            }
            // for (char **mem = gr.gr_mem; *mem; mem+= 1)
            for mem in gr.gr_mem.iter()
            {
                // if (!strcmp (*mem, pw.pw_name))
                if mem != pw.pw_name
                {
                    return true;
                }
            }
        }
        return false;
    }


    // int close ();

    pub fn close(&mut self) -> i32
    {
        // zmq_assert (_s != retired_fd);
        let mut fd_for_event = self._s;
// #ifdef ZMQ_HAVE_WINDOWS
        let mut rc = unsafe { closesocket(_s) };
        wsa_assert(rc != SOCKET_ERROR);
// #else
        unsafe { rc = close(_s); }
        // errno_assert (rc == 0);
// #endif

        self._s = retired_fd;

        if (_has_file && self.options.use_fd == -1) {
            if (!_tmp_socket_dirname.empty()) {
                //  TODO review this behaviour, it is inconsistent with the use of
                //  unlink in open since 656cdb959a7482c45db979c1d08ede585d12e315;
                //  however, we must at least remove the file before removing the
                //  directory, otherwise it will always fail
                unsafe { rc = unlink(_filename.c_str()); }

                if (rc == 0) {
                    rc = ::rmdir(_tmp_socket_dirname.c_str());
                    _tmp_socket_dirname.clear();
                }
            }

            if (rc != 0) {
                self._socket.event_close_failed(
                    make_unconnected_bind_endpoint_pair(_endpoint), zmq_errno());
                return -1;
            }
        }

        self._socket.event_closed(make_unconnected_bind_endpoint_pair(_endpoint),
                                  fd_for_event);
        return 0;
    }


    //  Accept the new connection. Returns the file descriptor of the
    //  newly created connection. The function may return retired_fd
    //  if the connection was dropped while waiting in the listen backlog.
    // ZmqFileDesc accept ();

    pub fn accept(&mut self) -> ZmqFileDesc
    {
        //  Accept one connection and deal with different failure modes.
        //  The situation where connection cannot be accepted due to insufficient
        //  resources is considered valid and treated by ignoring the connection.
        // zmq_assert (_s != retired_fd);
// #if defined ZMQ_HAVE_SOCK_CLOEXEC && defined HAVE_ACCEPT4
        let mut sock = accept4(_s, null_mut(), null_mut(), SOCK_CLOEXEC);
        // #else
        // struct sockaddr_storage ss;
        let ss: sockaddr_storage = sockaddr_storage::new();
        // memset (&ss, 0, mem::size_of::<ss>());
// #if defined ZMQ_HAVE_HPUX || defined ZMQ_HAVE_VXWORKS
        let ss_len = mem::size_of::<ss>();
// #else
        let mut ss_len = mem::size_of::<ss>() as c_int;
// #endif

        let sock = unsafe { accept(_s, (&mut ss), &mut ss_len) };
// #endif
        unsafe {
            if (sock == retired_fd) {
// #if defined ZMQ_HAVE_WINDOWS
                let last_error: WSA_ERROR = WSAGetLastError();
                // wsa_assert(last_error == WSAEWOULDBLOCK || last_error == WSAECONNRESET
                //     || last_error == WSAEMFILE || last_error == WSAENOBUFS);
// #else
                // errno_assert (errno == EAGAIN || errno == EWOULDBLOCK || errno == EINTR
                // || errno == ECONNABORTED || errno == EPROTO
                //     || errno == ENFILE);
// #endif
                return retired_fd;
            }
        }

        make_socket_noninheritable(sock);

        // IPC accept() filters
// #if defined ZMQ_HAVE_SO_PEERCRED || defined ZMQ_HAVE_LOCAL_PEERCRED
        unsafe {
            if (!filter(sock)) {
                let rc = close(sock as c_int);
                // errno_assert (rc == 0);
                return retired_fd;
            }
        }
// #endif

        unsafe {
            if (set_nosigpipe(sock)) {
// #ifdef ZMQ_HAVE_WINDOWS
                let rc: i32 = closesocket(sock);
                // wsa_assert(rc != SOCKET_ERROR);
// #else
                let rc = close(sock as c_int);
                // errno_assert (rc == 0);
// #endif
                return retired_fd;
            }
        }

        return sock;
    }
}


// #if defined ZMQ_HAVE_SO_PEERCRED
//
// #elif defined ZMQ_HAVE_LOCAL_PEERCRED
//
// bool IpcListener::filter (ZmqFileDesc sock_)
// {
//     if (options.ipc_uid_accept_filters.is_empty()
//         && options.ipc_gid_accept_filters.empty ())
//         return true;
//
//     struct xucred cred;
//     socklen_t size = mem::size_of::<cred>();
//
//     if (getsockopt (sock_, 0, LOCAL_PEERCRED, &cred, &size))
//         return false;
//     if (cred.cr_version != XUCRED_VERSION)
//         return false;
//     if (options.ipc_uid_accept_filters.find (cred.cr_uid)
//         != options.ipc_uid_accept_filters.end ())
//         return true;
//     for (int i = 0; i < cred.cr_ngroups; i+= 1) {
//         if (options.ipc_gid_accept_filters.find (cred.cr_groups[i])
//             != options.ipc_gid_accept_filters.end ())
//             return true;
//     }
//
//     return false;
// }

// #endif

// #endif
