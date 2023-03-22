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
// #include "dealer.hpp"
// #include "err.hpp"
// #include "msg.hpp"
pub struct dealer_t : public ZmqSocketBase
{
// public:
    dealer_t (ZmqContext *parent_, tid: u32, sid_: i32);
    ~dealer_t () ZMQ_OVERRIDE;

  protected:
    //  Overrides of functions from ZmqSocketBase.
    void xattach_pipe (pipe: &mut ZmqPipe,
                       subscribe_to_all_: bool,
                       locally_initiated_: bool) ZMQ_FINAL;
    int xsetsockopt (option_: i32,
                     const optval_: &mut [u8],
                     optvallen_: usize) ZMQ_OVERRIDE;
    int xsend (msg: &mut ZmqMessage) ZMQ_OVERRIDE;
    int xrecv (msg: &mut ZmqMessage) ZMQ_OVERRIDE;
    bool xhas_in () ZMQ_OVERRIDE;
    bool xhas_out () ZMQ_OVERRIDE;
    void xread_activated (pipe: &mut ZmqPipe) ZMQ_FINAL;
    void xwrite_activated (pipe: &mut ZmqPipe) ZMQ_FINAL;
    void xpipe_terminated (pipe: &mut ZmqPipe) ZMQ_OVERRIDE;

    //  Send and recv - knowing which pipe was used.
    int sendpipe (msg: &mut ZmqMessage ZmqPipe **pipe);
    int recvpipe (msg: &mut ZmqMessage ZmqPipe **pipe);

  // private:
    //  Messages are fair-queued from inbound pipes. And load-balanced to
    //  the outbound pipes.
    fq_t _fq;
    lb_t _lb;

    // if true, send an empty message to every connected router peer
    _probe_router: bool

    ZMQ_NON_COPYABLE_NOR_MOVABLE (dealer_t)
};

dealer_t::dealer_t (class ZmqContext *parent_, tid: u32, sid_: i32) :
    ZmqSocketBase (parent_, tid, sid_), _probe_router (false)
{
    options.type = ZMQ_DEALER;
    options.can_send_hello_msg = true;
    options.can_recv_hiccup_msg = true;
}

dealer_t::~dealer_t ()
{
}

void dealer_t::xattach_pipe (pipe: &mut ZmqPipe,
                                  subscribe_to_all_: bool,
                                  locally_initiated_: bool)
{
    LIBZMQ_UNUSED (subscribe_to_all_);
    LIBZMQ_UNUSED (locally_initiated_);

    zmq_assert (pipe);

    if (_probe_router) {
        ZmqMessage probe_msg;
        int rc = probe_msg.init ();
        errno_assert (rc == 0);

        rc = pipe.write (&probe_msg);
        // zmq_assert (rc) is not applicable here, since it is not a bug.
        LIBZMQ_UNUSED (rc);

        pipe.flush ();

        rc = probe_msg.close ();
        errno_assert (rc == 0);
    }

    _fq.attach (pipe);
    _lb.attach (pipe);
}

int dealer_t::xsetsockopt (option_: i32,
                                const optval_: &mut [u8],
                                optvallen_: usize)
{
    const bool is_int = (optvallen_ == mem::size_of::<int>());
    int value = 0;
    if (is_int)
        memcpy (&value, optval_, mem::size_of::<int>());

    switch (option_) {
        case ZMQ_PROBE_ROUTER:
            if (is_int && value >= 0) {
                _probe_router = (value != 0);
                return 0;
            }
            break;

        default:
            break;
    }

    errno = EINVAL;
    return -1;
}

int dealer_t::xsend (msg: &mut ZmqMessage)
{
    return sendpipe (msg, null_mut());
}

int dealer_t::xrecv (msg: &mut ZmqMessage)
{
    return recvpipe (msg, null_mut());
}

bool dealer_t::xhas_in ()
{
    return _fq.has_in ();
}

bool dealer_t::xhas_out ()
{
    return _lb.has_out ();
}

void dealer_t::xread_activated (pipe: &mut ZmqPipe)
{
    _fq.activated (pipe);
}

void dealer_t::xwrite_activated (pipe: &mut ZmqPipe)
{
    _lb.activated (pipe);
}

void dealer_t::xpipe_terminated (pipe: &mut ZmqPipe)
{
    _fq.pipe_terminated (pipe);
    _lb.pipe_terminated (pipe);
}

int dealer_t::sendpipe (msg: &mut ZmqMessage ZmqPipe **pipe)
{
    return _lb.sendpipe (msg, pipe);
}

int dealer_t::recvpipe (msg: &mut ZmqMessage ZmqPipe **pipe)
{
    return _fq.recvpipe (msg, pipe);
}
