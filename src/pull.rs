/*
    Copyright (c) 2007-2016 Contributors as noted in the AUTHORS file

    This file is part of libzmq, the ZeroMQ core engine in C++.

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
// #include "pull.hpp"
// #include "err.hpp"
// #include "msg.hpp"
// #include "pipe.hpp"

pull_t::pull_t (parent: &mut ZmqContext, tid: u32, sid_: i32) :
    ZmqSocketBase (parent_, tid, sid_)
{
    options.type = ZMQ_PULL;
}

pull_t::~pull_t ()
{
}

void pull_t::xattach_pipe (pipe: &mut ZmqPipe,
                                subscribe_to_all_: bool,
                                locally_initiated_: bool)
{
    LIBZMQ_UNUSED (subscribe_to_all_);
    LIBZMQ_UNUSED (locally_initiated_);

    zmq_assert (pipe);
    fair_queue.attach (pipe);
}

void pull_t::xread_activated (pipe: &mut ZmqPipe)
{
    fair_queue.activated (pipe);
}

void pull_t::xpipe_terminated (pipe: &mut ZmqPipe)
{
    fair_queue.pipe_terminated (pipe);
}

int pull_t::xrecv (msg: &mut ZmqMessage)
{
    return fair_queue.recv (msg);
}

bool pull_t::xhas_in ()
{
    return fair_queue.has_in ();
}
pub struct pull_t ZMQ_FINAL : public ZmqSocketBase
{
// public:
    pull_t (ZmqContext *parent_, tid: u32, sid_: i32);
    ~pull_t ();

  protected:
    //  Overrides of functions from ZmqSocketBase.
    void xattach_pipe (pipe: &mut ZmqPipe,
                       subscribe_to_all_: bool,
                       locally_initiated_: bool);
    int xrecv (msg: &mut ZmqMessage);
    bool xhas_in ();
    void xread_activated (pipe: &mut ZmqPipe);
    void xpipe_terminated (pipe: &mut ZmqPipe);

  // private:
    //  Fair queueing object for inbound pipes.
    fq_t fair_queue;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (pull_t)
};
