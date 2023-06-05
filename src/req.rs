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

use crate::address::ZmqAddress;
use crate::context::ZmqContext;
use crate::dealer::ZmqDealer;
use crate::defines::ZMQ_REQ;
use crate::message::{ZmqMessage, ZMQ_MSG_COMMAND, ZMQ_MSG_MORE};
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::session_base::ZmqSessionBase;
use crate::socket_base::ZmqSocketBase;
use crate::thread_context::ZmqThreadContext;
use crate::utils::copy_bytes;
use anyhow::anyhow;
use libc::EFAULT;
use std::ptr::null_mut;

// #include "precompiled.hpp"
// #include "macros.hpp"
// #include "req.hpp"
// #include "err.hpp"
// #include "msg.hpp"
// #include "wire.hpp"
// #include "random.hpp"
// #include "likely.hpp"
#[derive(Default, Debug, Clone)]
pub struct ZmqReq {
    //   : public ZmqDealer
    pub dealer: ZmqDealer,
    // ZmqReq (ZmqContext *parent_, tid: u32, sid_: i32);

    // ~ZmqReq ();

    //  Overrides of functions from ZmqSocketBase.
    // int xsend (msg: &mut ZmqMessage);

    // int xrecv (msg: &mut ZmqMessage);

    // bool xhas_in ();

    // bool xhas_out ();

    // int xsetsockopt (option_: i32, const optval_: &mut [u8], optvallen_: usize);

    // void xpipe_terminated (pipe: &mut ZmqPipe);

    //  Receive only from the pipe the request was sent to, discarding
    //  frames from other pipes.
    // int recv_reply_pipe (msg: &mut ZmqMessage);

    //
    //  If true, request was already sent and reply wasn't received yet or
    //  was received partially.
    pub _receiving_reply: bool,
    //  If true, we are starting to send/recv a message. The first part
    //  of the message must be empty message part (backtrace stack bottom).
    pub _message_begins: bool,
    //  The pipe the request was sent to and where the reply is expected.
    // ZmqPipe *_reply_pipe;
    pub _reply_pipe: Option<ZmqPipe>,
    //  Whether request id frames shall be sent and expected.
    pub _request_id_frames_enabled: bool,
    //  The current request id. It is incremented every time before a new
    //  request is sent.
    // u32 _request_id;
    pub _request_id: u32,
    //  If false, send() will reset its internal state and terminate the
    //  reply_pipe's connection instead of failing if a previous request is
    //  still pending.
    pub _strict: bool,
    // ZMQ_NON_COPYABLE_NOR_MOVABLE (req_t)
}

impl ZmqReq {
    pub fn new(options: &mut ZmqOptions, parent: &mut ZmqContext, tid: u32, sid_: i32) -> Self {
        // ZmqDealer (parent_, tid, sid_),
        //     _receiving_reply (false),
        //     _message_begins (true),
        //     _reply_pipe (null_mut()),
        //     _request_id_frames_enabled (false),
        //     _request_id (generate_random ()),
        //     _strict (true)
        let mut out = Self {
            dealer: ZmqDealer::new(options, parent, tid, sid),
            _receiving_reply: false,
            _message_begins: false,
            _reply_pipe: None,
            _request_id_frames_enabled: false,
            _request_id: 0,
            _strict: false,
        };
        out.dealer.options.type_ = ZMQ_REQ;
        out
    }

    pub fn xsend(&mut self, msg: &mut ZmqMessage) -> i32 {
        //  If we've sent a request and we still haven't got the reply,
        //  we can't send another request unless the strict option is disabled.
        if (self._receiving_reply) {
            if (self._strict) {
                errno = EFSM;
                return -1;
            }

            self._receiving_reply = false;
            self._message_begins = true;
        }

        //  First part of the request is the request routing id.
        if (self._message_begins) {
            self._reply_pipe = None;

            if (self._request_id_frames_enabled) {
                self._request_id += 1;

                let mut id: ZmqMessage = ZmqMessage::default();
                id.init_size(4);
                // memcpy (id.data (), &_request_id, mem::size_of::<u32>());
                id.data_mut() = &mut self._request_id.to_le_bytes();
                // errno_assert (rc == 0);
                id.set_flags(ZMQ_MSG_MORE);

                // TODO
                // rc = self.dealer.sendpipe (&mut id, &self._reply_pipe);
                // if (rc != 0) {
                //     return -1;
                // }
            }

            let mut bottom = ZmqMessage::default();
            bottom.init2();
            // errno_assert (rc == 0);
            bottom.set_flags(ZMQ_MSG_MORE);

            rc = self.dealer.sendpipe(&mut bottom, &mut _reply_pipe);
            if (rc != 0) {
                return -1;
            }
            // zmq_assert (_reply_pipe);

            self._message_begins = false;

            // Eat all currently available messages before the request is fully
            // sent. This is done to avoid:
            //   REQ sends request to A, A replies, B replies too.
            //   A's reply was first and matches, that is used.
            //   An hour later REQ sends a request to B. B's old reply is used.
            // ZmqMessage drop;
            let mut drop = ZmqMessage::default();
            loop {
                rc = drop.init2();
                // errno_assert (rc == 0);
                rc = self.dealer.xrecv(&mut drop);
                if (rc != 0) {
                    break;
                }
                drop.close();
            }
        }

        let more = (msg.flags() & ZMQ_MSG_MORE) != 0;

        self.dealer.xsend(msg);
        // if (rc != 0)
        // return rc;

        //  If the request was fully sent, flip the FSM into reply-receiving state.
        if (!more) {
            _receiving_reply = true;
            _message_begins = true;
        }

        return 0;
    }

    pub fn xrecv(&mut self, msg: &mut ZmqMessage) -> i32 {
        //  If request wasn't send, we can't wait for reply.
        if (self._receiving_reply) {
            errno = EFSM;
            return -1;
        }

        //  Skip messages until one with the right first frames is found.
        while (self._message_begins) {
            //  If enabled, the first frame must have the correct request_id.
            if _request_id_frames_enabled {
                let rc = recv_reply_pipe(msg);
                if (rc != 0) {
                    return rc;
                }

                if (!(msg.flags() & ZMQ_MSG_MORE) == 0
                    || msg.size() != 4
                    || (msg.data()) != _request_id)
                {
                    //  Skip the remaining frames and try the next message
                    while (msg.flags() & ZMQ_MSG_MORE) {
                        rc = recv_reply_pipe(msg);
                        // errno_assert (rc == 0);
                    }
                    continue;
                }
            }

            //  The next frame must be 0.
            // TODO: Failing this check should also close the connection with the peer!
            let rc = recv_reply_pipe(msg);
            if rc != 0 {
                return rc;
            }

            if !(msg.flags() & ZMQ_MSG_MORE) == 1 || msg.size() != 0 {
                //  Skip the remaining frames and try the next message
                while msg.flags() & ZMQ_MSG_MORE {
                    rc = recv_reply_pipe(msg);
                    // errno_assert (rc == 0);
                }
                continue;
            }

            _message_begins = false;
        }

        let rc: i32 = recv_reply_pipe(msg);
        if (rc != 0) {
            return rc;
        }

        //  If the reply is fully received, flip the FSM into request-sending state.
        if (!(msg.flags() & ZMQ_MSG_MORE)) {
            _receiving_reply = false;
            _message_begins = true;
        }

        return 0;
    }

    pub fn xhas_in(&mut self) -> bool {
        //  TODO: Duplicates should be removed here.

        if !self._receiving_reply {
            return false;
        }

        return self.dealer.xhas_in();
    }

    pub fn xhas_out(&mut self) -> bool {
        if (_receiving_reply && _strict) {
            return false;
        }

        return self.dealer.xhas_out();
    }

    pub fn xsetsockopt(&mut self, option_: i32, optval_: &mut [u8], optvallen_: usize) -> i32 {
        let is_int = (optvallen_ == 4);
        let mut value = 0;
        // TODO
        // if (is_int) {
        //     copy_bytes(&mut value, 0,optval_, 0,4);
        // }

        match (option_) {
            ZMQ_REQ_CORRELATE => {
                if (is_int && value >= 0) {
                    _request_id_frames_enabled = (value != 0);
                    return 0;
                }
            }

            ZMQ_REQ_RELAXED => {
                if (is_int && value >= 0) {
                    _strict = (value == 0);
                    return 0;
                }
            }

            _ => {}
        }

        return self.dealer.xsetsockopt(option_, optval_, optvallen_);
    }

    pub fn xpipe_terminated(&mut self, pipe: &mut ZmqPipe) {
        if (_reply_pipe == pipe) {
            _reply_pipe = null_mut();
        }
        self.dealer.xpipe_terminated(pipe);
    }

    pub fn recv_reply_pipe(&mut self, msg: &mut ZmqMessage) -> i32 {
        loop {
            let mut pipe: *mut ZmqPipe = null_mut();
            let rc: i32 = self.dealer.recvpipe(msg, &mut pipe);
            if (rc != 0) {
                return rc;
            }
            if (!_reply_pipe || pipe == _reply_pipe) {
                return 0;
            }
        }
    }
}

pub enum ReqSessionState {
    bottom,
    request_id,
    body,
}

#[derive(Default, Debug, Clone)]
pub struct ReqSession {
    // : public ZmqSessionBase
    pub session_base: ZmqSessionBase,
    // ReqSession (ZmqIoThread *io_thread_,
    //                connect_: bool,
    //                socket: *mut ZmqSocketBase,
    //                options: &ZmqOptions,
    //                Address *addr_);

    // ~ReqSession ();

    //  Overrides of the functions from ZmqSessionBase.
    // int push_msg (msg: &mut ZmqMessage);

    // void reset ();

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (req_session_t)
}

impl ReqSession {
    pub fn new(
        ctx: &mut ZmqContext,
        io_thread: &mut ZmqThreadContext,
        connect_: bool,
        socket: &mut ZmqSocketBase,
        options: &mut ZmqOptions,
        addr: &mut ZmqAddress,
    ) -> Self {
        //     ZmqSessionBase (io_thread_, connect_, socket, options_, addr_),
        //     _state (bottom)
        Self {
            session_base: ZmqSessionBase::new(ctx, io_thread, connect_, socket, options, addr),
        }
    }

    pub fn push_msg(&mut self, msg: &mut ZmqMessage) -> anyhow::Result<()> {
        //  Ignore commands, they are processed by the engine and should not
        //  affect the state machine.
        if (msg.flags() & ZMQ_MSG_COMMAND) {
            return Ok(());
        }

        match (_state) {
            ReqSessionState::bottom => {
                if (msg.flags() == ZMQ_MSG_MORE) {
                    //  In case option ZMQ_CORRELATE is on, allow request_id to be
                    //  transferred as first frame (would be too cumbersome to check
                    //  whether the option is actually on or not).
                    if (msg.size() == 4) {
                        _state = ReqSessionState::request_id;
                        return self.session_base.push_msg(msg);
                    }
                    if (msg.size() == 0) {
                        _state = ReqSessionState::body;
                        return self.session_base.push_msg(msg);
                    }
                }
            }

            ReqSessionState::request_id => {
                if (msg.flags() == ZMQ_MSG_MORE && msg.size() == 0) {
                    _state = ReqSessionState::body;
                    return self.session_base.push_msg(msg);
                }
            }

            ReqSessionState::body => {
                if msg.flags() == ZMQ_MSG_MORE {
                    return self.session_base.push_msg(msg);
                }
                if msg.flags() == 0 {
                    _state = ReqSessionState::bottom;
                    return self.session_base.push_msg(msg);
                }
            }
        }
        errno = EFAULT;
        return Err(anyhow!("EFAULT"));
    }

    pub fn reset(&mut self) {
        self.session_base.reset();
        _state = ReqSessionState::bottom;
    }
}
