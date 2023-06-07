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
// #include <string.h>

use crate::address::ZmqAddress;
use crate::context::ZmqContext;
use crate::defines::{ZMQ_RADIO, ZMQ_XPUB_NODROP};
use crate::dist::ZmqDist;
use crate::message::{ZmqMessage, ZMQ_MSG_COMMAND, ZMQ_MSG_MORE};

use crate::pipe::ZmqPipe;
use crate::session_base::ZmqSessionBase;
use crate::socket::ZmqSocket;
use crate::thread_context::ZmqThreadContext;
use crate::udp_address::UdpAddress;
use crate::utils::{cmp_bytes, copy_bytes};
use libc::{EAGAIN, EINVAL, ENOTSUP};
use std::collections::HashMap;

// #include "radio.hpp"
// #include "macros.hpp"
// #include "pipe.hpp"
// #include "err.hpp"
// #include "msg.hpp"
#[derive(Default, Debug, Clone)]
pub struct ZmqRadio {
    pub _subscriptions: HashMap<String, ZmqPipe>,
    //  List of udp pipes
    pub _udp_pipes: Vec<ZmqPipe>,
    //  Distributor of messages holding the list of outbound pipes.
    pub _dist: ZmqDist,
    //  Drop messages if HWM reached, otherwise return with EAGAIN
    pub _lossy: bool,
}

impl ZmqRadio {
    pub fn new(options: &mut ZmqContext, parent: &mut ZmqContext, tid: u32, sid_: i32) -> Self {
        // ZmqSocketBase (parent_, tid, sid_, true), _lossy (true)
        let mut out = Self {
            _subscriptions: HashMap::new(),
            _udp_pipes: vec![],
            _dist: ZmqDist::default(),
            _lossy: false,
        };
        out.session_base.options.type_ = ZMQ_RADIO;
        out
    }

    pub fn xattach_pipe(
        &mut self,
        pipe: &mut ZmqPipe,
        subscribe_to_all_: bool,
        locally_initiated_: bool,
    ) {
        //  Don't delay pipe termination as there is no one
        //  to receive the delimiter.
        pipe.set_nodelay();

        _dist.attach(pipe);

        if (subscribe_to_all_) {
            _udp_pipes.push_back(pipe);
        }
        //  The pipe is active when attached. Let's read the subscriptions from
        //  it, if any.
        else {
            xread_activated(pipe);
        }
    }

    pub fn xread_activated(&mut self, pipe: &mut ZmqPipe) {
        //  There are some subscriptions waiting. Let's process them.
        let mut msg = ZmqMessage::default();
        while pipe.read(&mut msg) {
            //  Apply the subscription to the trie
            if msg.is_join() || msg.is_leave() {
                let group = (msg.group());

                if (msg.is_join()) {
                    _subscriptions.ZMQ_MAP_INSERT_OR_EMPLACE(ZMQ_MOVE(group), pipe);
                } else {
                    // std::pair<subscriptions_t::iterator, subscriptions_t::iterator>
                    //     range = _subscriptions.equal_range (group);

                    // for (subscriptions_t::iterator it = range.first;
                    //     it != range.second; += 1it)
                    for it in self._subscriptions {
                        if (it.second == pipe) {
                            _subscriptions.erase(it);
                            break;
                        }
                    }
                }
            }
            msg.close();
        }
    }

    pub fn xwrite_activated(&mut self, pipe: &mut ZmqPipe) {
        _dist.activated(pipe);
    }

    pub fn xsetsockopt(
        &mut self,
        option_: i32,
        optval_: &mut [u8],
        optvallen_: usize,
    ) -> anyhow::Result<()> {
        if option_ == ZMQ_XPUB_NODROP {
            _lossy = ((optval_) == 0);
        } else {
            errno = EINVAL;
            return Err("ZmqRadio::xsetsockopt".into());
        }
        return Ok(());
    }

    pub fn xpipe_terminated(&mut self, pipe: &mut ZmqPipe) {
        // for (subscriptions_t::iterator it = _subscriptions.begin (),
        //                                end = _subscriptions.end ();
        //      it != end;)
        for it in self._subscriptions {
            if it.second == pipe {
                // #if __cplusplus >= 201103L || (defined _MSC_VER && _MSC_VER >= 1700)
                it = _subscriptions.erase(it);
            // #else
            //             _subscriptions.erase (it+= 1);
            // #endif
            } else {
                // += 1it;
            }
        }

        {
            let end = _udp_pipes.end();
            let it = (_udp_pipes.begin(), end, pipe);
            if (it != end) {
                _udp_pipes.erase(it);
            }
        }

        _dist.pipe_terminated(pipe);
    }

    pub fn xsend(&mut self, msg: &mut ZmqMessage) -> i32 {
        //  Radio sockets do not allow multipart data (ZMQ_SNDMORE)
        if (msg.flags() & ZMQ_MSG_MORE) {
            errno = EINVAL;
            return -1;
        }

        _dist.unmatch();

        let range = _subscriptions.equal_range(std::string(msg.group()));

        // for (subscriptions_t::iterator it = range.first; it != range.second; += 1it)
        for it in self._subscriptions {
            // _dist.
            // match (it.second);
        }

        // for (udp_pipes_t::iterator it = _udp_pipes.begin (),
        //                            end = _udp_pipes.end ();
        //      it != end; += 1it)
        for it in self._udp_pipes {
            // _dist.
            // match (*it);
        }

        let mut rc = -1;
        if _lossy || _dist.check_hwm() {
            if _dist.send_to_matching(msg) == 0 {
                rc = 0; //  Yay, sent successfully
            }
        } else {
            errno = EAGAIN;
        }

        return rc;
    }

    pub fn xhas_out(&mut self) {
        return _dist.has_out();
    }

    pub fn xrecv(&mut self, msg: &mut ZmqMessage) -> i32 {
        //  Messages cannot be received from PUB socket.
        LIBZMQ_UNUSED(msg);
        errno = ENOTSUP;
        return -1;
    }

    pub fn xhas_in(&mut self) -> bool {
        return false;
    }
}

pub enum RadioSessionState {
    group,
    body,
}

pub struct RadioSession {
    pub session_base: ZmqSessionBase,
    pub _pending_msg: ZmqMessage,
}

impl RadioSession {
    pub fn new(
        ctx: &mut ZmqContext,
        io_thread_: &mut ZmqThreadContext,
        connect_: bool,
        socket: &mut ZmqSocket,
        options: &mut ZmqContext,
        addr_: &mut UdpAddress,
    ) -> Self {
        Self {
            session_base: ZmqSessionBase::new(ctx, io_thread_, connect_, socket, options, addr_),
            _pending_msg: ZmqMessage::new(),
        }
    }

    pub fn push_msg(&mut self, msg: &mut ZmqMessage) -> i32 {
        if (msg.flags() & ZMQ_MSG_COMMAND) {
            char * command_data = (msg.data());
            let data_size = msg.size();

            group_length: i32;
            let mut group: String = String::new();

            let mut join_leave_msg: ZmqMessage = ZmqMessage::default();
            rc: i32;

            //  Set the msg type to either JOIN or LEAVE
            if data_size >= 5 && cmp_bytes(command_data, 0, b"\x04JOIN", 0, 5) == 0 {
                group_length = (data_size) - 5;
                group = command_data + 5;
                rc = join_leave_msg.init_join();
            } else if data_size >= 6 && cmp_bytes(command_data, 0, b"\x05LEAVE", 0, 6) == 0 {
                group_length = (data_size) - 6;
                group = command_data + 6;
                rc = join_leave_msg.init_leave();
            }
            //  If it is not a JOIN or LEAVE just push the message
            else {
                return self.session_base.push_msg(msg);
            }

            // errno_assert (rc == 0);

            //  Set the group
            rc = join_leave_msg.set_group(group);
            // errno_assert (rc == 0);

            //  Close the current command
            rc = msg.close();
            // errno_assert (rc == 0);

            //  Push the join or leave command
            *msg = join_leave_msg;
            return self.session_base.push_msg(msg);
        }
        return self.session_base.push_msg(msg);
    }

    pub fn pull_msg(&mut self, msg: &mut ZmqMessage) -> anyhow::Result<()> {
        if _state == RadioSessionState::group {
            self.session_base.pull_msg(&mut _pending_msg)?;

            let group = _pending_msg.group();
            let length: usize = group.len();

            //  First frame is the group
            rc = msg.init_size(length as usize);
            // errno_assert (rc == 0);
            msg.set_flags(ZMQ_MSG_MORE);
            copy_bytes(msg.data_mut(), 0, group, 0, length.clone());

            //  Next status is the body
            self._state = RadioSessionState::body;
            return Ok(());
        }
        *msg = self._pending_msg;
        self._state = RadioSessionState::group;
        Ok(())
    }

    pub fn reset(&mut self) {
        self.session_base.reset();
        _state = RadioSessionState::group;
    }
}
