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
// #include "push.hpp"
// #include "pipe.hpp"
// #include "err.hpp"
// #include "msg.hpp"
pub struct push_t ZMQ_FINAL : public ZmqSocketBase
{
// public:
    push_t (ZmqContext *parent_, u32 tid_, sid_: i32);
    ~push_t ();

  protected:
    //  Overrides of functions from ZmqSocketBase.
    void xattach_pipe (pipe_t *pipe,
                       subscribe_to_all_: bool,
                       locally_initiated_: bool);
    int xsend (msg: &mut ZmqMessage);
    bool xhas_out ();
    void xwrite_activated (pipe: &mut pipe_t);
    void xpipe_terminated (pipe: &mut pipe_t);

  // private:
    //  Load balancer managing the outbound pipes.
    lb_t _lb;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (push_t)
};

push_t::push_t (class ZmqContext *parent_, u32 tid_, sid_: i32) :
    ZmqSocketBase (parent_, tid_, sid_)
{
    options.type = ZMQ_PUSH;
}

push_t::~push_t ()
{
}

void push_t::xattach_pipe (pipe_t *pipe,
                                subscribe_to_all_: bool,
                                locally_initiated_: bool)
{
    LIBZMQ_UNUSED (subscribe_to_all_);
    LIBZMQ_UNUSED (locally_initiated_);

    //  Don't delay pipe termination as there is no one
    //  to receive the delimiter.
    pipe.set_nodelay ();

    zmq_assert (pipe);
    _lb.attach (pipe);
}

void push_t::xwrite_activated (pipe: &mut pipe_t)
{
    _lb.activated (pipe);
}

void push_t::xpipe_terminated (pipe: &mut pipe_t)
{
    _lb.pipe_terminated (pipe);
}

int push_t::xsend (msg: &mut ZmqMessage)
{
    return _lb.send (msg);
}

bool push_t::xhas_out ()
{
    return _lb.has_out ();
}
