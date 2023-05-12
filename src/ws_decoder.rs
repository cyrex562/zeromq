/*
    Copyright (c) 2007-2019 Contributors as noted in the AUTHORS file

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

// #include "ws_protocol.hpp"
// #include "ws_decoder.hpp"
// #include "likely.hpp"
// #include "wire.hpp"
// #include "err.hpp"

//  Decoder for Web socket framing protocol. Converts data stream into messages.
//  The class has to inherit from shared_message_memory_allocator because
//  the base class calls allocate in its constructor.
pub struct ws_decoder_t
    : public DecoderBase<ws_decoder_t, shared_message_memory_allocator>
{
//
    ws_decoder_t (bufsize_: usize,
                  maxmsgsize_: i64,
                  zero_copy_: bool,
                  must_mask_: bool);
    ~ws_decoder_t ();

    //  ZmqDecoderInterface interface.
    ZmqMessage *msg () { return &in_progress; }

  //
    int opcode_ready (unsigned char const *);
    int size_first_byte_ready (unsigned char const *);
    int short_size_ready (unsigned char const *);
    int long_size_ready (unsigned char const *);
    int mask_ready (unsigned char const *);
    int flags_ready (unsigned char const *);
    int message_ready (unsigned char const *);

    int size_ready (unsigned char const *);

    unsigned char _tmpbuf[8];
    unsigned char _msg_flags;
    ZmqMessage in_progress;

    const _zero_copy: bool
    const i64 _max_msg_size;
    const _must_mask: bool
    u64 _size;
    ws_protocol_t::opcode_t _opcode;
    unsigned char _mask[4];

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ws_decoder_t)
};

ws_decoder_t::ws_decoder_t (bufsize_: usize,
                                 maxmsgsize_: i64,
                                 zero_copy_: bool,
                                 must_mask_: bool) :
    DecoderBase<ws_decoder_t, shared_message_memory_allocator> (bufsize_),
    _msg_flags (0),
    _zero_copy (zero_copy_),
    _max_msg_size (maxmsgsize_),
    _must_mask (must_mask_),
    _size (0)
{
    memset (_tmpbuf, 0, mem::size_of::<_tmpbuf>());
    int rc = in_progress.init ();
    // errno_assert (rc == 0);

    //  At the beginning, read one byte and go to opcode_ready state.
    next_step (_tmpbuf, 1, &ws_decoder_t::opcode_ready);
}

ws_decoder_t::~ws_decoder_t ()
{
    let rc: i32 = in_progress.close ();
    // errno_assert (rc == 0);
}

int ws_decoder_t::opcode_ready (unsigned char const *)
{
    const bool final = (_tmpbuf[0] & 0x80) != 0; // final bit
    if (!final)
        return -1; // non final messages are not supported

    _opcode = static_cast<ws_protocol_t::opcode_t> (_tmpbuf[0] & 0xF);

    _msg_flags = 0;

    switch (_opcode) {
        case ws_protocol_t::opcode_binary:
            break;
        case ws_protocol_t::opcode_close:
            _msg_flags = ZMQ_MSG_COMMAND | ZMQ_MSG_CLOSE_CMD;
            break;
        case ws_protocol_t::opcode_ping:
            _msg_flags = ZMQ_MSG_PING | ZMQ_MSG_COMMAND;
            break;
        case ws_protocol_t::opcode_pong:
            _msg_flags = ZMQ_MSG_PONG | ZMQ_MSG_COMMAND;
            break;
        _ =>
            return -1;
    }

    next_step (_tmpbuf, 1, &ws_decoder_t::size_first_byte_ready);

    return 0;
}

int ws_decoder_t::size_first_byte_ready (unsigned char const *read_from_)
{
    const bool is_masked = (_tmpbuf[0] & 0x80) != 0;

    if (is_masked != _must_mask) // wrong mask value
        return -1;

    _size =  (_tmpbuf[0] & 0x7F);

    if (_size < 126) {
        if (_must_mask)
            next_step (_tmpbuf, 4, &ws_decoder_t::mask_ready);
        else if (_opcode == ws_protocol_t::opcode_binary) {
            if (_size == 0)
                return -1;
            next_step (_tmpbuf, 1, &ws_decoder_t::flags_ready);
        } else
            return size_ready (read_from_);
    } else if (_size == 126)
        next_step (_tmpbuf, 2, &ws_decoder_t::short_size_ready);
    else
        next_step (_tmpbuf, 8, &ws_decoder_t::long_size_ready);

    return 0;
}


int ws_decoder_t::short_size_ready (unsigned char const *read_from_)
{
    _size = (_tmpbuf[0] << 8) | _tmpbuf[1];

    if (_must_mask)
        next_step (_tmpbuf, 4, &ws_decoder_t::mask_ready);
    else if (_opcode == ws_protocol_t::opcode_binary) {
        if (_size == 0)
            return -1;
        next_step (_tmpbuf, 1, &ws_decoder_t::flags_ready);
    } else
        return size_ready (read_from_);

    return 0;
}

int ws_decoder_t::long_size_ready (unsigned char const *read_from_)
{
    //  The payload size is encoded as 64-bit unsigned integer.
    //  The most significant byte comes first.
    _size = get_uint64 (_tmpbuf);

    if (_must_mask)
        next_step (_tmpbuf, 4, &ws_decoder_t::mask_ready);
    else if (_opcode == ws_protocol_t::opcode_binary) {
        if (_size == 0)
            return -1;
        next_step (_tmpbuf, 1, &ws_decoder_t::flags_ready);
    } else
        return size_ready (read_from_);

    return 0;
}

int ws_decoder_t::mask_ready (unsigned char const *read_from_)
{
    memcpy (_mask, _tmpbuf, 4);

    if (_opcode == ws_protocol_t::opcode_binary) {
        if (_size == 0)
            return -1;

        next_step (_tmpbuf, 1, &ws_decoder_t::flags_ready);
    } else
        return size_ready (read_from_);

    return 0;
}

int ws_decoder_t::flags_ready (unsigned char const *read_from_)
{
    unsigned char flags;

    if (_must_mask)
        flags = _tmpbuf[0] ^ _mask[0];
    else
        flags = _tmpbuf[0];

    if (flags & ws_protocol_t::more_flag)
        _msg_flags |= ZMQ_MSG_MORE;
    if (flags & ws_protocol_t::command_flag)
        _msg_flags |= ZMQ_MSG_COMMAND;

    _size -= 1;

    return size_ready (read_from_);
}


int ws_decoder_t::size_ready (unsigned char const *read_pos_)
{
    //  Message size must not exceed the maximum allowed size.
    if (_max_msg_size >= 0)
        if ( (_size >  (_max_msg_size))) {
            errno = EMSGSIZE;
            return -1;
        }

    //  Message size must fit into size_t data type.
    if ( (_size !=  (_size))) {
        errno = EMSGSIZE;
        return -1;
    }

    int rc = in_progress.close ();
    assert (rc == 0);

    // the current message can exceed the current buffer. We have to copy the buffer
    // data into a new message and complete it in the next receive.

    shared_message_memory_allocator &allocator = get_allocator ();
    if ( (!_zero_copy || allocator.data () > read_pos_
                  ||  (read_pos_ - allocator.data ())
                       > allocator.size ()
                  || _size >  (
                       allocator.data () + allocator.size () - read_pos_))) {
        // a new message has started, but the size would exceed the pre-allocated arena
        // (or read_pos_ is in the initial handshake buffer)
        // this happens every time when a message does not fit completely into the buffer
        rc = in_progress.init_size ( (_size));
    } else {
        // construct message using n bytes from the buffer as storage
        // increase buffer ref count
        // if the message will be a large message, pass a valid refcnt memory location as well
        rc = in_progress.init (
           (read_pos_),  (_size),
          shared_message_memory_allocator::call_dec_ref, allocator.buffer (),
          allocator.provide_content ());

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
               &ws_decoder_t::message_ready);

    return 0;
}

int ws_decoder_t::message_ready (unsigned char const *)
{
    if (_must_mask) {
        int mask_index = _opcode == ws_protocol_t::opcode_binary ? 1 : 0;

        unsigned char *data =
           (in_progress.data ());
        for (size_t i = 0; i < _size; += 1i, mask_index+= 1)
            data[i] = data[i] ^ _mask[mask_index % 4];
    }

    //  Message is completely read. Signal this to the caller
    //  and prepare to decode next message.
    next_step (_tmpbuf, 1, &ws_decoder_t::opcode_ready);
    return 1;
}
