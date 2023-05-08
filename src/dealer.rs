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
// #include "macros.hpp"
// #include "dealer.hpp"
// #include "err.hpp"
// #include "msg.hpp"
// pub struct ZmqDealer : public ZmqSocketBase

use std::collections::VecDeque;
use std::mem;
use std::ptr::null_mut;

use anyhow::bail;

use crate::context::ZmqContext;
use crate::defines::ZMQ_DEALER;
use crate::lb::lb_t;
use crate::message::ZmqMessage;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::socket_base::ZmqSocketBase;
use crate::utils::copy_bytes;

#[derive(Default, Debug, Clone)]
pub struct ZmqDealer {
    pub socket_base: ZmqSocketBase,

    //
    //  Messages are fair-queued from inbound pipes. And load-balanced to
    //  the outbound pipes.
    // ZmqFq fair_queue;
    pub fair_queue: VecDeque<ZmqMessage>,

    // lb_t load_balance;
    pub load_balance: lb_t,

    // if true, send an empty message to every connected router peer
    pub probe_router: bool, // // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqDealer)
}

impl ZmqDealer {
    //
    // ZmqDealer (ZmqContext *parent_, tid: u32, sid_: i32);
    // ZmqSocketBase (parent_, tid, sid_), probe_router (false)
    pub fn new(
        options: &mut ZmqOptions,
        parent: &mut ZmqContext,
        tid: u32,
        sid_: i32,
    ) -> ZmqDealer {
        let mut out = Self::default();
        let mut base = ZmqSocketBase::new2(options, parent, tid, sid_);

        options.type_ = ZMQ_DEALER as i32;
        options.can_send_hello_msg = true;
        options.can_recv_hiccup_msg = true;
        out
    }

    // ~ZmqDealer () ;

    //
    //  Overrides of functions from ZmqSocketBase.
    // void xattach_pipe (pipe: &mut ZmqPipe,
    //    subscribe_to_all_: bool,
    //    locally_initiated_: bool) ;

    pub fn xattach_pipe(
        &mut self,
        pipe: &mut ZmqPipe,
        subscribe_to_all_: bool,
        locally_initiated_: bool,
    ) -> anyhow::Result<()> {
        // LIBZMQ_UNUSED (subscribe_to_all_);
        // LIBZMQ_UNUSED (locally_initiated_);

        // zmq_assert (pipe);

        if (self, probe_router) {
            let mut probe_msg: ZmqMessage = ZmqMessage::default();
            probe_msg.init2()?;
            // errno_assert (rc == 0);

            rc = pipe.write(&mut probe_msg);
            // zmq_assert (rc) is not applicable here, since it is not a bug.
            LIBZMQ_UNUSED(rc);

            pipe.flush();

            rc = probe_msg.close();
            // errno_assert(rc == 0);
        }

        self.fair_queue.attach(pipe);
        self.load_balance.attach(pipe);
        Ok(())
    }

    // int xsetsockopt (option_: i32,

    //                  const optval_: &mut [u8],
    //                  optvallen_: usize) ;

    pub fn xsetsockopt(option_: i32, optval_: &mut [u8], optvallen_: usize) -> anyhow::Result<()> {
        let is_int = (optvallen_ == mem::size_of::<int>());
        let mut value = 0;
        if (is_int) {
            let mut val_bytes: [u8; 4] = [0; 4];
            // memcpy (&value, optval_, mem::size_of::<int>());
            copy_bytes(&mut val_bytes, 0, optval_, 0, 4);
            value = i32::from_le_bytes(val_bytes);
        }

        match (option_) {
            ZMQ_PROBE_ROUTER => {
                if (is_int && value >= 0) {
                    probe_router = (value != 0);
                    return Ok(());
                }
            }
            // break;
            _ => {} // break;
        }

        // errno = EINVAL;
        // return -1;
        bail!("EINVAL")
    }

    // int xsend (msg: &mut ZmqMessage) ;
    pub fn xsend(&mut self, msg: &mut ZmqMessage) -> i32 {
        return self.sendpipe(msg, null_mut());
    }

    // int xrecv (msg: &mut ZmqMessage) ;
    pub fn xrecv(&mut self, msg: &mut ZmqMessage) -> i32 {
        return self.recvpipe(msg, null_mut());
    }

    // bool xhas_in () ;
    pub fn xhas_in(&mut self) -> bool {
        return self.fair_queue.has_in();
    }

    // bool xhas_out () ;
    pub fn xhas_out(&mut self) -> bool {
        return sellf.load_balance.has_out();
    }

    // void xread_activated (pipe: &mut ZmqPipe) ;
    pub fn xread_activated(&mut self, pipe: &mut ZmqPipe) {
        self.fair_queue.activated(pipe);
    }

    // void xwrite_activated (pipe: &mut ZmqPipe) ;
    pub fn xwrite_activated(&mut self, pipe: &mut ZmqPipe) {
        self.load_balance.activated(pipe);
    }

    // void xpipe_terminated (pipe: &mut ZmqPipe) ;
    pub fn xpipe_terminated(&mut self, pipe: &mut ZmqPipe) {
        self.fair_queue.pipe_terminated(pipe);
        self.load_balance.pipe_terminated(pipe);
    }

    //  Send and recv - knowing which pipe was used.

    // int sendpipe (msg: &mut ZmqMessage ZmqPipe **pipe);
    pub fn sendpipe(&mut self, msg: &mut ZmqMessage, pipe: *mut *mut ZmqPipe) -> i32 {
        return self.load_balance.sendpipe(msg, pipe);
    }

    pub fn recvpipe(&mut self, msg: &mut ZmqMessage, pipe: *mut *mut ZmqPipe) -> i32 {
        return self.fair_queue.recvpipe(msg, pipe);
    }

    // int recvpipe (msg: &mut ZmqMessage ZmqPipe **pipe);
}
