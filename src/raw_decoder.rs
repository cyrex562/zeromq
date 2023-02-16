/*
    Copyright (c) 2007-2016 Contributors as noted in the AUTHORS file

    This file is part of libzmq, the ZeroMQ core engine in C++.

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

// #include "raw_decoder.hpp"
// #include "err.hpp"

zmq::raw_decoder_t::raw_decoder_t (bufsize_: usize) : _allocator (bufsize_, 1)
{
    let rc: i32 = _in_progress.init ();
    errno_assert (rc == 0);
}

zmq::raw_decoder_t::~raw_decoder_t ()
{
    let rc: i32 = _in_progress.close ();
    errno_assert (rc == 0);
}

void zmq::raw_decoder_t::get_buffer (unsigned char **data, size: *mut usize)
{
    *data = _allocator.allocate ();
    *size = _allocator.size ();
}

int zmq::raw_decoder_t::decode (const uint8_t *data,
                                size: usize,
                                size_t &bytes_used_)
{
    let rc: i32 =
      _in_progress.init (const_cast<unsigned char *> (data), size,
                         shared_message_memory_allocator::call_dec_ref,
                         _allocator.buffer (), _allocator.provide_content ());

    // if the buffer serves as memory for a zero-copy message, release it
    // and allocate a new buffer in get_buffer for the next decode
    if (_in_progress.is_zcmsg ()) {
        _allocator.advance_content ();
        _allocator.release ();
    }

    errno_assert (rc != -1);
    bytes_used_ = size;
    return 1;
}
pub struct raw_decoder_t ZMQ_FINAL : public i_decoder
{
// public:
    raw_decoder_t (bufsize_: usize);
    ~raw_decoder_t ();

    //  i_decoder interface.

    void get_buffer (unsigned char **data, size: *mut usize);

    int decode (const unsigned char *data, size: usize, size_t &bytes_used_);

    ZmqMessage *msg () { return &_in_progress; }

    void resize_buffer (size_t) {}

  // private:
    ZmqMessage _in_progress;

    shared_message_memory_allocator _allocator;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (raw_decoder_t)
};
