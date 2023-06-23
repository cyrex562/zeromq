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
// #include "server.hpp"
// #include "pipe.hpp"
// #include "wire.hpp"
// #include "random.hpp"
// #include "likely.hpp"
// #include "err.hpp"


use std::collections::HashMap;
use std::ptr::null_mut;
use libc::{EAGAIN, EHOSTUNREACH, EINVAL, pipe};
use crate::context::ZmqContext;
use crate::defines::ZMQ_SERVER;
use crate::fair_queue::ZmqFq;
use crate::message::{ZMQ_MSG_MORE, ZmqMessage};
use crate::outpipe::ZmqOutpipe;

use crate::pipe::ZmqPipe;
use crate::socket::ZmqSocket;

//  TODO: This class uses O(n) scheduling. Rewrite it to use O(1) algorithm.
// #[derive(Default,Debug,Clone)]
// pub struct ZmqServer
// {
// // public ZmqSocketBase
//     pub socket_base: ZmqSocket,
//     // ZmqServer (ZmqContext *parent_, tid: u32, sid_: i32);
//     // ~ZmqServer ();
//     //  Overrides of functions from ZmqSocketBase.
//     // void xattach_pipe (pipe: &mut ZmqPipe,
//     //                    subscribe_to_all_: bool,
//     //                    locally_initiated_: bool);
//     // int xsend (msg: &mut ZmqMessage);
//     // int xrecv (msg: &mut ZmqMessage);
//     // bool xhas_in ();
//     // bool xhas_out ();
//     // void xread_activated (pipe: &mut ZmqPipe);
//     // void xwrite_activated (pipe: &mut ZmqPipe);
//     // void xpipe_terminated (pipe: &mut ZmqPipe);
//     //  Fair queueing object for inbound pipes.
//     // ZmqFq fair_queue;
//     pub fair_queue: ZmqFq,
//     //  Outbound pipes indexed by the peer IDs.
//     // typedef std::map<u32, ZmqOutpipe> out_pipes_t;
//     // out_pipes_t _out_pipes;
//     pub _out_pipes: HashMap<u32,ZmqOutpipe>,
//     //  Routing IDs are generated. It's a simple increment and wrap-over
//     //  algorithm. This value is the next ID to use (if not used already).
//     // u32 _next_routing_id;
//     pub _next_routing_id: u32,
//     // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqServer)
// }
// 
// impl ZmqServer {
//     pub fn new(options: &mut ZmqContext, parent: &mut ZmqContext, tid: u32, sid_: i32)  -> Self
// 
//     {
// //  ZmqSocketBase (parent_, tid, sid_, true),
// //     _next_routing_id (generate_random ())
//         options.type_ = ZMQ_SERVER as i32;
//         options.can_send_hello_msg = true;
//         options.can_recv_disconnect_msg = true;
//         options.can_recv_routing_id = true;
//         Self {
//             socket_base: ZmqSocket::new(parent, options, tid, sid_, false),
//             fair_queue: Default::default(),
//             _out_pipes: Default::default(),
//             _next_routing_id: 0,
//         }
//     }
// 
//     pub fn xattach_pipe (pipe: &mut ZmqPipe,
//                          subscribe_to_all_: bool,
//                          locally_initiated_: bool)
//     {
//         // LIBZMQ_UNUSED (subscribe_to_all_);
//         // LIBZMQ_UNUSED (locally_initiated_);
// 
//         // zmq_assert (pipe);
//         _next_routing_id+= 1;
//         let mut routing_id = _next_routing_id;
//         if (!routing_id) {
//             routing_id = _next_routing_id += 1;
//         } //  Never use Routing ID zero
// 
//         pipe.set_server_socket_routing_id (routing_id);
//         //  Add the record into output pipes lookup table
//         let outpipe = ZmqOutpipe{pipe: pipe.clone(), active: true };
//         let ok =
//             _out_pipes.ZMQ_MAP_INSERT_OR_EMPLACE (routing_id, outpipe).second;
//         // zmq_assert (ok);
// 
//         fair_queue.attach (pipe);
//     }
// 
//    
// }

pub fn server_xpipe_terminated (sock: &mut ZmqSocket, pipe: &mut ZmqPipe)
{
    let it =
       sock._out_pipes.find (pipe.get_server_socket_routing_id ());
    // zmq_assert (it != _out_pipes.end ());
   sock._out_pipes.erase (it);
    sock.fair_queue.pipe_terminated (pipe);
}

pub fn server_xread_activated (sock: &mut ZmqSocket, pipe: &mut ZmqPipe)
{
    sock.fair_queue.activated (pipe);
}

pub fn server_xwrite_activated (sock: &mut ZmqSocket, pipe: &mut ZmqPipe)
{
    let end =sock._out_pipes.end ();
    // out_pipes_t::iterator it;
    // for (it = _out_pipes.begin (); it != end; += 1it)
    for it in sock._out_pipes.iter_mut()
    {
        if (it.second.pipe == pipe) {
            break;
        }
    }

    // zmq_assert (it != _out_pipes.end ());
    // zmq_assert (!it.second.active);
    it.second.active = true;
}

pub fn server_xsend (sock: &mut ZmqSocket, msg: &mut ZmqMessage) -> i32
{
    //  SERVER sockets do not allow multipart data (ZMQ_SNDMORE)
    if msg.flags () & ZMQ_MSG_MORE {
        errno = EINVAL;
        return -1;
    }
    //  Find the pipe associated with the routing stored in the message.
    let routing_id = msg.get_routing_id ();
    let it =sock._out_pipes.find (routing_id);

    if it !=sock._out_pipes.end () {
        if !it.second.pipe.check_write () {
            it.second.active = false;
            errno = EAGAIN;
            return -1;
        }
    } else {
        errno = EHOSTUNREACH;
        return -1;
    }

    //  Message might be delivered over inproc, so we reset routing id
    let rc = msg.reset_routing_id ();
    // errno_assert (rc == 0);

    let ok = it.second.pipe.write (msg);
    if ( (!ok)) {
        // Message failed to send - we must close it ourselves.
        msg.close ();
        // errno_assert (rc == 0);
    } else {
        it.second.pipe.flush();
    }

    //  Detach the message from the data buffer.
    msg.init2 ();
    // errno_assert (rc == 0);

    return 0;
}

pub fn server_xrecv (sock: &mut ZmqSocket, msg: &mut ZmqMessage) -> i32
{
    ZmqPipe *pipe = null_mut();
    let rc = sock.fair_queue.recvpipe (msg, &pipe);

    // Drop any messages with more flag
    while rc == 0 && msg.flags () & ZMQ_MSG_MORE == 1 {
        // drop all frames of the current multi-frame message
        rc = sock.fair_queue.recvpipe (msg, null_mut());

        while (rc == 0 && msg.flags () & ZMQ_MSG_MORE == 1) {
            rc = sock.fair_queue.recvpipe(msg, null_mut());
        }

        // get the new message
        if (rc == 0) {
            rc = sock.fair_queue.recvpipe(msg, &pipe);
        }
    }

    if (rc != 0) {
        return rc;
    }

    // zmq_assert (pipe != null_mut());

    let routing_id = pipe.get_server_socket_routing_id ();
    msg.set_routing_id (routing_id);

    return 0;
}

pub fn server_xhas_in (sock: &mut ZmqSocket) -> bool
{
    return sock.fair_queue.has_in ();
}

pub fn server_xhas_out () -> bool
{
    //  In theory, SERVER socket is always ready for writing. Whether actual
    //  attempt to write succeeds depends on which pipe the message is going
    //  to be routed to.
    return true;
}


















