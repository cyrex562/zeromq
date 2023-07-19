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

use crate::context::ZmqContext;
use crate::defines::{ZMQ_RADIO, ZMQ_XPUB_NODROP};
use crate::dist::ZmqDist;
use crate::message::{ZmqMessage, ZMQ_MSG_MORE};

use crate::pipe::ZmqPipe;

use crate::socket::ZmqSocket;
use libc::{EAGAIN, EINVAL, ENOTSUP};
use std::collections::HashMap;

// #include "radio.hpp"
// #include "macros.hpp"
// #include "pipe.hpp"
// #include "err.hpp"
// #include "msg.hpp"
// #[derive(Default, Debug, Clone)]
// pub struct ZmqRadio {
//     pub _subscriptions: HashMap<String, ZmqPipe>,
//     //  List of udp pipes
//     pub _udp_pipes: Vec<ZmqPipe>,
//     //  Distributor of messages holding the list of outbound pipes.
//     pub _dist: ZmqDist,
//     //  Drop messages if HWM reached, otherwise return with EAGAIN
//     pub _lossy: bool,
// }

pub fn radio_xread_activated(sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    //  There are some subscriptions waiting. Let's process them.
    let mut msg = ZmqMessage::default();
    while pipe.read(&mut msg) {
        //  Apply the subscription to the trie
        if msg.is_join() || msg.is_leave() {
            let group = (msg.group());

            if (msg.is_join()) {
                sock._subscriptions
                    .ZMQ_MAP_INSERT_OR_EMPLACE(ZMQ_MOVE(group), pipe);
            } else {
                // std::pair<subscriptions_t::iterator, subscriptions_t::iterator>
                //     range = _subscriptions.equal_range (Group);

                // for (subscriptions_t::iterator it = range.first;
                //     it != range.second; += 1it)
                for it in sock._subscriptions {
                    if (it.second == pipe) {
                        sock._subscriptions.erase(it);
                        break;
                    }
                }
            }
        }
        msg.close();
    }
}

pub fn radio_xwrite_activated(sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    sock._dist.activated(pipe);
}

pub fn radio_xsetsockopt(
    sock: &mut ZmqSocket,
    option_: i32,
    optval_: &mut [u8],
    optvallen_: usize,
) -> anyhow::Result<()> {
    if option_ == ZMQ_XPUB_NODROP {
        _lossy = ((optval_) == 0);
    } else {
      // errno = EINVAL;
        return Err("ZmqRadio::xsetsockopt".into());
    }
    return Ok(());
}

pub fn radio_xpipe_terminated(sock: &mut ZmqSocket, pipe: &mut ZmqPipe) {
    // for (subscriptions_t::iterator it = _subscriptions.begin (),
    //                                end = _subscriptions.end ();
    //      it != end;)
    for it in sock._subscriptions {
        if it.second == pipe {
            // #if __cplusplus >= 201103L || (defined _MSC_VER && _MSC_VER >= 1700)
            it = sock._subscriptions.erase(it);
            // #else
            //             _subscriptions.erase (it+= 1);
            // #endif
        } else {
            // += 1it;
        }
    }

    {
        let end = sock._udp_pipes.end();
        let it = (sock._udp_pipes.begin(), end, pipe);
        if (it != end) {
            sock._udp_pipes.erase(it);
        }
    }

    sock._dist.pipe_terminated(pipe);
}

pub fn radio_xsend(sock: &mut ZmqSocket, msg: &mut ZmqMessage) -> i32 {
    //  Radio sockets do not allow multipart data (ZMQ_SNDMORE)
    if msg.flags() & ZMQ_MSG_MORE {
      // errno = EINVAL;
        return -1;
    }

    _dist.unmatch();

    let range = _subscriptions.equal_range(std::string(msg.group()));

    // for (subscriptions_t::iterator it = range.first; it != range.second; += 1it)
    for it in sock._subscriptions {
        // _dist.
        // match (it.second);
    }

    // for (udp_pipes_t::iterator it = _udp_pipes.begin (),
    //                            end = _udp_pipes.end ();
    //      it != end; += 1it)
    for it in sock._udp_pipes {
        // _dist.
        // match (*it);
    }

    let mut rc = -1;
    if _lossy || _dist.check_hwm() {
        if _dist.send_to_matching(msg) == 0 {
            rc = 0; //  Yay, sent successfully
        }
    } else {
      // errno = EAGAIN;
    }

    return rc;
}

pub fn radio_xhas_out(sock: &mut ZmqSocket) {
    return _dist.has_out();
}

pub fn radio_xrecv(sock: &mut ZmqSocket, msg: &mut ZmqMessage) -> i32 {
    //  Messages cannot be received from PUB socket.
    LIBZMQ_UNUSED(msg);
  // errno = ENOTSUP;
    return -1;
}

pub fn radio_xhas_in(sock: &mut ZmqSocket) -> bool {
    return false;
}
