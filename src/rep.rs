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
// #include "rep.hpp"
// #include "err.hpp"
// #include "msg.hpp"

use crate::context::ZmqContext;
use crate::defines::ZMQ_REP;
use crate::message::{ZMQ_MSG_MORE, ZmqMessage};
use crate::router::router_t;

#[derive(Default, Debug, Clone)]
pub struct ZmqRep {
    // : public router_t
    pub router: router_t,
    //
    // ZmqRep (ZmqContext *parent_, tid: u32, sid_: i32);

    // ~ZmqRep ();

    //  Overrides of functions from ZmqSocketBase.
    // int xsend (msg: &mut ZmqMessage);

    // int xrecv (msg: &mut ZmqMessage);

    // bool xhas_in ();

    // bool xhas_out ();

    //  If true, we are in process of sending the reply. If false we are
    //  in process of receiving a request.
    pub _sending_reply: bool,

    //  If true, we are starting to receive a request. The beginning
    //  of the request is the backtrace stack.
    pub _request_begins: bool,

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (rep_t)
}

impl ZmqRep {
    pub fn new(parent: &mut ZmqContext, tid: u32, sid: i32) -> Self {
        let mut out = Self {
            router: router_t::new(parent, tid, sid),
            _sending_reply: false,
            _request_begins: true,
        };
        out.options.type_ = ZMQ_REP;
        out
    }

    pub fn xsend(&mut self, msg: &mut ZmqMessage) -> i32 {
//  If we are in the middle of receiving a request, we cannot send reply.
        if (!_sending_reply) {
            errno = EFSM;
            return -1;
        }

        let more = (msg.flags() & ZMQ_MSG_MORE) != 0;

//  Push message to the reply pipe.
        let rc: i32 = router_t::xsend(msg);
        if (rc != 0) {
            return rc;
        }

//  If the reply is complete flip the FSM back to request receiving state.
        if (!more) {
            _sending_reply = false;
        }

        return 0;
    }

    pub fn xrecv(&mut self, msg: &mut ZmqMessage) -> i32 {
//  If we are in middle of sending a reply, we cannot receive next request.
        if (_sending_reply) {
            errno = EFSM;
            return -1;
        }

//  First thing to do when receiving a request is to copy all the labels
//  to the reply pipe.
        if (_request_begins) {
            loop {
                let rc = self.router.xrecv(msg);
                if (rc != 0) {
                    return rc;
                }

                if ((msg.flags() & ZMQ_MSG_MORE)) {
                    //  Empty message part delimits the traceback stack. const bool
                    bottom = (msg.size() == 0);

//  Push it to the reply pipe.
                    rc = router_t::xsend(msg);
// errno_assert (rc == 0);

                    if (bottom) {
                        break;
                    }
                } else {
//  If the traceback stack is malformed, discard anything
//  already sent to pipe (we're at end of invalid message).
                    rc = router_t::rollback();
// errno_assert (rc == 0);
                }
            }
            _request_begins = false;
        }

//  Get next message part to return to the user.
        let rc: i32 = router_t::xrecv(msg);
        if (rc != 0) {
            return rc;
        }

//  If whole request is read, flip the FSM to reply-sending state.
        if (!(msg.flags() & ZMQ_MSG_MORE)) {
            _sending_reply = true;
            _request_begins = true;
        }

        return 0;
    }

    pub fn xhas_in(&mut self) -> bool {
        if (_sending_reply) {
            return false;
        }

        return self.router.xhas_in();
    }

    pub fn xhas_out(&mut self) -> bool {
        if (!_sending_reply) {
            return false;
        }

        return self.router.xhas_out();
    }
}


