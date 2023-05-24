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
// #include <sys/types.h>

// #include "err.hpp"
// #include "socks.hpp"
// #include "tcp.hpp"
// #include "blob.hpp"

// #ifndef ZMQ_HAVE_WINDOWS
// #include <sys/socket.h>
// #include <netinet/in.h>
// #include <netdb.h>
// #endif

use std::ptr::null_mut;
use dns_lookup::getaddrinfo;
use libc::{memcpy, memset, size_t, uint8_t};
use windows::Win32::Networking::WinSock::{AI_NUMERICHOST, freeaddrinfo};
use crate::address_family::{AF_INET, AF_INET6};
use crate::fd::ZmqFileDesc;
use crate::unix_sockaddr::addrinfo;
use crate::utils::{copy_bytes, advance_ptr};

#[derive(Default, Debug, Clone)]
struct ZmqSocksGreeting {
    // ZmqSocksGreeting (uint8_t method_);
    // ZmqSocksGreeting (methods_: &[u8], uint8_t num_methods_);

    // uint8_t methods[UINT8_MAX];
    pub methods: [u8; u8::MAX as usize],
    pub num_methods: usize,
}

#[derive(Default, Debug, Clone)]
pub struct ZmqSocksGreetingEncoder {
    //
//     ZmqSocksGreetingEncoder ();
//     void encode (const ZmqSocksGreeting &greeting_);
//     int output (ZmqFileDesc fd);
//     bool has_pending_data () const;
//     void reset ();
    //
    pub _bytes_encoded: usize,
    pub _bytes_written: usize,
    pub buf: [u8; (2 + u8::MAX) as usize],
}

#[derive(Default, Debug, Clone)]
struct ZmqSocksChoice {
    // ZmqSocksChoice (uint8_t method_);
    // uint8_t method;
    pub method: u8,
}


#[derive(Default, Debug, Clone)]
pub struct ZmqSocksChoiceDecoder {
//
//     ZmqSocksChoiceDecoder ();
//     int input (ZmqFileDesc fd);
//     bool message_ready () const;
//     ZmqSocksChoice decode ();
//     void reset ();

    //
    pub buf: [u8; 2],
    pub _bytes_read: usize,
}

#[derive(Default, Debug, Clone)]
struct ZmqSocksBasicAuthRequest {
    // ZmqSocksBasicAuthRequest (const std::string &username_,
    //                             password_: &str);

    // const username: String;
    pub username: String,
    // const password: String;
    pub password: String,
}

#[derive(Default, Debug, Clone)]
pub struct ZmqSocksBasicAuthRequestEncoder {
//
//     ZmqSocksBasicAuthRequestEncoder ();
//     void encode (const ZmqSocksBasicAuthRequest &req_);
//     int output (ZmqFileDesc fd);
//     bool has_pending_data () const;
//     void reset ();

    //
    pub _bytes_encoded: usize,
    pub _bytes_written: usize,
    pub buf: [u8; 1 + 1 + UINT8_MAX + 1 + UINT8_MAX],
}

#[derive(Default, Debug, Clone)]
struct ZmqSocksAuthResponse {
    // ZmqSocksAuthResponse (uint8_t response_code_);
    // uint8_t response_code;
    pub response_code: u8,
}

#[derive(Default, Debug, CLone)]
pub struct ZmqSocksAuthResponseDecoder {
//
//     ZmqSocksAuthResponseDecoder ();
//     int input (ZmqFileDesc fd);
//     bool message_ready () const;
//     ZmqSocksAuthResponse decode ();
//     void reset ();

    //
    //   int8_t buf[2];
    pub buf: [i8; 2],
    pub _bytes_read: usize,
}

#[derive(Default, Debug, Clone)]
struct ZmqSocksRequest {
    // ZmqSocksRequest (uint8_t command_, std::string hostname_, uint16_t port_);

    // const uint8_t command;
    pub command: u8,
    // const hostname: String;
    pub hostname: String,
    // const uint16_t port;
    pub port: u16,
}

#[derive(Default, Debug, Clone)]
pub struct ZmqSocksRequestEncoder {
    //
//     ZmqSocksRequestEncoder ();
//     void encode (const ZmqSocksRequest &req_);
//     int output (ZmqFileDesc fd);
//     bool has_pending_data () const;
//     void reset ();
    //
    pub _bytes_encoded: usize,
    pub _bytes_written: usize,
    // uint8_t buf[4 + UINT8_MAX + 1 + 2];
    pub buf: [u8; (u8::MAX + 1 + 2) as usize],
}

#[derive(Default, Debug, Clone)]
struct ZmqSocksResponse {
    // ZmqSocksResponse (uint8_t response_code_,
    //                   const std::string &address_,
    //                   uint16_t port_);
    // uint8_t response_code;
    pub response_code: u8,
    // address: String;
    pub address: String,
    // uint16_t port;
    pub port: u16,
}

#[derive(Default, Debug, Clone)]
pub struct ZmqSocksResponseDecoder {
//
//     ZmqSocksResponseDecoder ();
//     int input (ZmqFileDesc fd);
//     bool message_ready () const;
//     ZmqSocksResponse decode ();
//     void reset ();

    //
    //   int8_t buf[4 + UINT8_MAX + 1 + 2];
    pub buf: [i8; (u8::MAX + 1 + 2) as usize],
    //   _bytes_read: usize;
    pub _bytes_read: usize,
}

impl ZmqSocksGreeting {
    pub fn new(method_: u8) -> Self {
        // : num_methods (1)
        methods[0] = method_;
        let mut out = Self {
            methods: [0; u8::MAX],
            num_methods: 1,
        };
        out.methods[0] = method_;
        out
    }

    pub fn new2(methods_: &[u8], num_methods_: u8) -> Self {
        // :
        //     num_methods (num_methods_)
        let mut out = Self {
            methods: [0; u8::MAX],
            num_methods: num_methods_ as usize,
        };
        // for (uint8_t i = 0; i < num_methods_; i+= 1)
        for i in 0..num_methods_ {
            out.methods[i] = methods_[i];
        }
        out
    }
}

impl ZmqSocksGreetingEncoder {
    pub fn new() -> Self {

        //  _bytes_encoded (0), _bytes_written (0)
        Self {
            buf: [0; u8::MAX],
            _bytes_encoded: 0,
            _bytes_written: 0,
        }
    }

    pub fn encode(&mut self, greeting_: &ZmqSocksGreeting) {
        let mut ptr = self.buf

            * ptr += 1 = 0x05;
        *ptr += 1 = (greeting_.num_methods);
        // for (uint8_t i = 0; i < greeting_.num_methods; i+= 1)
        for i in 0..greeting_.num_methods {
            *ptr += 1 = greeting_.methods[i];
        }

        _bytes_encoded = 2 + greeting_.num_methods;
        _bytes_written = 0;
    }

    pub fn output(&mut self, fd: ZmqFileDesc) -> i32 {
        let rc: i32 = tcp_write(fd, buf + _bytes_written, _bytes_encoded - _bytes_written);
        if (rc > 0) {
            _bytes_written += (rc);
        }
        return rc;
    }

    pub fn has_pending_data(&mut self) -> bool {
        return _bytes_written < _bytes_encoded;
    }

    pub fn reset(&mut self) {
        self._bytes_encoded = 0;
        self._bytes_written = 0;
    }
} // impl ZmqSocksGreetingEncoder

impl ZmqSocksChoice {
    // ZmqSocksChoice::ZmqSocksChoice (unsigned char method_) : method (method_)
    // {
    // }
    pub fn new(method_: u8) -> Self {
        Self {
            method: method_,
        }
    }
}

impl ZmqSocksChoiceDecoder {
    pub fn new() -> Self {
        Self {
            buf: [0; 2],
            _bytes_read: 0,
        }
    }

    pub fn input(&mut self, fd: ZmqFileDesc) -> i32 {
        // zmq_assert (_bytes_read < 2);
        let rc: i32 = tcp_read(fd, buf + _bytes_read, 2 - _bytes_read);
        if (rc > 0) {
            _bytes_read += (rc);
            if (buf[0] != 0x05) {
                return -1;
            }
        }
        return rc;
    }

    pub fn message_ready(&mut self) -> bool {
        return _bytes_read == 2;
    }

    pub fn decode(&mut self) -> ZmqSocksChoice {
        // zmq_assert (message_ready ());
        return ZmqSocksChoice(buf[1]);
    }

    pub fn reset(&mut self) {
        _bytes_read = 0;
    }
}

impl ZmqSocksBasicAuthRequest {
    pub fn new(username_: &str, password_: &str) -> Self {
        // zmq_assert (username_.size () <= UINT8_MAX);
        // zmq_assert (password_.size () <= UINT8_MAX);
        Self {
            username: username_.to_string(),
            password: password_.to_string(),
        }
    }
}

impl ZmqSocksBasicAuthRequestEncoder {
    pub fn new() -> Self {
        Self {
            buf: [0; u8::MAX],
            _bytes_encoded: 0,
            _bytes_written: 0,
        }
    }

    pub fn encode(&mut self, req_: &ZmqSocksBasicAuthRequest) {
        // unsigned char *ptr = buf;
        let mut ptr = self.buf;
        ptr[0] = 0x01;
        advance_ptr(&mut ptr, 1);
        // *ptr+= 1 = 0x01;
        ptr[0] = (req_.username.size());
        advance_ptr(&mut ptr, 1);
        copy_bytes(&mut ptr, 0, req_.username.as_bytes(), 0, req_.username.size());
        ptr += req_.username.size();
        *ptr += 1 = (req_.password.size());
        copy_bytes(&mut ptr, 0, req_.password.as_bytes(), 0, req_.password.size());
        ptr += req_.password.size();

        _bytes_encoded = ptr - buf;
        _bytes_written = 0;
    }

    pub fn output(&mut self, fd: ZmqFileDesc) -> i32 {
        let rc: i32 = tcp_write(fd, buf + _bytes_written, _bytes_encoded - _bytes_written);
        if (rc > 0) {
            _bytes_written += (rc);
        }
        return rc;
    }

    pub fn has_pending_data(&mut self) -> bool {
        return _bytes_written < _bytes_encoded;
    }

    pub fn reset(&mut self) {
        _bytes_encoded = _bytes_written = 0;
    }
}

impl ZmqSocksAuthResponse {
    pub fn new(response_code_: u8) -> Self {
        Self {
            response_code: response_code_,
        }
    }
}

impl ZmqSocksAuthResponseDecoder {
    pub fn new() -> Self {
        Self {
            buf: [0; 2],
            _bytes_read: 0,
        }
    }

    pub fn input(&mut self, fd: ZmqFileDesc) -> i32 {
        // zmq_assert (_bytes_read < 2);
        let rc: i32 = tcp_read(fd, buf + _bytes_read, 2 - _bytes_read);
        if (rc > 0) {
            _bytes_read += (rc);
            if (buf[0] != 0x01) {
                return -1;
            }
        }
        return rc;
    }

    pub fn message_ready() -> bool {
        return _bytes_read == 2;
    }

    pub fn decode(&mut self) -> ZmqSocksAuthResponse {
        // zmq_assert (message_ready ());
        return ZmqSocksAuthResponse(buf[1]);
    }

    pub fn reset(&mut self) {
        self._bytes_read = 0;
    }
}

impl ZmqSocksRequest {
    pub fn new(command_: u8, hostname_: &str, port_: u16) -> Self {
        // zmq_assert (hostname.size () <= UINT8_MAX);
        Self {
            command: command_,
            hostname: hostname_.to_string(),
            port: port_,
        }
    }
}

impl ZmqSocksRequestEncoder {
    pub fn new() -> Self {
        Self {
            buf: [0; u8::MAX],
            _bytes_encoded: 0,
            _bytes_written: 0,
        }
    }

    pub fn encode(&mut self, req_: &ZmqSocksRequest) {
        // zmq_assert (req_.hostname.size () <= UINT8_MAX);

        let mut ptr = buf;
        // *ptr+= 1 = 0x05;
        ptr[0] = 0x05;
        advance_ptr(ptr, 1);
        // *ptr+= 1 = req_.command;
        ptr[0] = req_.command;
        advance_ptr(ptr, 1);
        // *ptr+= 1 = 0x00;
        ptr[0] = 0x00;
        advance_ptr(ptr, 1);
// #if defined ZMQ_HAVE_OPENVMS && defined __ia64 && __INITIAL_POINTER_SIZE == 64
//     __addrinfo64 hints, *res = null_mut();
// #else
//     addrinfo hints, *res = null_mut();
        let mut hints: addrinfo = addrinfo::default();
        let mut res: addrinfo = addrinfo::default();
// #endif

        // memset (&hints, 0, sizeof hints);

        //  Suppress potential DNS lookups.
        hints.ai_flags = AI_NUMERICHOST as i32;

        // let rc: i32 = getaddrinfo (Some(&req_.hostname), None, Some(hints.clone()), &res);
        if rc == 0 && res.ai_family == AF_INET {
            let sockaddr_in = (res.ai_addr);
            // *ptr+= 1 = 0x01;
            ptr[0] = 0x01;
            advance_ptr(ptr, 1);
            copy_bytes(ptr, 0, &sockaddr_in.sin_addr, 0, 4);
            ptr += 4;
        } else if rc == 0 && res.ai_family == AF_INET6 {
            let sockaddr_in6 = (res.ai_addr);
            // *ptr+= 1 = 0x04;
            ptr[0] = 0x04;
            advance_ptr(ptr, 1);
            copy_bytes(ptr, 0, &sockaddr_in6.sin6_addr, 0, 16);
            // ptr += 16;
            advance_ptr(ptr, 16);
        } else {
            // *ptr+= 1 = 0x03;
            ptr[0] = 0x03;
            advance_ptr(ptr, 1);
            // *ptr+= 1 =  (req_.hostname.size ());
            ptr[0] = req_.hostname.size();
            advance_ptr(ptr, 1);
            copy_bytes(ptr, 0, req_.hostname.as_bytes(), 0, req_.hostname.size());
            ptr += req_.hostname.size();
        }

        if (rc == 0) {
            // freeaddrinfo(res);
        }

        // *ptr+= 1 = req_.port / 256;
        ptr[0] = req_.port / 256;
        advance_ptr(ptr, 1);
        // *ptr+= 1 = req_.port % 256;
        ptr[0] = req_.port % 256;
        advance_ptr(ptr, 1);

        _bytes_encoded = ptr - buf;
        _bytes_written = 0;
    }

    pub fn output(&mut self, fd: ZmqFileDesc) -> i32 {
        let rc: i32 = tcp_write(fd, buf + _bytes_written, _bytes_encoded - _bytes_written);
        if (rc > 0) {
            _bytes_written += (rc);
        }
        return rc;
    }

    pub fn has_pending_data(&mut self) -> bool {
        return _bytes_written < _bytes_encoded;
    }

    pub fn reset(&mut self) {
        self._bytes_encoded = 0;
        self._bytes_written = 0;
    }
}

impl ZmqSocksResponse {
    pub fn new(response_code_: u8, address_: &str, port_: u16) -> Self {
        // zmq_assert (address.size () <= UINT8_MAX);
        Self {
            response_code: response_code_,
            address: address_.to_string(),
            port: port_,
        }
    }
}

impl ZmqSocksResponseDecoder {
    pub fn new() -> Self {
        Self {
            buf: [0; u8::MAX],
            _bytes_read: 0,
        }
    }

    pub fn input(&mut self, fd: ZmqFileDesc) -> i32 {
        let mut n = 0;

        if (_bytes_read < 5) {
            n = 5 - _bytes_read;
        } else {
            let mut atyp = buf: [u8; 3];
            // zmq_assert (atyp == 0x01 || atyp == 0x03 || atyp == 0x04);
            if (atyp == 0x01) {
                n = 3 + 2;
            } else if (atyp == 0x03) {
                n = buf[4] + 2;
            } else if (atyp == 0x04) {
                n = 15 + 2;
            }
        }
        let rc: i32 = tcp_read(fd, buf + _bytes_read, n);
        if (rc > 0) {
            _bytes_read += (rc);
            if (buf[0] != 0x05) {
                return -1;
            }
            if (_bytes_read >= 2) {
                if (buf[1] > 0x08) {
                    return -1;
                }
            }
            if (_bytes_read >= 3) {
                if (buf[2] != 0x00) {
                    return -1;
                }
            }
            if (_bytes_read >= 4) {
                let atyp = buf[3];
                if (atyp != 0x01 && atyp != 0x03 && atyp != 0x04) {
                    return -1;
                }
            }
        }
        return rc;
    }

    pub fn message_ready(&mut self) -> bool {
        if (_bytes_read < 4) {
            return false;
        }

        let atyp = buf[3];
        // zmq_assert (atyp == 0x01 || atyp == 0x03 || atyp == 0x04);
        if (atyp == 0x01) {
            return _bytes_read == 10;
        }
        if (atyp == 0x03) {
            return _bytes_read > 4 && _bytes_read == 4 + 1 + buf[4] + 2;
        }

        return _bytes_read == 22;
    }

    pub fn decode(&mut self) -> ZmqSocksResponse {
        // zmq_assert (message_ready ());
        return ZmqSocksResponse(buf[1], "", 0);
    }

    pub fn reset(&mut self) {
        _bytes_read = 0;
    }
}










