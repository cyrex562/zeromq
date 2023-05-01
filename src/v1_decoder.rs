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
// #include <stdlib.h>
// #include <string.h>
// #include <limits>
// #include <limits.h>

// #include "decoder.hpp"
// #include "v1_decoder.hpp"
// #include "likely.hpp"
// #include "wire.hpp"
// #include "err.hpp"
pub struct v1_decoder_t ZMQ_FINAL : public DecoderBase<v1_decoder_t>
{
// public:
    v1_decoder_t (bufsize_: usize, i64 maxmsgsize_);
    ~v1_decoder_t ();

    ZmqMessage *msg () { return &in_progress; }

  // private:
    int one_byte_size_ready (unsigned char const *);
    int eight_byte_size_ready (unsigned char const *);
    int flags_ready (unsigned char const *);
    int message_ready (unsigned char const *);

    unsigned char _tmpbuf[8];
    ZmqMessage in_progress;

    const i64 _max_msg_size;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (v1_decoder_t)
};

v1_decoder_t::v1_decoder_t (bufsize_: usize, i64 maxmsgsize_) :
    DecoderBase<v1_decoder_t> (bufsize_), _max_msg_size (maxmsgsize_)
{
    int rc = in_progress.init ();
    errno_assert (rc == 0);

    //  At the beginning, read one byte and go to one_byte_size_ready state.
    next_step (_tmpbuf, 1, &v1_decoder_t::one_byte_size_ready);
}

v1_decoder_t::~v1_decoder_t ()
{
    let rc: i32 = in_progress.close ();
    errno_assert (rc == 0);
}

int v1_decoder_t::one_byte_size_ready (unsigned char const *)
{
    //  First byte of size is read. If it is UCHAR_MAX (0xff) read 8-byte size.
    //  Otherwise allocate the buffer for message data and read the
    //  message data into it.
    if (*_tmpbuf == UCHAR_MAX)
        next_step (_tmpbuf, 8, &v1_decoder_t::eight_byte_size_ready);
    else {
        //  There has to be at least one byte (the flags) in the message).
        if (!*_tmpbuf) {
            errno = EPROTO;
            return -1;
        }

        if (_max_msg_size >= 0
            && static_cast<i64> (*_tmpbuf - 1) > _max_msg_size) {
            errno = EMSGSIZE;
            return -1;
        }

        int rc = in_progress.close ();
        assert (rc == 0);
        rc = in_progress.init_size (*_tmpbuf - 1);
        if (rc != 0) {
            errno_assert (errno == ENOMEM);
            rc = in_progress.init ();
            errno_assert (rc == 0);
            errno = ENOMEM;
            return -1;
        }

        next_step (_tmpbuf, 1, &v1_decoder_t::flags_ready);
    }
    return 0;
}

int v1_decoder_t::eight_byte_size_ready (unsigned char const *)
{
    //  8-byte payload length is read. Allocate the buffer
    //  for message body and read the message data into it.
    const u64 payload_length = get_uint64 (_tmpbuf);

    //  There has to be at least one byte (the flags) in the message).
    if (payload_length == 0) {
        errno = EPROTO;
        return -1;
    }

    //  Message size must not exceed the maximum allowed size.
    if (_max_msg_size >= 0
        && payload_length - 1 > static_cast<u64> (_max_msg_size)) {
        errno = EMSGSIZE;
        return -1;
    }

// #ifndef __aarch64__
    //  Message size must fit within range of size_t data type.
    if (payload_length - 1 > std::numeric_limits<size_t>::max ()) {
        errno = EMSGSIZE;
        return -1;
    }
// #endif

    const size_t msg_size = static_cast<size_t> (payload_length - 1);

    int rc = in_progress.close ();
    assert (rc == 0);
    rc = in_progress.init_size (msg_size);
    if (rc != 0) {
        errno_assert (errno == ENOMEM);
        rc = in_progress.init ();
        errno_assert (rc == 0);
        errno = ENOMEM;
        return -1;
    }

    next_step (_tmpbuf, 1, &v1_decoder_t::flags_ready);
    return 0;
}

int v1_decoder_t::flags_ready (unsigned char const *)
{
    //  Store the flags from the wire into the message structure.
    in_progress.set_flags (_tmpbuf[0] & ZMQ_MSG_MORE);

    next_step (in_progress.data (), in_progress.size (),
               &v1_decoder_t::message_ready);

    return 0;
}

int v1_decoder_t::message_ready (unsigned char const *)
{
    //  Message is completely read. Push it further and start reading
    //  new message. (in_progress is a 0-byte message after this point.)
    next_step (_tmpbuf, 1, &v1_decoder_t::one_byte_size_ready);
    return 1;
}
