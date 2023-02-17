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

pull_t::pull_t (class ZmqContext *parent_, u32 tid_, sid_: i32) :
    ZmqSocketBase (parent_, tid_, sid_)
{
    options.type = ZMQ_PULL;
}

pull_t::~pull_t ()
{
}

void pull_t::xattach_pipe (pipe_t *pipe_,
                                bool subscribe_to_all_,
                                bool locally_initiated_)
{
    LIBZMQ_UNUSED (subscribe_to_all_);
    LIBZMQ_UNUSED (locally_initiated_);

    zmq_assert (pipe_);
    _fq.attach (pipe_);
}

void pull_t::xread_activated (pipe_t *pipe_)
{
    _fq.activated (pipe_);
}

void pull_t::xpipe_terminated (pipe_t *pipe_)
{
    _fq.pipe_terminated (pipe_);
}

int pull_t::xrecv (ZmqMessage *msg)
{
    return _fq.recv (msg);
}

bool pull_t::xhas_in ()
{
    return _fq.has_in ();
}
pub struct pull_t ZMQ_FINAL : public ZmqSocketBase
{
// public:
    pull_t (ZmqContext *parent_, u32 tid_, sid_: i32);
    ~pull_t ();

  protected:
    //  Overrides of functions from ZmqSocketBase.
    void xattach_pipe (pipe_t *pipe_,
                       bool subscribe_to_all_,
                       bool locally_initiated_);
    int xrecv (ZmqMessage *msg);
    bool xhas_in ();
    void xread_activated (pipe_t *pipe_);
    void xpipe_terminated (pipe_t *pipe_);

  // private:
    //  Fair queueing object for inbound pipes.
    fq_t _fq;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (pull_t)
};
