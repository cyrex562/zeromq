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
use crate::fair_queue::ZmqFq;
use crate::lb::LoadBalancer;
use crate::message::{ZmqMessage, ZMQ_MSG_MORE};
use crate::pipe::ZmqPipe;
use crate::socket::ZmqSocket;
use crate::socket_base_ops::ZmqSocketBaseOps;

// #include "precompiled.hpp"
// #include "macros.hpp"
// #include "client.hpp"
// #include "err.hpp"
// #include "msg.hpp"
// pub struct client_t  : public ZmqSocketBase
#[derive(Default, Debug, Clone)]
pub struct ZmqClient<'a> {
    //
    //  Messages are fair-queued from inbound pipes. And load-balanced to
    //  the outbound pipes.
    // ZmqFq fair_queue;
    pub fq: ZmqFq,
    // LoadBalancer load_balance;
    pub lb: LoadBalancer,
    pub base: ZmqSocket<'a>, // // ZMQ_NON_COPYABLE_NOR_MOVABLE (client_t)
}

// fn client_xattach_pipe(
//
//     sock: &mut ZmqSocket,
//     pipe: &mut ZmqPipe,
//     subscribe_to_all: bool,
//     locally_initiated: bool,
// ) {
//     // LIBZMQ_UNUSED(subscribe_to_all_);
//     // LIBZMQ_UNUSED(locally_initiated_);
//
//     // zmq_assert(pipe_);
//
//     self.fq.attach(pipe);
//     self.lb.attach(pipe);
// }

//     int xsend (msg: &mut ZmqMessage);
pub fn client_xsend(sock: &mut ZmqSocket, msg: &mut ZmqMessage) -> anyhow::Result<()> {
    //  CLIENT sockets do not allow multipart data (ZMQ_SNDMORE)
    if (msg.flags() & ZMQ_MSG_MORE) {
        // errno = EINVAL;
        // return -1;
        return Err(anyhow!(
            "EINVAL: client sockets do not allow multipart dart (ZMQ_SNDMORE)"
        ));
    }
    sock.lb.sendpipe(msg, None)
}

//     int xrecv (msg: &mut ZmqMessage);
pub fn client_xrecv(sock: &mut ZmqSocket, msg: &mut ZmqMessage) -> anyhow::Result<()> {
    let mut rc = sock.fq.recvpipe(msg, None);

    // Drop any messages with more flag
    while msg.flags() & ZMQ_MSG_MORE != 0 {
        // drop all frames of the current multi-frame message
        sock.fq.recvpipe(msg, None)?;

        while msg.flags() & ZMQ_MSG_MORE != 0 {
            sock.fq.recvpipe(msg, None)?;
        }

        // get the new message
        // if (rc == 0) {
        //     fair_queue.recvpipe(msg, null_mut());
        // }
        sock.fq.recvpipe(msg, None)?;
    }

    Ok(())
}

//     bool xhas_in ();
pub fn client_xhas_in(sock: &mut ZmqSocket) -> bool {
    return self.fq.has_in();
}

//     bool xhas_out ();
pub fn client_xhas_out(sock: &mut ZmqSocket) -> bool {
    return self.lb.has_out();
}

//     void xread_activated (pipe_: &mut ZmqPipe);
pub fn client_xread_activated(sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    sock.fq.activated(pipe);
}

//     void xwrite_activated (pipe_: &mut ZmqPipe);
pub fn client_xwrite_activated(sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    sock.lb.activated(pipe);
}

//     void xpipe_terminated (pipe_: &mut ZmqPipe);
pub fn client_xpipe_terminated(sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    sock.fq.pipe_terminated(pipe);
    sock.lb.pipe_terminated(pipe);
}
