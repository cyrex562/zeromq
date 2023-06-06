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

use bincode::options;
use libc::{EINVAL, ENOTSUP};
use crate::context::ZmqContext;
use crate::decoder_allocators::data;
use crate::defines::{ZMQ_SUB, ZMQ_UNSUBSCRIBE};
use crate::message::{close_and_return, ZmqMessage};

use crate::xsub::XSub;

// #include "precompiled.hpp"
// #include "sub.hpp"
// #include "msg.hpp"
pub struct ZmqSub
{
// : public XSub
pub xsub: XSub,
//     ZmqSub (ZmqContext *parent_, tid: u32, sid_: i32);
//     ~ZmqSub ();
//     int xsetsockopt (option_: i32, const optval_: &mut [u8], optvallen_: usize);
//     int xsend (msg: &mut ZmqMessage);
//     bool xhas_out ();

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqSub)
}

impl ZmqSub {
    pub fn new(ctx: &mut ZmqContext, parent: &mut ZmqContext, tid: u32, sid_: i32) -> Self

    {
        // XSub (parent_, tid, sid_)
        ctx.type_ = ZMQ_SUB as i32;
        //  Switch filtering messages on (as opposed to XSUB which where the
        //  filtering is off).
        ctx.filter = true;
        Self {
            xsub: XSub::new(parent, tid, sid_),
        }
    }

    // ZmqSub::~ZmqSub ()
    // {
    // }

    pub fn xsetsockopt (&mut self,
                        option_: i32,
                        optval_: &mut [u8],
                        optvallen_: usize) -> i32
    {
        if option_ != ZMQ_SUBSCRIBE && option_ != ZMQ_UNSUBSCRIBE {
            errno = EINVAL;
            return -1;
        }

        //  Create the subscription message.
    let mut msg = ZmqMessage::default();
        rc: i32;
        let data = (optval_);
        if option_ == ZMQ_SUBSCRIBE {
            rc = msg.init_subscribe (optvallen_, data);
        } else {
            rc = msg.init_cancel (optvallen_, data);
        }
        // errno_assert (rc == 0);

        //  Pass it further on in the stack.
        rc = self.xsub.xsend (&mut msg);
        return close_and_return (&mut msg, rc);
    }

    pub fn xsend (&mut self, msg: &mut ZmqMessage) -> i32
    {
        //  Override the XSUB's send.
        errno = ENOTSUP;
        return -1;
    }

    pub fn xhas_out (&mut self) -> bool
    {
        //  Override the XSUB's send.
        return false;
    }

}
