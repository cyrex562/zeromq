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
// #include "ipc_connecter.hpp"

// #if defined ZMQ_HAVE_IPC

// #include <new>
// #include <string>

// #include "io_thread.hpp"
// #include "random.hpp"
// #include "err.hpp"
// #include "ip.hpp"
// #include "address.hpp"
// #include "ipc_address.hpp"
// #include "session_base.hpp"

use crate::address::SocketEnd::SocketEndLocal;
use crate::address::{get_socket_name, ZmqAddress};
use crate::address_family::AF_UNIX;
use crate::defines::ZMQ_RECONNECT_STOP_AFTER_DISCONNECT;
use crate::err::wsa_error_to_errno;
use crate::defines::ZmqFileDesc;
use crate::ip::{open_socket, unblock_socket};
use crate::ops::zmq_errno;

use crate::session_base::ZmqSessionBase;
use crate::stream_connecter_base::StreamConnecterBase;
use crate::thread_context::ZmqThreadContext;
use libc::{
    c_char, close, connect, getsockopt, open, ECONNREFUSED, EHOSTUNREACH, EINPROGRESS, EINTR,
    ENETDOWN, ENETUNREACH, ENOPROTOOPT, ETIMEDOUT,
};
use windows::Win32::Networking::WinSock::{
    WSAGetLastError, SOCK_STREAM, SOL_SOCKET, SO_ERROR, WSAEINPROGRESS, WSAEWOULDBLOCK,
};
use crate::context::ZmqContext;

// #ifdef _MSC_VER
// #include <afunix.h>
// #else
// #include <unistd.h>
// #include <sys/types.h>
// #include <sys/socket.h>
// #include <sys/un.h>
// #endif
pub struct IpcConnecter<'a> {
    // : public stream_connecter_base_t
    pub stream_connecter_base: StreamConnecterBase<'a>,
}

impl IpcConnecter {
    //
    //  If 'delayed_start' is true connecter first waits for a while,
    //  then starts connection process.
    // IpcConnecter (ZmqIoThread *io_thread_,
    //                 ZmqSessionBase *session_,
    //              options: &ZmqOptions,
    //                 Address *addr_,
    //              delayed_start_: bool);
    pub fn new(
        ctx: &mut ZmqContext,
        io_thread_: &mut ZmqThreadContext,
        session: &mut ZmqSessionBase,
        addr: &mut ZmqAddress,
        delayed_start_: bool,
    ) -> Self {
        // stream_connecter_base_t (
        //       io_thread_, session_, options_, addr_, delayed_start_)
        // zmq_assert (_addr.protocol == protocol_name::ipc);
        Self {
            stream_connecter_base: Default::default(),
        }
    }

    //
    //  Handlers for I/O events.
    // void out_event ();
    pub fn out_event(&mut self) {
        // let fd = unsafe { connect() };
        rm_handle();

        //  Handle the error condition by attempt to reconnect.
        if (fd == retired_fd) {
            unsafe { close(fd) };
            add_reconnect_timer();
            return;
        }

        create_engine(fd, get_socket_name(fd as ZmqFileDesc, SocketEndLocal));
    }

    //  Internal function to start the actual connection establishment.
    // void start_connecting ();

    pub fn start_connecting(&mut self) {
        //  Open the connecting socket.
        // let rc: i32 = unsafe { open() };

        //  Connect may succeed in synchronous manner.
        if (rc == 0) {
            _handle = add_fd(_s);
            out_event();
        }
        //  Connection establishment may be delayed. Poll for its completion.
        else if (rc == -1 && errno == EINPROGRESS) {
            _handle = add_fd(_s);
            set_pollout(_handle);
            self._socket.event_connect_delayed(
                make_unconnected_connect_endpoint_pair(_endpoint),
                zmq_errno(),
            );

            // TODO, ZmqTcpConnector adds a connect timer in this case; maybe this
            // should be Done here as well (and then this could be pulled up to
            // stream_connecter_base_t).
        }
        //Stop connecting after called zmq_disconnect
        else if (rc == -1
            && (self.options.reconnect_stop & ZMQ_RECONNECT_STOP_AFTER_DISCONNECT)
            && errno == ECONNREFUSED
            && self._socket.is_disconnected())
        {
            if (_s != retired_fd) {
                // unsafe { close() };
            }
        }
        //  Handle any other error condition by eventual reconnect.
        else {
            if (_s != retired_fd) {
                // close();
            }
            add_reconnect_timer();
        }
    }

    pub fn open(&mut self) -> i32 {
        // zmq_assert (_s == retired_fd);

        //  Create the socket.
        self._s = open_socket(AF_UNIX as i32, SOCK_STREAM as i32, 0);
        if (_s == retired_fd) {
            return -1;
        }

        //  Set the non-blocking flag.
        unblock_socket(_s);

        //  Connect to the remote peer.
        let rc: i32 = ::connect(
            _s,
            _addr.resolved.ipc_addr.addr(),
            _addr.resolved.ipc_addr.addrlen(),
        );

        //  Connect was successful immediately.
        if (rc == 0) {
            return 0;
        }

        //  Translate other error codes indicating asynchronous connect has been
        //  launched to a uniform EINPROGRESS.
        // #ifdef ZMQ_HAVE_WINDOWS
        if cfg!(target_os = "windows") {
            let last_error = unsafe { WSAGetLastError() };
            if (last_error == WSAEINPROGRESS || last_error == WSAEWOULDBLOCK) {
                errno = EINPROGRESS;
            } else {
                errno = wsa_error_to_errno(last_error);
            }
        }
        // #else
        else {
            if (rc == -1 && errno == EINTR) {
                errno = EINPROGRESS;
            }
        }
        // #endif

        //  Forward the error.
        return -1;
    }

    //  Open IPC connecting socket. Returns -1 in case of error,
    //  0 if connect was successful immediately. Returns -1 with
    //  EAGAIN errno if async connect was launched.
    // int open ();

    //  Get the file descriptor of newly created connection. Returns
    //  retired_fd if the connection was unsuccessful.
    // ZmqFileDesc connect ();
    pub fn connect(&mut self) {
        //  Following code should handle both Berkeley-derived socket
        //  implementations and Solaris.
        let mut err = 0;
        let mut len = 4;
        let rc: i32 = unsafe {
            getsockopt(
                _s,
                SOL_SOCKET,
                SO_ERROR,
                (&mut err.to_le_bytes() as &mut c_char),
                &mut len,
            )
        };
        if (rc == -1) {
            if (errno == ENOPROTOOPT) {
                errno = 0;
            }
            err = errno;
        }
        if (err != 0) {
            //  Assert if the error was caused by 0MQ bug.
            //  Networking problems are OK. No need to assert.
            errno = err;
            // errno_assert (errno == ECONNREFUSED || errno == ECONNRESET
            // || errno == ETIMEDOUT || errno == EHOSTUNREACH
            //     || errno == ENETUNREACH || errno == ENETDOWN);

            return retired_fd;
        }

        let result = _s;
        _s = retired_fd;
        return result;
    }

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ipc_connecter_t)
}

// #endif
