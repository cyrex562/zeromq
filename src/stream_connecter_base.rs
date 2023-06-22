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

use crate::address::ZmqAddress;
use crate::defines::ZmqHandle;
use crate::endpoint::EndpointType::endpoint_type_connect;
use crate::endpoint_uri::EndpointUriPair;
use crate::defines::ZmqFileDesc;
use crate::io_object::ZmqIoObject;

use crate::own::ZmqOwn;
use crate::proxy::ZmqSocket;
use crate::raw_engine::RawEngine;
use crate::session_base::ZmqSessionBase;
use crate::thread_context::ZmqThreadContext;
use crate::zmtp_engine::ZmqZmtpEngine;
use libc::{c_int, close};
use windows::Win32::Networking::WinSock::{closesocket, SOCKET_ERROR};
use crate::context::ZmqContext;

// enum
// {
//     reconnect_timer_id = 1
// };

#[derive(Default, Debug, Clone)]
pub struct StreamConnecterBase<'a> {
    // : public ZmqOwn, public ZmqIoObject
    pub own: ZmqOwn,
    pub io_object: ZmqIoObject,
    //  Address to connect to. Owned by ZmqSessionBase.
    //  It is non-const since some parts may change during opening.
    // Address *const _addr;
    pub _addr: &'a ZmqAddress(a),
    //  Underlying socket.
    // ZmqFileDesc _s;
    pub _s: ZmqFileDesc,
    //  Handle corresponding to the listening socket, if file descriptor is
    //  registered with the poller, or NULL.
    // handle_t _handle;
    pub _handle: Option<ZmqHandle>,
    // String representation of endpoint to connect to
    pub _endpoint: String,
    // Socket
    // ZmqSocketBase *const self._socket;
    pub _socket: &'a ZmqSocket,
    //  ID of the timer used to delay the reconnection.
    // virtual void start_connecting () = 0;
    pub start_connecting: Option<fn()>,
    //  If true, connecter is waiting a while before trying to connect.
    pub _delayed_start: bool,
    //  True iff a timer has been started.
    pub _reconnect_timer_started: bool,
    //  Current reconnect ivl, updated for backoff strategy
    pub _current_reconnect_ivl: i32,
    // ZMQ_NON_COPYABLE_NOR_MOVABLE (stream_connecter_base_t)
    //  Reference to the session we belong to.
    pub _session: &'a ZmqSessionBase,
}

impl StreamConnecterBase {
    //
    //  If 'delayed_start' is true connecter first waits for a while,
    //  then starts connection process.
    // StreamConnecterBase (ZmqIoThread *io_thread_,
    //                     ZmqSessionBase *session_,
    //                     options: &ZmqOptions,
    //                     Address *addr_,
    //                     delayed_start_: bool);
    pub fn new(
        io_thread_: &mut ZmqThreadContext,
        session_: &mut ZmqSessionBase,
        ctx: &ZmqContext,
        addr_: &mut ZmqAddress,
        delayed_start_: bool,
    ) -> Self {
        // ZmqOwn (io_thread_, options_),
        //     ZmqIoObject (io_thread_),
        //     _addr (addr_),
        //     _s (retired_fd),
        //     _handle (static_cast<handle_t> (null_mut())),
        //     self._socket (session_.get_socket ()),
        //     _delayed_start (delayed_start_),
        //     _reconnect_timer_started (false),
        //     _current_reconnect_ivl (options.reconnect_ivl),
        //     _session (session_)
        // zmq_assert (_addr);
        // _addr.to_string (_endpoint);
        // TODO the return value is unused! what if it fails? if this is impossible
        // or does not matter, change such that endpoint in initialized using an
        // initializer, and make endpoint const
        Self {
            own: ZmqOwn::default(),
            io_object: ZmqIoObject::new(Some(io_thread_.clone())),
            _addr: &Default::default(),
            _s: 0,
            _handle: None,
            _endpoint: "".to_string(),
            _socket: session_.get_socket(),
            start_connecting: None,
            _delayed_start: delayed_start_,
            _reconnect_timer_started: false,
            _current_reconnect_ivl: ctx.reconnect_ivl,
            _session: session_,
        }
    }

    // ~StreamConnecterBase () ;
    // StreamConnecterBase::~StreamConnecterBase ()
    // {
    // // zmq_assert (!_reconnect_timer_started);
    // // zmq_assert (!_handle);
    // // zmq_assert (_s == retired_fd);
    // }

    //  Handlers for incoming commands.
    // void process_plug () ;
    pub fn process_plug(&mut self) {
        if (_delayed_start) {
            self.add_reconnect_timer();
        } else {
            self.start_connecting();
        }
    }

    // void process_term (linger: i32) ;
    pub fn process_term(&mut self, linger: i32) {
        if (self._reconnect_timer_started) {
            self.cancel_timer(self.reconnect_timer_id);
            self._reconnect_timer_started = false;
            if (_handle) {
                rm_handle();
            }
        }

        if (self._s != retired_fd) {
            self.close();
        }

        self.own.process_term(linger);
    }

    //  Handlers for I/O events.
    // void in_event () ;

    // void timer_event (id_: i32) ;

    //  Internal function to create the engine after connection was established.
    // virtual void create_engine (fd: ZmqFileDesc, local_address_: &str);

    //  Internal function to add a reconnect timer
    // void add_reconnect_timer ();
    pub fn add_reconnect_timer(&mut self) {
        if (self.options.reconnect_ivl > 0) {
            let interval: i32 = self.get_new_reconnect_ivl();
            self.add_timer(interval, reconnect_timer_id);
            self._socket.event_connect_retried(
                self.make_unconnected_connect_endpoint_pair(_endpoint),
                interval,
            );
            self._reconnect_timer_started = true;
        }
    }

    //  Removes the handle from the poller.
    // void rm_handle ();
    pub fn rm_handle(&mut self) {
        self.rm_fd(self._handle);
        self._handle = None;
    }

    //  Close the connecting socket.
    // void close ();
    pub fn close(&mut self) {
        // TODO before, this was an assertion for _s != retired_fd, but this does not match usage of close
        if (self._s != retired_fd) {
            // #ifdef ZMQ_HAVE_WINDOWS
            let rc: i32 = unsafe { closesocket(_s) };
            wsa_assert(rc != SOCKET_ERROR);
            // #else
            let rc: i32 = unsafe { close(self._s as c_int) };
            // errno_assert (rc == 0);
            // #endif
            self._socket
                .event_closed(make_unconnected_connect_endpoint_pair(_endpoint), _s);
            _s = retired_fd;
        }
    }

    //  Internal function to return a reconnect backoff delay.
    //  Will modify the current_reconnect_ivl used for next call
    //  Returns the currently used interval
    // int get_new_reconnect_ivl ();
    pub fn get_new_reconnect_ivl(&mut self) -> i32 {
        //  TODO should the random jitter be really based on the configured initial
        //  reconnect interval options.reconnect_ivl, or better on the
        //  _current_reconnect_ivl?

        //  The new interval is the current interval + random value.
        let random_jitter: i32 = generate_random() % self.options.reconnect_ivl;
        let interval: i32 = if self._current_reconnect_ivl < i32::MAX - random_jitter {
            self._current_reconnect_ivl + random_jitter
        } else {
            i32::MAX
        };

        //  Only change the new current reconnect interval if the maximum reconnect
        //  interval was set and if it's larger than the reconnect interval.
        if (self.options.reconnect_ivl_max > 0
            && self.options.reconnect_ivl_max > self.options.reconnect_ivl)
        {
            //  Calculate the next interval
            self._current_reconnect_ivl = if self._current_reconnect_ivl < i32::MAX / 2 {
                i32::min(
                    self._current_reconnect_ivl * 2,
                    self.options.reconnect_ivl_max,
                )
            } else {
                self.options.reconnect_ivl_max
            };
        }

        return interval;
    }

    pub fn in_event(&mut self) {
        //  We are not polling for incoming data, so we are actually called
        //  because of error here. However, we can get error on out event as well
        //  on some platforms, so we'll simply handle both events in the same way.
        self.out_event();
    }

    pub fn create_engine(&mut self, fd: ZmqFileDesc, local_address_: &str) {
        let mut endpoint_pair =
            EndpointUriPair::new(local_address_, _endpoint, endpoint_type_connect);

        //  Create the engine object for this connection.
        let mut engine = ZmqEngine::default();
        if (self.options.raw_socket) {
            engine = RawEngine::new(fd, self.options, endpoint_pair);
        } else {
            engine = ZmqZmtpEngine::new(fd, self.options, endpoint_pair);
        }
        // alloc_assert (engine);

        //  Attach the engine to the corresponding session object.
        send_attach(_session, engine);

        //  Shut the connecter down.
        terminate();

        self._socket.event_connected(&endpoint_pair, fd);
    }

    pub fn timer_event(&mut self, id_: i32) {
        // zmq_assert (id_ == reconnect_timer_id);
        self._reconnect_timer_started = false;
        self.start_connecting();
    }
}
