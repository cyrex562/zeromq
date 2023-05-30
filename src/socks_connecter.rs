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
// #include <new>
// #include <string>

// #include "macros.hpp"
// #include "socks_connecter.hpp"
// #include "random.hpp"
// #include "err.hpp"
// #include "ip.hpp"
// #include "tcp.hpp"
// #include "address.hpp"
// #include "tcp_address.hpp"
// #include "session_base.hpp"
// #include "socks.hpp"

use std::ptr::null_mut;
use bincode::options;
use libc::{atoi, close, EHOSTUNREACH, EINPROGRESS, EINTR, EINVAL, ENETDOWN, ENETUNREACH, ETIMEDOUT, getsockopt, uint16_t};
use serde::__private::de::Content::String;
use windows::Win32::Networking::WinSock::{SO_ERROR, socklen_t, SOL_SOCKET, WSA_ERROR, WSAEACCES, WSAEADDRINUSE, WSAECONNABORTED, WSAECONNREFUSED, WSAEHOSTUNREACH, WSAEINPROGRESS, WSAEINVAL, WSAENETDOWN, WSAENETUNREACH, WSAETIMEDOUT, WSAEWOULDBLOCK, WSAGetLastError};
use crate::address::{ZmqAddress, get_socket_name};
use crate::address::SocketEnd::SocketEndLocal;
use crate::command::CommandType::bind;
use crate::err::wsa_error_to_errno;
use crate::fd::ZmqFileDesc;
use crate::io_thread::ZmqIoThread;
use crate::ip::unblock_socket;
use crate::ops::zmq_errno;
use crate::options::ZmqOptions;
use crate::session_base::ZmqSessionBase;
use crate::socks::{ZmqSocksAuthResponseDecoder, ZmqSocksBasicAuthRequestEncoder, ZmqSocksChoiceDecoder, ZmqSocksGreetingEncoder, ZmqSocksRequestEncoder, ZmqSocksResponseDecoder};
use crate::socks_connecter::ZmqSocksConnectorState::{sending_basic_auth_request, sending_greeting, sending_request, unplugged, waiting_for_auth_response, waiting_for_choice, waiting_for_proxy_connection, waiting_for_response};
use crate::stream_connecter_base::StreamConnecterBase;
use crate::tcp_address::TcpAddress;
use crate::zmtp_engine::ZmqIoThread;

// #ifndef ZMQ_HAVE_WINDOWS
// #include <unistd.h>
// #include <sys/types.h>
// #include <sys/socket.h>
// #if defined ZMQ_HAVE_VXWORKS
// #include <sockLib.h>
// #endif
// #endif

pub enum ZmqSocksConnectorState
{
    unplugged,
    waiting_for_reconnect_time,
    waiting_for_proxy_connection,
    sending_greeting,
    waiting_for_choice,
    sending_basic_auth_request,
    waiting_for_auth_response,
    sending_request,
    waiting_for_response
}

// enum
// {
pub const socks_no_auth_required: u8 = 0x00;
pub const socks_basic_auth: u8 = 0x02;
pub const socks_no_acceptable_method: u8 = 0xff;
// };

#[derive(Default,Debug,Clone)]
pub struct ZmqSocksConnector<'a>
{
// : public StreamConnecterBase
    pub connector_base: StreamConnecterBase<'a>,

    //  If 'delayed_start' is true connecter first waits for a while,
    //  then starts connection process.
    // ZmqSocksConnector (ZmqIoThread *io_thread_,
    //                    ZmqSessionBase *session_,
    //                    options: &ZmqOptions,
    //                    Address *addr_,
    //                    Address *proxy_addr_,
    //                    delayed_start_: bool);
    // ~ZmqSocksConnector ();
    // void set_auth_method_basic (const std::string &username,
    //                             password: &str);
    // void set_auth_method_none ();
  //
    //  Method ID\
    //  Handlers for I/O events.
    // void in_event ();
    // void out_event ();
    //  Internal function to start the actual connection establishment.
    // void start_connecting ();
    // static int process_server_response (const ZmqSocksChoice &response_);
    // static int process_server_response (const ZmqSocksResponse &response_);
    // static int process_server_response (const ZmqSocksAuthResponse &response_);
    // static int parse_address (const std::string &address_,
    //                           std::string &hostname_,
    //                           uint16_t &port_);
    // int connect_to_proxy ();
    // void // error ();
    //  Open TCP connecting socket. Returns -1 in case of error,
    //  0 if connect was successful immediately. Returns -1 with
    //  EAGAIN errno if async connect was launched.
    // int open ();
    //  Get the file descriptor of newly created connection. Returns
    //  retired_fd if the connection was unsuccessful.
    // ZmqFileDesc check_proxy_connection () const;

    // ZmqSocksGreetingEncoder _greeting_encoder;
    pub _greeting_encoder: ZmqSocksGreetingEncoder,
    // ZmqSocksChoiceDecoder _choice_decoder;
    pub _choice_decoder: ZmqSocksChoiceDecoder,
    // ZmqSocksBasicAuthRequestEncoder _basic_auth_request_encoder;
    pub _basic_auth_request_encoder: ZmqSocksBasicAuthRequestEncoder,
    // ZmqSocksAuthResponseDecoder _auth_response_decoder;
    pub _auth_reply_decoder: ZmqSocksAuthResponseDecoder,
    // ZmqSocksRequestEncoder _request_encoder;
    pub _request_encoder: ZmqSocksRequestEncoder,
    // ZmqSocksResponseDecoder _response_decoder;
    pub _response_decoder: ZmqSocksResponseDecoder,
    //  SOCKS address; owned by this connecter.
    // Address *_proxy_addr;
    pub _proxy_addr: ZmqAddress<'a, TcpAddress>,
    // User defined authentication method
    pub _auth_method: i32,
    // Credentials for basic authentication
    pub _auth_username: String,
    pub _auth_password: String,
    pub _status: i32,

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqSocksConnector)
}

impl ZmqSocksConnector {
    pub fn new(io_thread_: &mut ZmqIoThread,
               session: &mut ZmqSessionBase,
               options: &ZmqOptions,
               addr_: &mut ZmqAddress<TcpAddress>,
               proxy_addr_: &mut ZmqAddress<TcpAddress>,
               delayed_start_: bool) -> ZmqSocksConnector {

        // StreamConnecterBase (
        //       io_thread_, session_, options_, addr_, delayed_start_),
        //     _proxy_addr (proxy_addr_),
        //     _auth_method (socks_no_auth_required),
        //     _status (unplugged)
        // zmq_assert (_addr.protocol == protocol_name::tcp);
        let mut out = Self {
            connector_base: StreamConnecterBase::new(io_thread_, session, options, addr_, delayed_start_),
            _greeting_encoder: Default::default(),
            _choice_decoder: Default::default(),
            _basic_auth_request_encoder: Default::default(),
            _auth_reply_decoder: Default::default(),
            _request_encoder: Default::default(),
            _response_decoder: Default::default(),
            _proxy_addr: proxy_addr_.clone(),
            _auth_method: 0,
            _auth_username: "".to_string(),
            _auth_password: "".to_string(),
            _status: 0,
        };
        out._proxy_addr.to_string (_endpoint);
        out
    }

    pub fn set_auth_method_none (&mut self)
    {
        self._auth_method = socks_no_auth_required as i32;
        self._auth_username.clear ();
        self._auth_password.clear ();
    }

    pub fn set_auth_method_basic (&mut self,
                                  username_: &str,
                                  password_: &str)
    {
        _auth_method = socks_basic_auth;
        _auth_username = username_;
        _auth_password = password_;
    }


    pub fn in_event (&mut self)
    {
        let mut expected_status = -1;
        // zmq_assert (_status != unplugged);

        if (_status == waiting_for_choice) {
            let rc = _choice_decoder.input (_s);
            if (rc == 0 || rc == -1){}
            // error ();
            else if (_choice_decoder.message_ready ()) {
                let choice = _choice_decoder.decode ();
                rc = process_server_response (choice);
                if (rc == -1){}
                // error ();
                else {
                    if (choice.method == socks_basic_auth) {
                        expected_status = sending_basic_auth_request as i32;
                    }
                    else {
                        expected_status = sending_request as i32;
                    }
                }
            }
        } else if (_status == waiting_for_auth_response) {
            let rc = _auth_response_decoder.input (_s);
            if (rc == 0 || rc == -1){}
            // error ();
            else if (_auth_response_decoder.message_ready ()) {
                let auth_response =
                    _auth_response_decoder.decode ();
                rc = process_server_response (auth_response);
                if (rc == -1){}
                // error ();
                else {
                    expected_status = sending_request as i32;
                }
            }
        } else if (_status == waiting_for_response) {
            let rc = _response_decoder.input (_s);
            if (rc == 0 || rc == -1) {}
            // error ();
            else if (_response_decoder.message_ready ()) {
                let response = _response_decoder.decode ();
                rc = process_server_response (response);
                if (rc == -1){}
                // error ();
                else {
                    rm_handle ();
                    create_engine (
                        _s, get_socket_name(_s, SocketEndLocal));
                    _s = -1;
                    _status = unplugged;
                }
            }
        } else{}
        // error ();

        if (expected_status == sending_basic_auth_request) {
            _basic_auth_request_encoder.encode (
                ZmqSocksBasicAuthRequest (_auth_username, _auth_password));
            reset_pollin (_handle);
            set_pollout (_handle);
            _status = sending_basic_auth_request;
        } else if (expected_status == sending_request) {
            let mut hostname = String::new();
            let mut port = 0;
            if (parse_address (_addr.address, hostname, port) == -1){}
            // error ();
            else {
                _request_encoder.encode (ZmqSocksRequest (1, hostname, port));
                reset_pollin (_handle);
                set_pollout (_handle);
                _status = sending_request;
            }
        }
    }


    pub fn out_event (&mut self)
    {
        // zmq_assert (
        // _status == waiting_for_proxy_connection || _status == sending_greeting
        //     || _status == sending_basic_auth_request || _status == sending_request);

        if (_status == waiting_for_proxy_connection) {
            let rc: i32 =  (check_proxy_connection ());
            if (rc == -1){}
            // error ();
            else {
                _greeting_encoder.encode (ZmqSocksGreeting (_auth_method));
                _status = sending_greeting;
            }
        } else if (_status == sending_greeting) {
            // zmq_assert (_greeting_encoder.has_pending_data ());
            let rc: i32 = _greeting_encoder.output (_s);
            if (rc == -1 || rc == 0){}
            // error ();
            else if (!_greeting_encoder.has_pending_data ()) {
                reset_pollout (_handle);
                set_pollin (_handle);
                _status = waiting_for_choice;
            }
        } else if (_status == sending_basic_auth_request) {
            // zmq_assert (_basic_auth_request_encoder.has_pending_data ());
            let rc: i32 = _basic_auth_request_encoder.output (_s);
            if (rc == -1 || rc == 0){}
            // error ();
            else if (!_basic_auth_request_encoder.has_pending_data ()) {
                reset_pollout (_handle);
                set_pollin (_handle);
                _status = waiting_for_auth_response;
            }
        } else {
            // zmq_assert (_request_encoder.has_pending_data ());
            let rc: i32 = _request_encoder.output (_s);
            if (rc == -1 || rc == 0){}
            // error ();
            else if (!_request_encoder.has_pending_data ()) {
                reset_pollout (_handle);
                set_pollin (_handle);
                _status = waiting_for_response;
            }
        }
    }

    pub fn start_connecting (&mut self)
    {
        // zmq_assert (_status == unplugged);

        //  Open the connecting socket.
        let rc: i32 = connect_to_proxy ();

        //  Connect may succeed in synchronous manner.
        if (rc == 0) {
            _handle = add_fd (_s);
            set_pollout (_handle);
            _status = sending_greeting;
        }
        //  Connection establishment may be delayed. Poll for its completion.
        else if (errno == EINPROGRESS) {
            _handle = add_fd (_s);
            set_pollout (_handle);
            _status = waiting_for_proxy_connection;
            self._socket.event_connect_delayed (
                make_unconnected_connect_endpoint_pair (_endpoint), zmq_errno ());
        }
        //  Handle any other error condition by eventual reconnect.
        else {
            if (_s != retired_fd) {
                // TODO
                // close();
            }
            add_reconnect_timer ();
        }
    }

    pub fn process_server_response ( &mut self,
                                     response_: &ZmqSocksChoice) -> i32
    {
        return if response_.method == socks_no_auth_required
            || response_.method == socks_basic_auth
        { 0 }
        else { -1 };
    }

    pub fn process_server_response2 (
        response_: &ZmqSocksResponse) -> i32
    {
        return if response_.response_code == 0 { 0 } else { -1 };
    }

    pub fn process_server_response3 (
        &mut self, response_: &ZmqSocksAuthResponse) -> i32
    {
        return if response_.response_code == 0 { 0 } else { -1 };
    }

    pub fn error (&mut self)
    {
        rm_fd (_handle);
        // close ();
        _greeting_encoder.reset ();
        _choice_decoder.reset ();
        _basic_auth_request_encoder.reset ();
        _auth_response_decoder.reset ();
        _request_encoder.reset ();
        _response_decoder.reset ();
        _status = unplugged;
        add_reconnect_timer ();
    }


    pub fn connect_to_proxy (&mut self) -> i32
    {
        // zmq_assert (_s == retired_fd);

        //  Resolve the address
        if (_proxy_addr.resolved.tcp_addr != null_mut()) {
            LIBZMQ_DELETE (_proxy_addr.resolved.tcp_addr);
        }

        _proxy_addr.resolved.tcp_addr =  TcpAddress ();
        // alloc_assert (_proxy_addr.resolved.tcp_addr);
        //  Automatic fallback to ipv4 is disabled here since this was the existing
        //  behaviour, however I don't see a real reason for this. Maybe this can
        //  be changed to true (and then the parameter can be removed entirely).
        _s = tcp_open_socket (_proxy_addr.address, options, false, false,
                              _proxy_addr.resolved.tcp_addr);
        if (_s == retired_fd) {
            //  TODO we should emit some event in this case!
            LIBZMQ_DELETE (_proxy_addr.resolved.tcp_addr);
            return -1;
        }
        // zmq_assert (_proxy_addr.resolved.tcp_addr != null_mut());

        // Set the socket to non-blocking mode so that we get async connect().
        unblock_socket (_s);

        let tcp_addr = _proxy_addr.resolved.tcp_addr;

        rc: i32;

        // Set a source address for conversations
        if (tcp_addr.has_src_addr ()) {
// #if defined ZMQ_HAVE_VXWORKS
//         rc = ::bind (_s, (sockaddr *) tcp_addr.src_addr (),
//                      tcp_addr.src_addrlen ());
// #else
//         rc = bind (_s, tcp_addr.src_addr (), tcp_addr.src_addrlen ());
// #endif
//         if (rc == -1) {
//             close ();
//             return -1;
//         }
        }

        //  Connect to the remote peer.
// #if defined ZMQ_HAVE_VXWORKS
//     rc = ::connect (_s, (sockaddr *) tcp_addr.addr (), tcp_addr.addrlen ());
// #else
//     rc = ::connect (_s, tcp_addr.addr (), tcp_addr.addrlen ());
// #endif
        //  Connect was successful immediately.
        // if (rc == 0)
        //     return 0;

        //  Translate error codes indicating asynchronous connect has been
        //  launched to a uniform EINPROGRESS.
// #ifdef ZMQ_HAVE_WINDOWS
        let last_error: WSA_ERROR = unsafe { WSAGetLastError() };
        if (last_error == WSAEINPROGRESS || last_error == WSAEWOULDBLOCK) {
            errno = EINPROGRESS;
        }
        else {
            // errno = wsa_error_to_errno (last_error);
            // close ();
        }
// #else
        if (errno == EINTR) {
            errno = EINPROGRESS;
        }
// #endif
        return -1;
    }


    pub fn check_proxy_connection (&mut self) -> ZmqFileDesc
    {
        //  Async connect has finished. Check whether an error occurred
        let mut err = 0;
// #if defined ZMQ_HAVE_HPUX || defined ZMQ_HAVE_VXWORKS
//         let mut len = 4;
// #else
        let mut len = 4;
// #endif

        // TODO
        // let rc = unsafe {
        //     getsockopt(_s, SOL_SOCKET, SO_ERROR,
        //                (&err), &len)
        // };

        //  Assert if the error was caused by 0MQ bug.
        //  Networking problems are OK. No need to assert.
// #ifdef ZMQ_HAVE_WINDOWS
        // zmq_assert (rc == 0);
        if (err != 0) {
            wsa_assert (err == WSAECONNREFUSED || err == WSAETIMEDOUT
                || err == WSAECONNABORTED || err == WSAEHOSTUNREACH
                || err == WSAENETUNREACH || err == WSAENETDOWN
                || err == WSAEACCES || err == WSAEINVAL
                || err == WSAEADDRINUSE);
            return -1;
        }
// #else
        //  Following code should handle both Berkeley-derived socket
        //  implementations and Solaris.
        if (rc == -1) {
            err = errno;
        }
        if (err != 0) {
            errno = err;
            // errno_assert (errno == ECONNREFUSED || errno == ECONNRESET
            // || errno == ETIMEDOUT || errno == EHOSTUNREACH
            //     || errno == ENETUNREACH || errno == ENETDOWN
            //     || errno == EINVAL);
            return -1;
        }
// #endif

        rc = tune_tcp_socket (_s);
        rc = rc
            | tune_tcp_keepalives (
            _s, options.tcp_keepalive, options.tcp_keepalive_cnt,
            options.tcp_keepalive_idle, options.tcp_keepalive_intvl);
        if (rc != 0) {
            return -1;
        }

        return 0;
    }

} // end of impl




















pub fn parse_address (address_: &str, hostname_: &mut str,
                      mut port_:u16) -> i32
{
    //  Find the ':' at end that separates address from the port number.
    let idx = address_.rfind (':');
    if (idx == std::string::npos) {
        errno = EINVAL;
        return -1;
    }

    //  Extract hostname
    if (idx < 2 || address_[0] != '[' || address_[idx - 1] != ']') {
        *hostname_ = address_.substr(0, idx);
    }
    else {
        *hostname_ = address_.substr(1, idx - 2);
    }

    //  Separate the hostname/port.
    let port_str = address_.substr (idx + 1);
    //  Parse the port number (0 is not a valid port).
    port_ = u16::from_str_radix (port_str, 10).unwrap();
    if (port_ == 0) {
        errno = EINVAL;
        return -1;
    }
    return 0;
}
