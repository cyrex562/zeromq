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

// #ifndef __ZMQ_PLAIN_COMMON_HPP_INCLUDED__
// #define __ZMQ_PLAIN_COMMON_HPP_INCLUDED__


pub const hello_prefix: &[u8;6]= b"\x05HELLO";
// const size_t hello_prefix_len = mem::size_of::<hello_prefix>() - 1;
pub const hello_prefix_len: usize = 5;

pub const welcome_prefix: &[u8;8] = b"\x07WELCOME";
// const size_t welcome_prefix_len = mem::size_of::<welcome_prefix>() - 1;
pub const welcome_prefix_len: usize = 7;

pub const initiate_prefix: &[u8;9] = b"\x08INITIATE";
// const size_t initiate_prefix_len = mem::size_of::<initiate_prefix>() - 1;
pub const initiate_prefix_len: usize = 8;

pub const ready_prefix: &[u8;6] = b"\x05READY";
// const size_t ready_prefix_len = mem::size_of::<ready_prefix>() - 1;
pub const ready_prefix_len: usize = 5;

pub const error_prefix: &[u8;6] = b"\x05ERROR";
// const size_t error_prefix_len = mem::size_of::<error_prefix>() - 1;
pub const error_prefix_len: usize = 5;

pub const brief_len_size: usize = 1;


// #endif
