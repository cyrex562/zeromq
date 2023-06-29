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

// #ifndef __ZMQ_I_DECODER_HPP_INCLUDED__
// #define __ZMQ_I_DECODER_HPP_INCLUDED__

// #include "macros.hpp"
// #include "stdint.hpp"

use crate::message::ZmqMessage;

//  Interface to be implemented by message decoder.
pub trait ZmqDecoderInterface {
    //
    //     virtual ~ZmqDecoderInterface () ZMQ_DEFAULT;
    //     virtual void get_buffer (unsigned char **data, size: *mut usize) = 0;
    fn get_buffer(&mut self, data: &mut [u8], size: &mut usize);
    // virtual void resize_buffer (size_t) = 0;
    fn resize_buffer(&mut self, size: usize);
    //  Decodes data pointed to by data.
    //  When a message is decoded, 1 is returned.
    //  When the decoder needs more data, 0 is returned.
    //  On error, -1 is returned and errno is set accordingly.
    // virtual int decode (const data: &mut [u8], size: usize, size_t &processed_) = 0;
    fn decode(data: &[u8], size: usize, processed: &usize);
    // virtual ZmqMessage *msg () = 0;
    fn msg() -> ZmqMessage;
}

// #endif
