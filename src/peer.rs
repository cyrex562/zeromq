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

use bincode::options;
use libc::EFAULT;
use crate::context::ZmqContext;
use crate::defines::ZMQ_PEER;
use crate::options::ZmqOptions;
use crate::pipe::ZmqPipe;
use crate::server::ZmqServer;

// #include "precompiled.hpp"
// #include "macros.hpp"
// #include "peer.hpp"
// #include "pipe.hpp"
// #include "wire.hpp"
// #include "random.hpp"
// #include "likely.hpp"
// #include "err.hpp"
#[derive(Default, Debug, Clone)]
pub struct ZmqPeer {
    //   : public ZmqServer
    pub server: ZmqServer,
    // u32 _peer_last_routing_id;
    pub peer_last_routing_id: u32,

}

impl ZmqPeer {
    // ZmqPeer (ZmqContext *parent_, tid: u32, sid_: i32);
    pub fn new(&mut options: ZmqOptions, parent: &mut ZmqContext, tid: u32, sid_: i32) -> Self

    {
        options.type_ = ZMQ_PEER;
        options.can_send_hello_msg = true;
        options.can_recv_disconnect_msg = true;
        options.can_recv_hiccup_msg = true;
        options.can_recv_routing_id = true;
// ZmqServer (parent_, tid, sid_) -> Self
        Self {
            server: ZmqServer::new(parent, tid, sid_),
            ..Default::default()
        }
    }

    //  Overrides of functions from ZmqSocketBase.
    // void xattach_pipe (pipe: &mut ZmqPipe,
    // subscribe_to_all_: bool,
    // locally_initiated_: bool);
    pub fn xattach_pipe(&mut self, pipe: &mut ZmqPipe,
                        subscribe_to_all_: bool,
                        locally_initiated_: bool) {
        self.server.xattach_pipe(pipe, subscribe_to_all_, locally_initiated_);
        self._peer_last_routing_id = pipe.get_server_socket_routing_id();
    }

    // u32 connect_peer (endpoint_uri_: &str);
    pub fn connect_peer(&mut self, endpoint_uri_: &str) -> u32 {
        let mut sync_lock = scoped_optional_lock_t::new(&sync);

        // connect_peer cannot work with immediate enabled
        if options.immediate == 1 {
            errno = EFAULT;
            return 0;
        }

        let rc = self.server.connect_internal(endpoint_uri_);
        if rc != 0 {
            return 0;
        }

        return _peer_last_routing_id;
    }


    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqPeer)
}






