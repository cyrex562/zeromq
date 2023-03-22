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

use std::ptr::null_mut;
use anyhow::anyhow;
use crate::context::ZmqContext;
use crate::fq::fq_t;
use crate::lb::lb_t;
use crate::message::{ZMQ_MSG_MORE, ZmqMessage};
use crate::options::ZmqOptions;
use crate::pipe::pipe_t;
use crate::socket_base::ZmqSocketBase;
use crate::socket_base_ops::ZmqSocketBaseOps;
use crate::zmq_content::ZmqContent;
use crate::zmq_hdr::ZMQ_CLIENT;

// #include "precompiled.hpp"
// #include "macros.hpp"
// #include "client.hpp"
// #include "err.hpp"
// #include "msg.hpp"
// pub struct client_t ZMQ_FINAL : public ZmqSocketBase
#[derive(Default,Debug,Clone)]
pub struct ZmqClient
{


  // private:
    //  Messages are fair-queued from inbound pipes. And load-balanced to
    //  the outbound pipes.
    // fq_t _fq;
  pub fq: fq_t,
  // lb_t _lb;
    pub lb: lb_t,
    pub base: ZmqSocketBase
    // ZMQ_NON_COPYABLE_NOR_MOVABLE (client_t)
}

impl ZmqClient {
    // public:
//     client_t (ZmqContext *parent_, tid: u32, sid_: i32);
//     ~client_t ();
//
//   // protected:
//     client_t::client_t (class ZmqContext *parent_, tid: u32, sid_: i32) :
//     ZmqSocketBase (parent_, tid, sid_, true)
//     {
//     options.type = ZMQ_CLIENT;
//     options.can_send_hello_msg = true;
//     options.can_recv_hiccup_msg = true;
//     }
    pub fn new(parent: &mut ZmqContext, options: &mut ZmqOptions, tid: u32, sid: i32) -> Self {
        options.type_ = ZMQ_CLIENT;
        options.can_send_hello_msg = true;
        options.can_recv_hiccup_msg = true;
        Self {
            fq: fq_t::Default(),
            lb: lb_t::Default(),
            base: ZmqSocketBase::new(parent, options, tid, sid, true)
        }
    }

    // client_t::~client_t ()
    // {
    // }
}

impl ZmqSocketBaseOps for ZmqClient {
    //     //  Overrides of functions from ZmqSocketBase.
    //     void xattach_pipe (pipe_t *pipe_,
    //                        subscribe_to_all_: bool,
    //                        locally_initiated_: bool);
    fn xattach_pipe(&mut self, skt_base: &mut ZmqSocketBase, pipe: &mut pipe_t, subscribe_to_all: bool, locally_initiated: bool)
    {
        // LIBZMQ_UNUSED(subscribe_to_all_);
        // LIBZMQ_UNUSED(locally_initiated_);

        // zmq_assert(pipe_);

        self.fq.attach(pipe);
        self.lb.attach(pipe);
    }

    //     int xsend (msg: &mut ZmqMessage);
    fn xsend(&mut self, skt_base: &mut ZmqSocketBase, msg: &mut ZmqMessage) -> anyhow::Result<()>
    {
        //  CLIENT sockets do not allow multipart data (ZMQ_SNDMORE)
        if (msg.flags() & ZMQ_MSG_MORE) {
            // errno = EINVAL;
            // return -1;
            return Err(anyhow!("EINVAL: client sockets do not allow multipart dart (ZMQ_SNDMORE)"));
        }
        return self.lb.sendpipe(msg, null_mut());
    }

    //     int xrecv (msg: &mut ZmqMessage);
    fn xrecv(&mut self, skt_base: &mut ZmqSocketBase, msg: &mut ZmqMessage) -> anyhow::Result<()>
    {
        let mut rc = self.fq.recvpipe(msg, null_mut());

        // Drop any messages with more flag
        while (rc == 0 && (msg.flags() & ZMQ_MSG_MORE) != 0) {
            // drop all frames of the current multi-frame message
            rc = self.fq.recvpipe(msg, null_mut());

            while (rc == 0 && (msg.flags() & ZMQ_MSG_MORE) != 0) {
                rc = self.fq.recvpipe(msg, null_mut());
            }

            // get the new message
            if (rc == 0) {
                rc = _fq.recvpipe(msg, null_mut());
            }
        }

        return rc;
    }

    //     bool xhas_in ();
    fn xhas_in(&mut self, skt_base: &mut ZmqSocketBase) -> bool
    {
        return self.fq.has_in ();
    }

    //     bool xhas_out ();
    fn xhas_out(&mut self, skt_base: &mut ZmqSocketBase) -> bool
    {
        return self.lb.has_out ();
    }

    //     void xread_activated (pipe_: &mut pipe_t);
    fn xread_activated(&mut self, skt_base: &mut ZmqSocketBase, pipe: &mut pipe_t)
    {
        self.fq.activated (pipe);
    }

    //     void xwrite_activated (pipe_: &mut pipe_t);
    fn xwrite_activated(&mut self, skt_base: &mut ZmqSocketBase, pipe: &mut pipe_t)
    {
        self.lb.activated (pipe);
    }

    //     void xpipe_terminated (pipe_: &mut pipe_t);
    fn xpipe_terminated(&mut self, skt_base: &mut ZmqSocketBase, pipe: &mut pipe_t)
    {
    self.fq.pipe_terminated (pipe);
    self.lb.pipe_terminated (pipe);
    }

















}




