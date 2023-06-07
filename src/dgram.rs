/*
    Copyright (c) 2016 Contributors as noted in the AUTHORS file

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

use anyhow::bail;
use libc::{EAGAIN, EINVAL};

use crate::content::ZmqContent;
use crate::context::ZmqContext;
use crate::defines::ZMQ_DGRAM;
use crate::message::{ZmqMessage, ZMQ_MSG_MORE};
use crate::pipe::ZmqPipe;
use crate::socket::ZmqSocket;

// #include "precompiled.hpp"
// #include "macros.hpp"
// #include "dgram.hpp"
// #include "pipe.hpp"
// #include "wire.hpp"
// #include "random.hpp"
// #include "likely.hpp"
// #include "err.hpp"
#[derive(Copy, Clone, Debug)]
pub struct ZmqDgram<'a> {
    // public:

    // private:
    // ZmqPipe *pipe;
    pub pipe: Option<&'a mut ZmqPipe>,

    //  If true, more outgoing message parts are expected.
    pub _more_out: bool,

    // // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqDgram)
    pub socket_base: ZmqSocket,
}

impl ZmqDgram {
    // ZmqDgram (ZmqContext *parent_, tid: u32, sid_: i32);
    // ZmqDgram::ZmqDgram (parent: &mut ZmqContext, tid: u32, sid_: i32) :
    // ZmqSocketBase (parent_, tid, sid_), pipe (null_mut()), _more_out (false)
    // {
    //     options.type = ZMQ_DGRAM;
    //     options.raw_socket = true;
    // }
    pub fn new(parent_: &mut ZmqContext, tid: u32, sid_: i32) -> Self {
        let mut socket_base = ZmqSocket::new(parent_, tid, sid_, false);
        socket_base.options.type_ = ZMQ_DGRAM;
        socket_base.options.raw_socket = true;
        Self {
            socket_base,
            pipe: None,
            _more_out: false,
        }
    }

    // ~ZmqDgram ();
    // ZmqDgram::~ZmqDgram ()
    // {
    //     zmq_assert (!pipe);
    // }

    //  Overrides of functions from ZmqSocketBase.
    // void xattach_pipe (pipe: &mut ZmqPipe,
    //                    subscribe_to_all_: bool,
    //                    locally_initiated_: bool);
    pub fn xattach_pipe(
        &mut self,
        pipe: &mut ZmqPipe,
        subscribe_to_all_: bool,
        locally_initiated_: bool,
    ) {
        // LIBZMQ_UNUSED (subscribe_to_all_);
        // LIBZMQ_UNUSED (locally_initiated_);

        // zmq_assert (pipe);

        //  ZMQ_DGRAM socket can only be connected to a single peer.
        //  The socket rejects any further connection requests.
        if (self.pipe == None) {
            self.pipe = Some(pipe);
        } else {
            pipe.terminate(false);
        }
    }

    // int xsend (msg: &mut ZmqMessage);
    pub fn xsend(&mut self, msg: &mut ZmqMessage) -> anyhow::Result<()> {
        // If there's no out pipe, just drop it.
        if self.pipe.is_none() {
            msg.close()?;
            // errno_assert (rc == 0);
        }

        //  If this is the first part of the message it's the ID of the
        //  peer to send the message to.
        if (!self._more_out) {
            if (!(msg.flags() & ZMQ_MSG_MORE)) {
                errno = EINVAL;
                bail!("EINVAL");
            }
        } else {
            //  dgram messages are two part only, reject part if more is set
            if (msg.flags() & ZMQ_MSG_MORE) {
                errno = EINVAL;
                bail!("EINVAL");
            }
        }

        // Push the message into the pipe.
        if (!unsafe { self.pipe.write(msg) }) {
            errno = EAGAIN;
            bail!("EAGAIN");
        }

        if (!(msg.flags() & ZMQ_MSG_MORE)) {
            self.pipe.flush();
        }

        // flip the more flag
        self._more_out = !self._more_out;

        //  Detach the message from the data buffer.
        msg.init2()?;
        // errno_assert(rc == 0);

        Ok(())
    }

    // int xrecv (msg: &mut ZmqMessage);
    pub fn xrecv(&mut self, msg: &mut ZmqMessage) -> i32 {
        //  Deallocate old content of the message.
        let mut rc = msg.close();
        // errno_assert (rc == 0);

        if (self.pipe.is_none() || !self.pipe.unwrap().read(msg)) {
            //  Initialise the output parameter to be a 0-byte message.
            msg.init2();
            errno_assert(rc == 0);

            errno = EAGAIN;
            return -1;
        }

        return 0;
    }

    // bool xhas_in ();
    pub fn xhas_in(&mut self) -> bool {
        if (self.pipe.is_none()) {
            return false;
        }

        return self.pipe.unwrap().check_read();
    }

    // bool xhas_out ();

    // void xread_activated (pipe: &mut ZmqPipe);
    pub fn xread_activated(&mut self, pipe: *mut ZmqPipe) {
        //  There's just one pipe. No lists of active and inactive pipes.
        //  There's nothing to do here.
        unimplemented!("xread_activated")
    }

    // void xwrite_activated (pipe: &mut ZmqPipe);
    pub fn xwrite_activated(&mut self, pipe: *mut ZmqPipe) {
        //  There's just one pipe. No lists of active and inactive pipes.
        //  There's nothing to do here.
        unimplemented!("xwrite_activated")
    }

    // void xpipe_terminated (pipe: &mut ZmqPipe);
    pub fn xpipe_terminated(&mut self, pipe: &mut ZmqPipe) {
        if (pipe == self.pipe) {
            self.pipe = None;
        }
    }

    pub fn xhas_out(&mut self) -> bool {
        if (self.pipe.is_none()) {
            return false;
        }

        return self.pipe.check_write();
    }
}
