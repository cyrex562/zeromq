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
use bincode::options;
use libc::{EAGAIN, pipe};
use crate::context::ZmqContext;
use crate::defines::ZMQ_PAIR;
use crate::message::{ZMQ_MSG_MORE, ZmqMessage};

use crate::pipe::ZmqPipe;
use crate::socket::ZmqSocket;

// #[derive(Default, Debug, Clone)]
// pub struct ZmqPair<'a> {
//     //   : public ZmqSocketBase
//     pub socket_base: ZmqSocket,
// //
//
//
//     //
//     //   ZmqPipe *pipe;
//     pipe: Option<&'a mut ZmqPipe>,
//
//     // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqPair)
// }




pub fn pair_xsend(sock: &mut ZmqSocket, msg: &mut ZmqMessage) -> i32 {
    if (!pipe || !pipe.write(msg)) {
        errno = EAGAIN;
        return -1;
    }

    if (!(msg.flags() & ZMQ_MSG_MORE)) {
        pipe.flush();
    }

    //  Detach the original message from the data buffer.
    msg.init2();
    // errno_assert (rc == 0);

    return 0;
}

// int xrecv (msg: &mut ZmqMessage);
pub fn pair_xrecv(sock: &mut ZmqSocket, msg: &mut ZmqMessage) -> i32 {
    //  Deallocate old content of the message.
    let rc = msg.close();
    // errno_assert (rc == 0);

    if (!pipe || !pipe.read(msg)) {
        //  Initialise the output parameter to be a 0-byte message.
        msg.init2();
        // errno_assert (rc == 0);

        errno = EAGAIN;
        return -1;
    }
    return 0;
}

// bool xhas_in ();
pub fn pair_xhas_in() -> bool {
    if (!pipe) {
        return false;
    }

    return pipe.check_read();
}

// bool xhas_out ();
pub fn pair_xhas_out() -> bool {
    if (!pipe) {
        return false;
    }

    return pipe.check_write();
}

// void xread_activated (pipe: &mut ZmqPipe);
pub fn pair_xread_activated(sock: &mut ZmqSocket) {
    //  There's just one pipe. No lists of active and inactive pipes.
    //  There's nothing to do here.
    unimplemented!()
}

// void xwrite_activated (pipe: &mut ZmqPipe);
pub fn pair_xwrite_activated(sock: &mut ZmqSocket) {
    //  There's just one pipe. No lists of active and inactive pipes.
    //  There's nothing to do here.
    unimplemented!()
}

// void xpipe_terminated (pipe: &mut ZmqPipe);
pub fn pair_xpipe_terminated(sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    if self.pipe.unwrap() == pipe {
        self.pipe = None;
    }
}

// ZmqPair::~ZmqPair ()
// {
//     // zmq_assert (!pipe);
// }














