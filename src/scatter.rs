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

use libc::EINVAL;
use crate::context::ZmqContext;
use crate::defines::ZMQ_SCATTER;
use crate::lb::LoadBalancer;
use crate::message::{ZMQ_MSG_MORE, ZmqMessage};
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::socket_base::ZmqSocketBase;

// #include "precompiled.hpp"
// #include "macros.hpp"
// #include "scatter.hpp"
// #include "pipe.hpp"
// #include "err.hpp"
// #include "msg.hpp"
#[derive(Default, Debug, Clone)]
pub struct ZmqScatter {
    // : public ZmqSocketBase
    pub socket_base: ZmqSocketBase,
    //     ZmqScatter (ZmqContext *parent_, tid: u32, sid_: i32);
//     ~ZmqScatter ();
    //  Overrides of functions from ZmqSocketBase.
    // void xattach_pipe (pipe: &mut ZmqPipe,
    //                    subscribe_to_all_: bool,
    //                    locally_initiated_: bool);
    // int xsend (msg: &mut ZmqMessage);
    // bool xhas_out ();
    // void xwrite_activated (pipe: &mut ZmqPipe);
    // void xpipe_terminated (pipe: &mut ZmqPipe);
    //  Load balancer managing the outbound pipes.
    pub load_balance: LoadBalancer,
    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqScatter)
}

impl ZmqScatter {
    pub fn new(options: &mut ZmqOptions, parent: &mut ZmqContext, tid: u32, sid_: i32) -> Self

    {
        // ZmqSocketBase (parent_, tid, sid_, true)
        options.type_ = ZMQ_SCATTER as i32;
        Self {
            socket_base: ZmqSocketBase::new(parent, options, tid, sid, false),
            load_balance: LoadBalancer::new(),
        }
    }


    pub fn xattach_pipe(&mut self, pipe: &mut ZmqPipe,
                        subscribe_to_all_: bool,
                        locally_initiated_: bool) {
        // LIBZMQ_UNUSED (subscribe_to_all_);
        // LIBZMQ_UNUSED (locally_initiated_);

        //  Don't delay pipe termination as there is no one
        //  to receive the delimiter.
        pipe.set_nodelay();

        // zmq_assert (pipe);
        load_balance.attach(pipe);
    }

    pub fn xwrite_activated(&mut self, pipe: &mut ZmqPipe) {
        load_balance.activated(pipe);
    }

    pub fn xpipe_terminated(pipe: &mut ZmqPipe) {
        load_balance.pipe_terminated(pipe);
    }

    pub fn xsend(&mut self, msg: &mut ZmqMessage) -> i32 {
        //  SCATTER sockets do not allow multipart data (ZMQ_SNDMORE)
        if (msg.flags() & ZMQ_MSG_MORE) {
            errno = EINVAL;
            return -1;
        }

        return load_balance.send(msg);
    }

    pub fn xhas_out(&mut self) -> bool {
        return load_balance.has_out();
    }
}









