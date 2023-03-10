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

// #ifndef __ZMQ_CONFIG_HPP_INCLUDED__
// #define __ZMQ_CONFIG_HPP_INCLUDED__


//  Compile-time settings.


//  Number of new messages in message pipe needed to trigger new memory
//  allocation. Setting this parameter to 256 decreases the impact of
//  memory allocation by approximately 99.6%
pub const MESSAGE_PIPE_GRANULARITY: i32 = 256;

//  Commands in pipe per allocation event.
pub const COMMAND_PIPE_GRANULARITY: i32 = 16;

//  Determines how often does socket poll for new commands when it
//  still has unprocessed messages to handle. Thus, if it is set to 100,
//  socket will process 100 inbound messages before doing the poll.
//  If there are no unprocessed messages available, poll is done
//  immediately. Decreasing the value trades overall latency for more
//  real-time behaviour (less latency peaks).
pub const INBOUND_POLL_RATE: i32 = 100;

//  Maximal delta between high and low watermark.
pub const MAX_WM_DELTA: i32 = 1024;

//  Maximum number of events the I/O thread can process in one go.
pub const MAX_IO_EVENTS: i32 = 256;

//  Maximal batch size of packets forwarded by a ZMQ proxy.
//  Increasing this value improves throughput at the expense of
//  latency and fairness.
pub const PROXY_BURST_SIZE: i32 = 1000;

//  Maximal delay to process command in API thread (in CPU ticks).
//  3,000,000 ticks equals to 1 - 2 milliseconds on current CPUs.
//  Note that delay is only applied when there is continuous stream of
//  messages to process. If not so, commands are processed immediately.
pub const MAX_COMMAND_DELAY: i32 = 3000000;

//  Low-precision clock precision in CPU ticks. 1ms. Value of 1000000
//  should be OK for CPU frequencies above 1GHz. If should work
//  reasonably well for CPU frequencies above 500MHz. For lower CPU
//  frequencies you may consider lowering this value to get best
//  possible latencies.
pub const CLOCK_PRECISION: i32 = 1000000;

//  On some OSes the signaler has to be emulated using a TCP
//  connection. In such cases following port is used.
//  If 0, it lets the OS choose a free port without requiring use of a
//  global mutex. The original implementation of a Windows signaler
//  socket used port 5905 instead of letting the OS choose a free port.
//  https://github.com/zeromq/libzmq/issues/1542
pub const SIGNALER_PORT: i32 = 0;


// #endif
