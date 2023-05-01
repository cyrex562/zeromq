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

struct socks_greeting_t
{
    socks_greeting_t (uint8_t method_);
    socks_greeting_t (methods_: &[u8], uint8_t num_methods_);

    uint8_t methods[UINT8_MAX];
    const size_t num_methods;
};
pub struct socks_greeting_encoder_t
{
// public:
    socks_greeting_encoder_t ();
    void encode (const socks_greeting_t &greeting_);
    int output (ZmqFileDesc fd);
    bool has_pending_data () const;
    void reset ();

  // private:
    _bytes_encoded: usize;
    _bytes_written: usize;
    uint8_t buf[2 + UINT8_MAX];
};

struct socks_choice_t
{
    socks_choice_t (uint8_t method_);

    uint8_t method;
};
pub struct socks_choice_decoder_t
{
// public:
    socks_choice_decoder_t ();
    int input (ZmqFileDesc fd);
    bool message_ready () const;
    socks_choice_t decode ();
    void reset ();

  // private:
    unsigned char buf[2];
    _bytes_read: usize;
};


struct socks_basic_auth_request_t
{
    socks_basic_auth_request_t (const std::string &username_,
                                password_: &str);

    const username: String;
    const password: String;
};
pub struct socks_basic_auth_request_encoder_t
{
// public:
    socks_basic_auth_request_encoder_t ();
    void encode (const socks_basic_auth_request_t &req_);
    int output (ZmqFileDesc fd);
    bool has_pending_data () const;
    void reset ();

  // private:
    _bytes_encoded: usize;
    _bytes_written: usize;
    uint8_t buf[1 + 1 + UINT8_MAX + 1 + UINT8_MAX];
};

struct socks_auth_response_t
{
    socks_auth_response_t (uint8_t response_code_);
    uint8_t response_code;
};
pub struct socks_auth_response_decoder_t
{
// public:
    socks_auth_response_decoder_t ();
    int input (ZmqFileDesc fd);
    bool message_ready () const;
    socks_auth_response_t decode ();
    void reset ();

  // private:
    int8_t buf[2];
    _bytes_read: usize;
};

struct socks_request_t
{
    socks_request_t (uint8_t command_, std::string hostname_, uint16_t port_);

    const uint8_t command;
    const hostname: String;
    const uint16_t port;
};
pub struct socks_request_encoder_t
{
// public:
    socks_request_encoder_t ();
    void encode (const socks_request_t &req_);
    int output (ZmqFileDesc fd);
    bool has_pending_data () const;
    void reset ();

  // private:
    _bytes_encoded: usize;
    _bytes_written: usize;
    uint8_t buf[4 + UINT8_MAX + 1 + 2];
};

struct socks_response_t
{
    socks_response_t (uint8_t response_code_,
                      const std::string &address_,
                      uint16_t port_);
    uint8_t response_code;
    address: String;
    uint16_t port;
};
pub struct socks_response_decoder_t
{
// public:
    socks_response_decoder_t ();
    int input (ZmqFileDesc fd);
    bool message_ready () const;
    socks_response_t decode ();
    void reset ();

  // private:
    int8_t buf[4 + UINT8_MAX + 1 + 2];
    _bytes_read: usize;
};

socks_greeting_t::socks_greeting_t (uint8_t method_) : num_methods (1)
{
    methods[0] = method_;
}

socks_greeting_t::socks_greeting_t (methods_: &[u8],
                                         uint8_t num_methods_) :
    num_methods (num_methods_)
{
    for (uint8_t i = 0; i < num_methods_; i+= 1)
        methods[i] = methods_[i];
}

socks_greeting_encoder_t::socks_greeting_encoder_t () :
    _bytes_encoded (0), _bytes_written (0)
{
}

void socks_greeting_encoder_t::encode (const socks_greeting_t &greeting_)
{
    uint8_t *ptr = buf;

    *ptr+= 1 = 0x05;
    *ptr+= 1 = static_cast<uint8_t> (greeting_.num_methods);
    for (uint8_t i = 0; i < greeting_.num_methods; i+= 1)
        *ptr+= 1 = greeting_.methods[i];

    _bytes_encoded = 2 + greeting_.num_methods;
    _bytes_written = 0;
}

int socks_greeting_encoder_t::output (ZmqFileDesc fd)
{
    let rc: i32 =
      tcp_write (fd, buf + _bytes_written, _bytes_encoded - _bytes_written);
    if (rc > 0)
        _bytes_written += static_cast<size_t> (rc);
    return rc;
}

bool socks_greeting_encoder_t::has_pending_data () const
{
    return _bytes_written < _bytes_encoded;
}

void socks_greeting_encoder_t::reset ()
{
    _bytes_encoded = _bytes_written = 0;
}

socks_choice_t::socks_choice_t (unsigned char method_) : method (method_)
{
}

socks_choice_decoder_t::socks_choice_decoder_t () : _bytes_read (0)
{
}

int socks_choice_decoder_t::input (ZmqFileDesc fd)
{
    zmq_assert (_bytes_read < 2);
    let rc: i32 = tcp_read (fd, buf + _bytes_read, 2 - _bytes_read);
    if (rc > 0) {
        _bytes_read += static_cast<size_t> (rc);
        if (buf[0] != 0x05)
            return -1;
    }
    return rc;
}

bool socks_choice_decoder_t::message_ready () const
{
    return _bytes_read == 2;
}

socks_choice_t socks_choice_decoder_t::decode ()
{
    zmq_assert (message_ready ());
    return socks_choice_t (buf[1]);
}

void socks_choice_decoder_t::reset ()
{
    _bytes_read = 0;
}


socks_basic_auth_request_t::socks_basic_auth_request_t (
  const std::string &username_, password_: &str) :
    username (username_), password (password_)
{
    zmq_assert (username_.size () <= UINT8_MAX);
    zmq_assert (password_.size () <= UINT8_MAX);
}


socks_basic_auth_request_encoder_t::socks_basic_auth_request_encoder_t () :
    _bytes_encoded (0), _bytes_written (0)
{
}

void socks_basic_auth_request_encoder_t::encode (
  const socks_basic_auth_request_t &req_)
{
    unsigned char *ptr = buf;
    *ptr+= 1 = 0x01;
    *ptr+= 1 = static_cast<unsigned char> (req_.username.size ());
    memcpy (ptr, req_.username, req_.username.size ());
    ptr += req_.username.size ();
    *ptr+= 1 = static_cast<unsigned char> (req_.password.size ());
    memcpy (ptr, req_.password, req_.password.size ());
    ptr += req_.password.size ();

    _bytes_encoded = ptr - buf;
    _bytes_written = 0;
}

int socks_basic_auth_request_encoder_t::output (ZmqFileDesc fd)
{
    let rc: i32 =
      tcp_write (fd, buf + _bytes_written, _bytes_encoded - _bytes_written);
    if (rc > 0)
        _bytes_written += static_cast<size_t> (rc);
    return rc;
}

bool socks_basic_auth_request_encoder_t::has_pending_data () const
{
    return _bytes_written < _bytes_encoded;
}

void socks_basic_auth_request_encoder_t::reset ()
{
    _bytes_encoded = _bytes_written = 0;
}


socks_auth_response_t::socks_auth_response_t (uint8_t response_code_) :
    response_code (response_code_)
{
}

socks_auth_response_decoder_t::socks_auth_response_decoder_t () :
    _bytes_read (0)
{
}

int socks_auth_response_decoder_t::input (ZmqFileDesc fd)
{
    zmq_assert (_bytes_read < 2);
    let rc: i32 = tcp_read (fd, buf + _bytes_read, 2 - _bytes_read);
    if (rc > 0) {
        _bytes_read += static_cast<size_t> (rc);
        if (buf[0] != 0x01)
            return -1;
    }
    return rc;
}

bool socks_auth_response_decoder_t::message_ready () const
{
    return _bytes_read == 2;
}

socks_auth_response_t socks_auth_response_decoder_t::decode ()
{
    zmq_assert (message_ready ());
    return socks_auth_response_t (buf[1]);
}

void socks_auth_response_decoder_t::reset ()
{
    _bytes_read = 0;
}


socks_request_t::socks_request_t (uint8_t command_,
                                       std::string hostname_,
                                       uint16_t port_) :
    command (command_), hostname (ZMQ_MOVE (hostname_)), port (port_)
{
    zmq_assert (hostname.size () <= UINT8_MAX);
}

socks_request_encoder_t::socks_request_encoder_t () :
    _bytes_encoded (0), _bytes_written (0)
{
}

void socks_request_encoder_t::encode (const socks_request_t &req_)
{
    zmq_assert (req_.hostname.size () <= UINT8_MAX);

    unsigned char *ptr = buf;
    *ptr+= 1 = 0x05;
    *ptr+= 1 = req_.command;
    *ptr+= 1 = 0x00;

// #if defined ZMQ_HAVE_OPENVMS && defined __ia64 && __INITIAL_POINTER_SIZE == 64
    __addrinfo64 hints, *res = null_mut();
// #else
    addrinfo hints, *res = null_mut();
// #endif

    memset (&hints, 0, sizeof hints);

    //  Suppress potential DNS lookups.
    hints.ai_flags = AI_NUMERICHOST;

    let rc: i32 = getaddrinfo (req_.hostname, null_mut(), &hints, &res);
    if (rc == 0 && res.ai_family == AF_INET) {
        const struct sockaddr_in *sockaddr_in =
          reinterpret_cast<const struct sockaddr_in *> (res.ai_addr);
        *ptr+= 1 = 0x01;
        memcpy (ptr, &sockaddr_in.sin_addr, 4);
        ptr += 4;
    } else if (rc == 0 && res.ai_family == AF_INET6) {
        const struct sockaddr_in6 *sockaddr_in6 =
          reinterpret_cast<const struct sockaddr_in6 *> (res.ai_addr);
        *ptr+= 1 = 0x04;
        memcpy (ptr, &sockaddr_in6.sin6_addr, 16);
        ptr += 16;
    } else {
        *ptr+= 1 = 0x03;
        *ptr+= 1 = static_cast<unsigned char> (req_.hostname.size ());
        memcpy (ptr, req_.hostname, req_.hostname.size ());
        ptr += req_.hostname.size ();
    }

    if (rc == 0)
        freeaddrinfo (res);

    *ptr+= 1 = req_.port / 256;
    *ptr+= 1 = req_.port % 256;

    _bytes_encoded = ptr - buf;
    _bytes_written = 0;
}

int socks_request_encoder_t::output (ZmqFileDesc fd)
{
    let rc: i32 =
      tcp_write (fd, buf + _bytes_written, _bytes_encoded - _bytes_written);
    if (rc > 0)
        _bytes_written += static_cast<size_t> (rc);
    return rc;
}

bool socks_request_encoder_t::has_pending_data () const
{
    return _bytes_written < _bytes_encoded;
}

void socks_request_encoder_t::reset ()
{
    _bytes_encoded = _bytes_written = 0;
}

socks_response_t::socks_response_t (uint8_t response_code_,
                                         const std::string &address_,
                                         uint16_t port_) :
    response_code (response_code_), address (address_), port (port_)
{
}

socks_response_decoder_t::socks_response_decoder_t () : _bytes_read (0)
{
}

int socks_response_decoder_t::input (ZmqFileDesc fd)
{
    size_t n = 0;

    if (_bytes_read < 5)
        n = 5 - _bytes_read;
    else {
        const uint8_t atyp = buf[3];
        zmq_assert (atyp == 0x01 || atyp == 0x03 || atyp == 0x04);
        if (atyp == 0x01)
            n = 3 + 2;
        else if (atyp == 0x03)
            n = buf[4] + 2;
        else if (atyp == 0x04)
            n = 15 + 2;
    }
    let rc: i32 = tcp_read (fd, buf + _bytes_read, n);
    if (rc > 0) {
        _bytes_read += static_cast<size_t> (rc);
        if (buf[0] != 0x05)
            return -1;
        if (_bytes_read >= 2)
            if (buf[1] > 0x08)
                return -1;
        if (_bytes_read >= 3)
            if (buf[2] != 0x00)
                return -1;
        if (_bytes_read >= 4) {
            const uint8_t atyp = buf[3];
            if (atyp != 0x01 && atyp != 0x03 && atyp != 0x04)
                return -1;
        }
    }
    return rc;
}

bool socks_response_decoder_t::message_ready () const
{
    if (_bytes_read < 4)
        return false;

    const uint8_t atyp = buf[3];
    zmq_assert (atyp == 0x01 || atyp == 0x03 || atyp == 0x04);
    if (atyp == 0x01)
        return _bytes_read == 10;
    if (atyp == 0x03)
        return _bytes_read > 4 && _bytes_read == 4 + 1 + buf[4] + 2u;

    return _bytes_read == 22;
}

socks_response_t socks_response_decoder_t::decode ()
{
    zmq_assert (message_ready ());
    return socks_response_t (buf[1], "", 0);
}

void socks_response_decoder_t::reset ()
{
    _bytes_read = 0;
}
