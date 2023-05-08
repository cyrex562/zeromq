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
// #include <cmath>

// #include "v2_protocol.hpp"
// #include "v2_decoder.hpp"
// #include "likely.hpp"
// #include "wire.hpp"
// #include "err.hpp"

//  Decoder for ZMTP/2.x framing protocol. Converts data stream into messages.
//  The class has to inherit from shared_message_memory_allocator because
//  the base class calls allocate in its constructor.
pub struct v2_decoder_t
    : public DecoderBase<v2_decoder_t, shared_message_memory_allocator>
{
//
    v2_decoder_t (bufsize_: usize, i64 maxmsgsize_, zero_copy_: bool);
    ~v2_decoder_t ();

    //  ZmqDecoderInterface interface.
    ZmqMessage *msg () { return &in_progress; }

  //
    int flags_ready (unsigned char const *);
    int one_byte_size_ready (unsigned char const *);
    int eight_byte_size_ready (unsigned char const *);
    int message_ready (unsigned char const *);

    int size_ready (size: u64, unsigned char const *);

    unsigned char _tmpbuf[8];
    unsigned char _msg_flags;
    ZmqMessage in_progress;

    const _zero_copy: bool
    const i64 _max_msg_size;

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (v2_decoder_t)
};

v2_decoder_t::v2_decoder_t (bufsize_: usize,
                                 i64 maxmsgsize_,
                                 zero_copy_: bool) :
    DecoderBase<v2_decoder_t, shared_message_memory_allocator> (bufsize_),
    _msg_flags (0),
    _zero_copy (zero_copy_),
    _max_msg_size (maxmsgsize_)
{
    int rc = in_progress.init ();
    // errno_assert (rc == 0);

    //  At the beginning, read one byte and go to flags_ready state.
    next_step (_tmpbuf, 1, &v2_decoder_t::flags_ready);
}

v2_decoder_t::~v2_decoder_t ()
{
    let rc: i32 = in_progress.close ();
    // errno_assert (rc == 0);
}

int v2_decoder_t::flags_ready (unsigned char const *)
{
    _msg_flags = 0;
    if (_tmpbuf[0] & v2_protocol_t::more_flag)
        _msg_flags |= ZMQ_MSG_MORE;
    if (_tmpbuf[0] & v2_protocol_t::command_flag)
        _msg_flags |= ZMQ_MSG_COMMAND;

    //  The payload length is either one or eight bytes,
    //  depending on whether the 'large' bit is set.
    if (_tmpbuf[0] & v2_protocol_t::large_flag)
        next_step (_tmpbuf, 8, &v2_decoder_t::eight_byte_size_ready);
    else
        next_step (_tmpbuf, 1, &v2_decoder_t::one_byte_size_ready);

    return 0;
}

int v2_decoder_t::one_byte_size_ready (unsigned char const *read_from_)
{
    return size_ready (_tmpbuf[0], read_from_);
}

int v2_decoder_t::eight_byte_size_ready (unsigned char const *read_from_)
{
    //  The payload size is encoded as 64-bit unsigned integer.
    //  The most significant byte comes first.
    const u64 msg_size = get_uint64 (_tmpbuf);

    return size_ready (msg_size, read_from_);
}

int v2_decoder_t::size_ready (msg_size_: u64,
                                   unsigned char const *read_pos_)
{
    //  Message size must not exceed the maximum allowed size.
    if (_max_msg_size >= 0)
        if ( (msg_size_ > static_cast<u64> (_max_msg_size))) {
            errno = EMSGSIZE;
            return -1;
        }

    //  Message size must fit into size_t data type.
    if ( (msg_size_ !=  (msg_size_))) {
        errno = EMSGSIZE;
        return -1;
    }

    int rc = in_progress.close ();
    assert (rc == 0);

    // the current message can exceed the current buffer. We have to copy the buffer
    // data into a new message and complete it in the next receive.

    shared_message_memory_allocator &allocator = get_allocator ();
    if ( (!_zero_copy
                  || msg_size_ >  (
                       allocator.data () + allocator.size () - read_pos_))) {
        // a new message has started, but the size would exceed the pre-allocated arena
        // this happens every time when a message does not fit completely into the buffer
        rc = in_progress.init_size ( (msg_size_));
    } else {
        // construct message using n bytes from the buffer as storage
        // increase buffer ref count
        // if the message will be a large message, pass a valid refcnt memory location as well
        rc =
          in_progress.init (const_cast<unsigned char *> (read_pos_),
                              (msg_size_),
                             shared_message_memory_allocator::call_dec_ref,
                             allocator.buffer (), allocator.provide_content ());

        // For small messages, data has been copied and refcount does not have to be increased
        if (in_progress.is_zcmsg ()) {
            allocator.advance_content ();
            allocator.inc_ref ();
        }
    }

    if ( (rc)) {
        // errno_assert (errno == ENOMEM);
        rc = in_progress.init ();
        // errno_assert (rc == 0);
        errno = ENOMEM;
        return -1;
    }

    in_progress.set_flags (_msg_flags);
    // this sets read_pos to
    // the message data address if the data needs to be copied
    // for small message / messages exceeding the current buffer
    // or
    // to the current start address in the buffer because the message
    // was constructed to use n bytes from the address passed as argument
    next_step (in_progress.data (), in_progress.size (),
               &v2_decoder_t::message_ready);

    return 0;
}

int v2_decoder_t::message_ready (unsigned char const *)
{
    //  Message is completely read. Signal this to the caller
    //  and prepare to decode next message.
    next_step (_tmpbuf, 1, &v2_decoder_t::flags_ready);
    return 1;
}
