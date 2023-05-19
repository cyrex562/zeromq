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

use libc::{EAGAIN, EINVAL, ENOTSUP};
use crate::address::Address;
use crate::context::ZmqContext;
use crate::defines::{ZMQ_RADIO, ZMQ_XPUB_NODROP};
use crate::dist::ZmqDist;
use crate::io_thread::ZmqIoThread;
use crate::message::{ZMQ_MSG_COMMAND, ZMQ_MSG_MORE, ZmqMessage};
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::session_base::ZmqSessionBase;
use crate::socket_base::ZmqSocketBase;
use crate::utils::{cmp_bytes, copy_bytes};

// #include "radio.hpp"
// #include "macros.hpp"
// #include "pipe.hpp"
// #include "err.hpp"
// #include "msg.hpp"
#[derive(Default,Debug,Clone)]
pub struct ZmqRadio
{
//  : public ZmqSocketBase
//     ZmqRadio (ZmqContext *parent_, tid: u32, sid_: i32);
//     ~ZmqRadio ();
    //  Implementations of virtual functions from ZmqSocketBase.
    // void xattach_pipe (pipe: &mut ZmqPipe,
    //                    bool subscribe_to_all_ = false,
    //                    bool locally_initiated_ = false);
    // int xsend (msg: &mut ZmqMessage);
    // bool xhas_out ();
    // int xrecv (msg: &mut ZmqMessage);
    // bool xhas_in ();
    // void xread_activated (pipe: &mut ZmqPipe);
    // void xwrite_activated (pipe: &mut ZmqPipe);
    // int xsetsockopt (option_: i32, const optval_: &mut [u8], optvallen_: usize);
    // void xpipe_terminated (pipe: &mut ZmqPipe);
    //  List of all subscriptions mapped to corresponding pipes.
    // typedef std::multimap<std::string, ZmqPipe *> subscriptions_t;
    // subscriptions_t _subscriptions;
    pub _subscriptions: HashMap<String, ZmqPipe>,
    //  List of udp pipes
    // typedef std::vector<ZmqPipe *> udp_pipes_t;
    // udp_pipes_t _udp_pipes;
    pub _udp_pipes: Vec<ZmqPipe>,
    //  Distributor of messages holding the list of outbound pipes.
    pub _dist: ZmqDist,
    //  Drop messages if HWM reached, otherwise return with EAGAIN
    pub _lossy: bool,
    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqRadio)
}

impl ZmqRadio {
    pub fn new(options: &mut ZmqOptions, parent: &mut ZmqContext, tid: u32, sid_: i32) -> Self

    {
// ZmqSocketBase (parent_, tid, sid_, true), _lossy (true)
        let mut out = Self {
            _subscriptions: (),
            _udp_pipes: (),
            _dist: ZmqDist::default(),
            _lossy: false,
        };
        out.session_base.options.type_ = ZMQ_RADIO;
        out
    }

    pub fn xattach_pipe (&mut self,
                         pipe: &mut ZmqPipe,
                         subscribe_to_all_: bool,
                         locally_initiated_: bool)
    {
        // LIBZMQ_UNUSED (subscribe_to_all_);
        // LIBZMQ_UNUSED (locally_initiated_);

        // zmq_assert (pipe);

        //  Don't delay pipe termination as there is no one
        //  to receive the delimiter.
        pipe.set_nodelay ();

        _dist.attach (pipe);

        if (subscribe_to_all_) {
            _udp_pipes.push_back(pipe);
        }
        //  The pipe is active when attached. Let's read the subscriptions from
        //  it, if any.
        else {
            xread_activated(pipe);
        }
    }

    pub fn xread_activated (&mut self, pipe: &mut ZmqPipe)
    {
        //  There are some subscriptions waiting. Let's process them.
        let mut msg = ZmqMessage::default();
        while pipe.read (&mut msg) {
            //  Apply the subscription to the trie
            if msg.is_join () || msg.is_leave () {
                let group = (msg.group ());

                if (msg.is_join ()) {
                    _subscriptions.ZMQ_MAP_INSERT_OR_EMPLACE(ZMQ_MOVE(group),
                                                             pipe);
                }
                else {
                    // std::pair<subscriptions_t::iterator, subscriptions_t::iterator>
                    //     range = _subscriptions.equal_range (group);

                    // for (subscriptions_t::iterator it = range.first;
                    //     it != range.second; += 1it)
                    for it in self._subscriptions
                    {
                        if (it.second == pipe) {
                            _subscriptions.erase (it);
                            break;
                        }
                    }
                }
            }
            msg.close ();
        }
    }

    pub fn xwrite_activated (&mut self, pipe: &mut ZmqPipe)
    {
        _dist.activated (pipe);
    }

    pub fn xsetsockopt (&mut self, option_: i32, optval_: &mut [u8], optvallen_: usize) -> i32
    {
    if optvallen_ != 4 ||  (optval_) < 0 {
    errno = EINVAL;
    return -1;
    }
    if (option_ == ZMQ_XPUB_NODROP) {
        _lossy = ((optval_) == 0);
    }
    else {
    errno = EINVAL;
    return -1;
    }
    return 0;
    }

    pub fn xpipe_terminated (&mut self, pipe: &mut ZmqPipe)
    {
        // for (subscriptions_t::iterator it = _subscriptions.begin (),
        //                                end = _subscriptions.end ();
        //      it != end;)
        for it in self._subscriptions
        {
            if it.second == pipe {
// #if __cplusplus >= 201103L || (defined _MSC_VER && _MSC_VER >= 1700)
                it = _subscriptions.erase (it);
// #else
//             _subscriptions.erase (it+= 1);
// #endif
            } else {
                // += 1it;
            }
        }

        {
            let end = _udp_pipes.end ();
            let it = (_udp_pipes.begin (), end, pipe);
            if (it != end) {
                _udp_pipes.erase(it);
            }
        }

        _dist.pipe_terminated (pipe);
    }


    pub fn xsend (&mut self, msg: &mut ZmqMessage) -> i32
    {
        //  Radio sockets do not allow multipart data (ZMQ_SNDMORE)
        if (msg.flags () & ZMQ_MSG_MORE) {
            errno = EINVAL;
            return -1;
        }

        _dist.unmatch ();

        let range = _subscriptions.equal_range (std::string (msg.group ()));

        // for (subscriptions_t::iterator it = range.first; it != range.second; += 1it)
        for it in self._subscriptions
        {
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
        if _lossy || _dist.check_hwm () {
            if _dist.send_to_matching (msg) == 0 {
                rc = 0; //  Yay, sent successfully
            }
        } else {
            errno = EAGAIN;
        }

        return rc;
    }

    pub fn xhas_out (&mut self)
    {
        return _dist.has_out ();
    }

    pub fn xrecv (&mut self, msg: &mut ZmqMessage) -> i32
    {
        //  Messages cannot be received from PUB socket.
        LIBZMQ_UNUSED (msg);
        errno = ENOTSUP;
        return -1;
    }

    pub fn xhas_in (&mut self) -> bool
    {
        return false;
    }
}

pub enum RadioSessionState
{
    group,
    body
}

pub struct RadioSession
{
    // : public ZmqSessionBase
    pub session_base: ZmqSessionBase,
    //
    //     RadioSession (ZmqIoThread *io_thread_,
    //                      connect_: bool,
    //                      socket: *mut ZmqSocketBase,
    //                      options: &ZmqOptions,
    //                      Address *addr_);
    //     ~RadioSession ();
    //  Overrides of the functions from ZmqSessionBase.
    // int push_msg (msg: &mut ZmqMessage);
    // int pull_msg (msg: &mut ZmqMessage);
    // void reset ();
    // ZmqMessage _pending_msg;
    pub _pending_msg: ZmqMessage,
    // ZMQ_NON_COPYABLE_NOR_MOVABLE (RadioSession)
}

impl RadioSession {
    pub fn new(ctx: &mut ZmqContext,
               io_thread_: &mut ZmqIoThread,
               connect_: bool,
               socket: &mut ZmqSocketBase,
               options: &mut ZmqOptions,
               addr_: &mut Address) -> Self

    {
        // ZmqSessionBase (io_thread_, connect_, socket, options_, addr_),
        //     _state (group)
        Self {
            session_base: ZmqSessionBase::new(ctx, io_thread_, connect_, socket, options, addr_),
            _pending_msg: ZmqMessage::new(),
        }
    }


    pub fn push_msg (&mut self, msg: &mut ZmqMessage) -> i32
    {
        if (msg.flags () & ZMQ_MSG_COMMAND) {
            char *command_data =  (msg.data ());
            let data_size = msg.size ();

            group_length: i32;
            let mut group: String = String::new();

            let mut join_leave_msg: ZmqMessage = ZmqMessage::default();
            rc: i32;

            //  Set the msg type to either JOIN or LEAVE
            if data_size >= 5 && cmp_bytes (command_data, 0, b"\x04JOIN", 0, 5) == 0 {
                group_length =  (data_size) - 5;
                group = command_data + 5;
                rc = join_leave_msg.init_join ();
            } else if data_size >= 6 && cmp_bytes (command_data, 0, b"\x05LEAVE", 0, 6) == 0 {
                group_length =  (data_size) - 6;
                group = command_data + 6;
                rc = join_leave_msg.init_leave ();
            }
            //  If it is not a JOIN or LEAVE just push the message
            else {
                return self.session_base.push_msg(msg);
            }

            // errno_assert (rc == 0);

            //  Set the group
            rc = join_leave_msg.set_group (group);
            // errno_assert (rc == 0);

            //  Close the current command
            rc = msg.close ();
            // errno_assert (rc == 0);

            //  Push the join or leave command
            *msg = join_leave_msg;
            return self.session_base.push_msg (msg);
        }
        return self.session_base.push_msg (msg);
    }

    pub fn pull_msg (&mut self, msg: &mut ZmqMessage) -> i32
    {
        if _state == RadioSessionState::group {
            int rc = self.session_base.pull_msg (&mut _pending_msg);
            if (rc != 0) {
                return rc;
            }

            let group = _pending_msg.group ();
            let length: i32 =  group.len();

            //  First frame is the group
            rc = msg.init_size (length as usize);
            // errno_assert (rc == 0);
            msg.set_flags (ZMQ_MSG_MORE);
            copy_bytes (msg.data_mut(), 0, group,0, length.clone());

            //  Next status is the body
            _state = RadioSessionState::body;
            return 0;
        }
        *msg = _pending_msg;
        _state = RadioSessionState::group;
        return 0;
    }

    pub fn reset (&mut self)
    {
        self.session_base.reset ();
        _state = RadioSessionState::group;
    }
}





