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



use crate::message::ZmqMessage;
use crate::socket::ZmqSocket;


// pub struct ZmqPub {
//     pub xpub: XPub,
//     // ZmqPub (ZmqContext *parent_, tid: u32, sid_: i32);
//
//     // ~ZmqPub ();
//
//     //  Implementations of virtual functions from ZmqSocketBase.
//     // void xattach_pipe (pipe: &mut ZmqPipe,
//     //                    bool subscribe_to_all_ = false,
//     //                    bool locally_initiated_ = false);
//
//     // int xrecv (msg: &mut ZmqMessage);
//
//     // bool xhas_in ();
//
//     // ZMQ_NON_COPYABLE_NOR_MOVABLE (pub_t)
// }

pub fn pub_xrecv(sock: &mut ZmqSocket, msg: &mut ZmqMessage) -> i32 {
    //  Messages cannot be received from PUB socket.
    // errno = ENOTSUP; return - 1;
    unimplemented!()
}

// TODO send should call xpub_send()

pub fn pub_xhas_in() -> bool {
    return false;
}