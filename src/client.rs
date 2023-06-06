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

use std::ptr::null_mut;

use anyhow::anyhow;

use crate::content::ZmqContent;
use crate::context::ZmqContext;
use crate::defines::ZMQ_CLIENT;
use crate::fq::ZmqFq;
use crate::lb::LoadBalancer;
use crate::message::{ZmqMessage, ZMQ_MSG_MORE};
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::socket_base::ZmqSocketBase;
use crate::socket_base_ops::ZmqSocketBaseOps;

// #include "precompiled.hpp"
// #include "macros.hpp"
// #include "client.hpp"
// #include "err.hpp"
// #include "msg.hpp"
// pub struct client_t  : public ZmqSocketBase
#[derive(Default, Debug, Clone)]
pub struct ZmqClient {
    //
    //  Messages are fair-queued from inbound pipes. And load-balanced to
    //  the outbound pipes.
    // ZmqFq fair_queue;
    pub fq: ZmqFq,
    // LoadBalancer load_balance;
    pub lb: LoadBalancer,
    pub base: ZmqSocketBase, // // ZMQ_NON_COPYABLE_NOR_MOVABLE (client_t)
}

impl ZmqClient {
    //
    //     client_t (ZmqContext *parent_, tid: u32, sid_: i32);
    //     ~client_t ();
    //
    //   //
    //     client_t::client_t (parent: &mut ZmqContext, tid: u32, sid_: i32) :
    //     ZmqSocketBase (parent_, tid, sid_, true)
    //     {
    //     options.type = ZMQ_CLIENT;
    //     options.can_send_hello_msg = true;
    //     options.can_recv_hiccup_msg = true;
    //     }
    pub fn new(parent: &mut ZmqContext, tid: u32, sid: i32) -> Self {
        parent.type_ = ZMQ_CLIENT;
        parent.can_send_hello_msg = true;
        parent.can_recv_hiccup_msg = true;
        Self {
            fq: ZmqFq::Default(),
            lb: LoadBalancer::Default(),
            base: ZmqSocketBase::new(parent, tid, sid, true),
        }
    }

    // client_t::~client_t ()
    // {
    // }
}

impl ZmqSocketBaseOps for ZmqClient {
    //     //  Overrides of functions from ZmqSocketBase.
    //     void xattach_pipe (ZmqPipe *pipe_,
    //                        subscribe_to_all_: bool,
    //                        locally_initiated_: bool);
    fn xattach_pipe(
        &mut self,
        skt_base: &mut ZmqSocketBase,
        pipe: &mut ZmqPipe,
        subscribe_to_all: bool,
        locally_initiated: bool,
    ) {
        // LIBZMQ_UNUSED(subscribe_to_all_);
        // LIBZMQ_UNUSED(locally_initiated_);

        // zmq_assert(pipe_);

        self.fq.attach(pipe);
        self.lb.attach(pipe);
    }

    //     int xsend (msg: &mut ZmqMessage);
    fn xsend(&mut self, skt_base: &mut ZmqSocketBase, msg: &mut ZmqMessage) -> anyhow::Result<()> {
        //  CLIENT sockets do not allow multipart data (ZMQ_SNDMORE)
        if (msg.flags() & ZMQ_MSG_MORE) {
            // errno = EINVAL;
            // return -1;
            return Err(anyhow!(
                "EINVAL: client sockets do not allow multipart dart (ZMQ_SNDMORE)"
            ));
        }
        self.lb.sendpipe(msg, None)
    }

    //     int xrecv (msg: &mut ZmqMessage);
    fn xrecv(&mut self, skt_base: &mut ZmqSocketBase, msg: &mut ZmqMessage) -> anyhow::Result<()> {
        let mut rc = self.fq.recvpipe(msg, None);

        // Drop any messages with more flag
        while msg.flags() & ZMQ_MSG_MORE != 0 {
            // drop all frames of the current multi-frame message
            self.fq.recvpipe(msg, None)?;

            while msg.flags() & ZMQ_MSG_MORE != 0 {
                self.fq.recvpipe(msg, None)?;
            }

            // get the new message
            // if (rc == 0) {
            //     fair_queue.recvpipe(msg, null_mut());
            // }
            self.fq.recvpipe(msg, None)?;
        }

        Ok(())
    }

    //     bool xhas_in ();
    fn xhas_in(&mut self, skt_base: &mut ZmqSocketBase) -> bool {
        return self.fq.has_in();
    }

    //     bool xhas_out ();
    fn xhas_out(&mut self, skt_base: &mut ZmqSocketBase) -> bool {
        return self.lb.has_out();
    }

    //     void xread_activated (pipe_: &mut ZmqPipe);
    fn xread_activated(&mut self, skt_base: &mut ZmqSocketBase, pipe: &mut ZmqPipe) {
        self.fq.activated(pipe);
    }

    //     void xwrite_activated (pipe_: &mut ZmqPipe);
    fn xwrite_activated(&mut self, skt_base: &mut ZmqSocketBase, pipe: &mut ZmqPipe) {
        self.lb.activated(pipe);
    }

    //     void xpipe_terminated (pipe_: &mut ZmqPipe);
    fn xpipe_terminated(&mut self, skt_base: &mut ZmqSocketBase, pipe: &mut ZmqPipe) {
        self.fq.pipe_terminated(pipe);
        self.lb.pipe_terminated(pipe);
    }
}
