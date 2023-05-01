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
// #include "pub.hpp"
// #include "pipe.hpp"
// #include "err.hpp"
// #include "msg.hpp"
pub struct pub_t ZMQ_FINAL : public xpub_t
{
// public:
    pub_t (ZmqContext *parent_, tid: u32, sid_: i32);
    ~pub_t ();

    //  Implementations of virtual functions from ZmqSocketBase.
    void xattach_pipe (pipe: &mut ZmqPipe,
                       bool subscribe_to_all_ = false,
                       bool locally_initiated_ = false);
    int xrecv (msg: &mut ZmqMessage);
    bool xhas_in ();

    ZMQ_NON_COPYABLE_NOR_MOVABLE (pub_t)
};

pub_t::pub_t (parent: &mut ZmqContext, tid: u32, sid_: i32) :
    xpub_t (parent_, tid, sid_)
{
    options.type = ZMQ_PUB;
}

pub_t::~pub_t ()
{
}

void pub_t::xattach_pipe (pipe: &mut ZmqPipe,
                               subscribe_to_all_: bool,
                               locally_initiated_: bool)
{
    zmq_assert (pipe);

    //  Don't delay pipe termination as there is no one
    //  to receive the delimiter.
    pipe.set_nodelay ();

    xpub_t::xattach_pipe (pipe, subscribe_to_all_, locally_initiated_);
}

int pub_t::xrecv (class ZmqMessage *)
{
    //  Messages cannot be received from PUB socket.
    errno = ENOTSUP;
    return -1;
}

bool pub_t::xhas_in ()
{
    return false;
}
