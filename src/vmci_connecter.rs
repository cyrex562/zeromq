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

// #include "vmci_connecter.hpp"

// #if defined ZMQ_HAVE_VMCI

// #include <new>

use crate::address::SocketEnd::SocketEndLocal;
use crate::address::{get_socket_address, SocketEnd, ZmqAddress};
use crate::defines::ZmqFileDesc;
use crate::defines::ZMQ_RECONNECT_STOP_CONN_REFUSED;
use crate::err::wsa_error_to_errno;
use crate::ip::unblock_socket;
use crate::ops::zmq_errno;

use crate::context::ZmqContext;
use crate::session_base::ZmqSessionBase;
use crate::stream_connecter_base::StreamConnecterBase;
use crate::thread_context::ZmqThreadContext;
use crate::v3_1_encoder::ZmqV31Encoder;
use crate::vmci::{tune_vmci_buffer_size, tune_vmci_connect_timeout, vmci_open_socket};
use crate::vmci_address::ZmqVmciAddress;
use bincode::options;
use libc::{
    c_char, close, connect, getsockopt, open, timeval, ECONNREFUSED, EINPROGRESS, EINTR, ENOBUFS,
    ENOTSOCK,
};
use std::ptr::null_mut;
use windows::Win32::Networking::WinSock::{
    socklen_t, WSAGetLastError, SOL_SOCKET, SO_ERROR, WSAEBADF, WSAEINPROGRESS, WSAENOBUFS,
    WSAENOPROTOOPT, WSAENOTSOCK, WSAEWOULDBLOCK,
};

// #include "io_thread.hpp"
// #include "platform.hpp"
// #include "random.hpp"
// #include "err.hpp"
// #include "ip.hpp"
// #include "address.hpp"
// #include "vmci_address.hpp"
// #include "vmci.hpp"
// #include "session_base.hpp"

// enum
// {
//     connect_timer_id = 2
// };
pub const connect_timer_id: i32 = 2;

pub struct ZmqVmciConnecter<'a> {
    // : public StreamConnecterBase
    pub stream_connecter_base: StreamConnecterBase<'a>,
    //  If 'delayed_start' is true connecter first waits for a while,
    //  then starts connection process.
    // ZmqVmciConnecter (ZmqIoThread *io_thread_,
    //                   ZmqSessionBase *session_,
    //                   options: &ZmqOptions,
    //                   Address *addr_,
    //                   delayed_start_: bool);
    // ~ZmqVmciConnecter ();
    // std::string get_socket_name (fd: ZmqFileDesc, SocketEnd socket_end_) const;
    //  ID of the timer used to check the connect timeout, must be different from stream_connecter_base_t::reconnect_timer_id.
    //  Handlers for incoming commands.
    // void process_term (linger: i32);
    //  Handlers for I/O events.
    // void in_event ();
    // void out_event ();
    // void timer_event (id_: i32);
    //  Internal function to start the actual connection establishment.
    // void start_connecting ();
    //  Internal function to add a connect timer
    // void add_connect_timer ();
    //  Internal function to return a reconnect backoff delay.
    //  Will modify the current_reconnect_ivl used for next call
    //  Returns the currently used interval
    // int get_new_reconnect_ivl ();
    //  Open VMCI connecting socket. Returns -1 in case of error,
    //  0 if connect was successful immediately. Returns -1 with
    //  EAGAIN errno if async connect was launched.
    // int open ();
    //  Get the file descriptor of newly created connection. Returns
    //  retired_fd if the connection was unsuccessful.
    // ZmqFileDesc connect ();
    //  True iff a timer has been started.
    pub _connect_timer_started: bool,
    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqVmciConnecter)
}

impl ZmqVmciConnecter {
    // ZmqVmciConnecter::ZmqVmciConnecter (class ZmqIoThread *io_thread_,
    // pub struct ZmqSessionBase *session_,
    //                                          options: &ZmqOptions,
    //                                          Address *addr_,
    //                                          delayed_start_: bool) :
    //     StreamConnecterBase (
    //       io_thread_, session_, options_, addr_, delayed_start_),
    //     _connect_timer_started (false)
    // {
    //     // zmq_assert (_addr.protocol == protocol_name::vmci);
    // }
    pub fn new(
        io_thread_: &mut ZmqThreadContext,
        session_: &mut ZmqSessionBase,
        ctx: &mut ZmqContext,
        addr: &mut ZmqVmciAddress,
        delayed_start: bool,
    ) -> Self {
        let mut out = Self {
            stream_connecter_base: Default::default(),
            _connect_timer_started: false,
        };

        out.stream_connecter_base =
            StreamConnecterBase::new(io_thread_, session_, ctx, addr, delayed_start);
        out._connect_timer_started = false;
        out
    }

    pub fn process_term(&mut self, linger: i32) {
        if (_connect_timer_started) {
            cancel_timer(connect_timer_id);
            _connect_timer_started = false;
        }

        self.stream_connecter_base.process_term(linger);
    }

    pub fn in_event(&mut self) {
        //  We are not polling for incoming data, so we are actually called
        //  because of error here. However, we can get error on out event as well
        //  on some platforms, so we'll simply handle both events in the same way.
        out_event();
    }

    pub fn out_event(&mut self) {
        if (_connect_timer_started) {
            cancel_timer(connect_timer_id);
            _connect_timer_started = false;
        }

        //  TODO this is still very similar to (t)ipc_connecter_t, maybe the
        //  differences can be factored out

        rm_handle();

        let mut fd = self.connect();

        if (fd == retired_fd
            && ((options.reconnect_stop & ZMQ_RECONNECT_STOP_CONN_REFUSED)
                && errno == ECONNREFUSED))
        {
            send_conn_failed(_session);
            self.close();
            terminate();
            return;
        }

        //  Handle the error condition by attempt to reconnect.
        if (fd == retired_fd) {
            self.close();
            add_reconnect_timer();
            return;
        }

        tune_vmci_buffer_size(
            this.get_ctx(),
            &mut fd,
            options.vmci_buffer_size,
            options.vmci_buffer_min_size,
            options.vmci_buffer_max_size,
        );

        if (options.vmci_connect_timeout > 0) {
            // #if defined ZMQ_HAVE_WINDOWS
            tune_vmci_connect_timeout(this.get_ctx(), &mut fd, options.vmci_connect_timeout);
            // #else
            //         let timeout = timeval{ tv_sec: 0, tv_usec: options.vmci_connect_timeout * 1000 };
            //         tune_vmci_connect_timeout (this.get_ctx (), &mut fd, timeout.tv_usec as u32);
            // #endif
        }

        create_engine(fd, self.connecter.get_socket_name(fd, SocketEndLocal));
    }

    pub fn get_socket_name(&mut self, fd: ZmqFileDesc, socket_end_: SocketEnd) -> String {
        // TODO
        // struct sockaddr_storage ss;
        // const ZmqSocklen sl = get_socket_address (fd, socket_end_, &ss);
        // if (sl == 0) {
        //     return std::string ();
        // }

        let mut addr = ZmqVmciAddress::new3(&ss, sl, self.stream_connecter_base.ctx);
        let mut address_string: String = String::new();
        addr.to_string(&address_string);
        return address_string;
    }

    pub fn timer_event(&mut self, id_: i32) {
        if (id_ == connect_timer_id) {
            _connect_timer_started = false;
            rm_handle();
            self.close();
            add_reconnect_timer();
        } else {
            self.stream_connecter_base.timer_event(id_);
        }
    }

    pub fn start_connecting(&mut self) {
        //  Open the connecting socket.
        let rc: i32 = self.open();

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

            //  add userspace connect timeout
            add_connect_timer();
        }
        //  Handle any other error condition by eventual reconnect.
        else {
            if (_s != retired_fd) {
                self.close();
            }
            add_reconnect_timer();
        }
    }

    pub fn add_connect_timer(&mut self) {
        if (options.connect_timeout > 0) {
            add_timer(options.connect_timeout, connect_timer_id);
            _connect_timer_started = true;
        }
    }

    pub fn open(&mut self) -> i32 {
        // zmq_assert (_s == retired_fd);

        //  Resolve the address
        if (_addr.resolved.vmci_addr != null_mut()) {
            LIBZMQ_DELETE(_addr.resolved.vmci_addr);
        }

        _addr.resolved.vmci_addr = ZmqVmciAddress(this.get_ctx());
        // alloc_assert (_addr.resolved.vmci_addr);
        _s = vmci_open_socket(_addr.address, self.options, _addr.resolved.vmci_addr);
        if (_s == retired_fd) {
            //  TODO we should emit some event in this case!

            LIBZMQ_DELETE(_addr.resolved.vmci_addr);
            return -1;
        }
        // zmq_assert (_addr.resolved.vmci_addr != null_mut());

        // Set the socket to non-blocking mode so that we get async connect().
        unblock_socket(_s);

        let vmci_addr = _addr.resolved.vmci_addr;

        let mut rc: i32;

        //  Connect to the remote peer.
        // #if defined ZMQ_HAVE_VXWORKS
        //     rc = connect (_s, (sockaddr *) vmci_addr.addr (), vmci_addr.addrlen ());
        // #else
        unsafe {
            rc = connect(_s, vmci_addr.addr(), vmci_addr.addrlen());
        }
        // #endif
        //  Connect was successful immediately.
        if (rc == 0) {
            return 0;
        }

        //  Translate error codes indicating asynchronous connect has been
        //  launched to a uniform EINPROGRESS.
        // #ifdef ZMQ_HAVE_WINDOWS
        let last_error: i32 = unsafe { WSAGetLastError() as i32 };
        if (last_error == WSAEINPROGRESS || last_error == WSAEWOULDBLOCK) {
          // errno = EINPROGRESS;
        } else {
            // errno = wsa_error_to_errno(last_error);
        }
        // #else
        if (errno == EINTR) {
          // errno = EINPROGRESS;
        }
        // #endif
        return -1;
    }

    pub fn connect(&mut self) -> ZmqFileDesc {
        //  Async connect has finished. Check whether an error occurred
        let mut err = 0;
        // #if defined ZMQ_HAVE_HPUX || defined ZMQ_HAVE_VXWORKS
        let mut len = 4;
        // #else
        let mut len = 4;
        // #endif

        let rc: i32 = unsafe {
            getsockopt(
                _s,
                SOL_SOCKET,
                SO_ERROR,
                (&mut err) as *mut c_char,
                &mut len,
            )
        };

        //  Assert if the error was caused by 0MQ bug.
        //  Networking problems are OK. No need to assert.
        // #ifdef ZMQ_HAVE_WINDOWS
        // zmq_assert (rc == 0);
        if (err != 0) {
            if (err == WSAEBADF || err == WSAENOPROTOOPT || err == WSAENOTSOCK || err == WSAENOBUFS)
            {
                wsa_assert_no(err);
            }
            // errno = wsa_error_to_errno (err);
            return retired_fd;
        }
        // #else
        //  Following code should handle both Berkeley-derived socket
        //  implementations and Solaris.
        if (rc == -1) {
            err = errno;
        }
        if (err != 0) {
          // errno = err;
            // #if !defined(TARGET_OS_IPHONE) || !TARGET_OS_IPHONE
            // errno_assert (errno != EBADF && errno != ENOPROTOOPT
            //               && errno != ENOTSOCK && errno != ENOBUFS);
            // #else
            // errno_assert (errno != ENOPROTOOPT && errno != ENOTSOCK
            //               && errno != ENOBUFS);
            // #endif
            return retired_fd;
        }
        // #endif

        //  Return the newly Connected socket.
        let result = _s;
        _s = retired_fd;
        return result;
    }
}

// #endif
