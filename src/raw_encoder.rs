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
// #include "encoder.hpp"
// #include "raw_encoder.hpp"
// #include "msg.hpp"
pub struct raw_encoder_t ZMQ_FINAL : public encoder_base_t<raw_encoder_t>
{
// public:
    raw_encoder_t (bufsize_: usize);
    ~raw_encoder_t ();

  // private:
    void raw_message_ready ();

    ZMQ_NON_COPYABLE_NOR_MOVABLE (raw_encoder_t)
};

raw_encoder_t::raw_encoder_t (bufsize_: usize) :
    encoder_base_t<raw_encoder_t> (bufsize_)
{
    //  Write 0 bytes to the batch and go to message_ready state.
    next_step (null_mut(), 0, &raw_encoder_t::raw_message_ready, true);
}

raw_encoder_t::~raw_encoder_t ()
{
}

void raw_encoder_t::raw_message_ready ()
{
    next_step (in_progress ()->data (), in_progress ()->size (),
               &raw_encoder_t::raw_message_ready, true);
}
