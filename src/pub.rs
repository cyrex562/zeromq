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

use crate::context::ZmqContext;
use crate::defines::ZMQ_PUB;
use crate::message::ZmqMessage;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::xpub::XPub;

// #include "precompiled.hpp"
// #include "pub.hpp"
// #include "pipe.hpp"
// #include "err.hpp"
// #include "msg.hpp"
pub struct ZmqPub {
    pub xpub: XPub,
    // ZmqPub (ZmqContext *parent_, tid: u32, sid_: i32);

    // ~ZmqPub ();

    //  Implementations of virtual functions from ZmqSocketBase.
    // void xattach_pipe (pipe: &mut ZmqPipe,
    //                    bool subscribe_to_all_ = false,
    //                    bool locally_initiated_ = false);

    // int xrecv (msg: &mut ZmqMessage);

    // bool xhas_in ();

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (pub_t)
}

impl ZmqPub {
    pub fn new(options: &mut ZmqOptions, parent: &mut ZmqContext, tid: u32, sid_: i32) -> Self {
        // : XPub (parent_, tid, sid_)
        let mut out = Self {
            xpub: XPub::new(parent, options, tid, sid),
        };
        out.xpub.options.type_ = ZMQ_PUB;
        out
    }

    // ZmqPub::~ZmqPub ()
    // {}

    pub fn xattach_pipe(&mut self, pipe: &mut ZmqPipe,
                        subscribe_to_all_: bool,
                        locally_initiated_: bool) {
        // zmq_assert (pipe);

        //  Don't delay pipe termination as there is no one
        //  to receive the delimiter.
        pipe.set_nodelay();

        self.xpub.xattach_pipe(self.xpub.options, pipe, subscribe_to_all_, locally_initiated_);
    }

    pub fn xrecv(&mut self, msg: &mut ZmqMessage) -> i32 {
        //  Messages cannot be received from PUB socket.
        // errno = ENOTSUP; return - 1;
        unimplemented!()
    }

    pub fn xhas_in() -> bool {
        return false;
    }
}