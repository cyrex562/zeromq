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
// #include "client.hpp"
// #include "err.hpp"
// #include "msg.hpp"
pub struct client_t ZMQ_FINAL : public ZmqSocketBase
{
// public:
    client_t (ZmqContext *parent_, u32 tid_, sid_: i32);
    ~client_t ();

  protected:
    //  Overrides of functions from ZmqSocketBase.
    void xattach_pipe (pipe_t *pipe_,
                       bool subscribe_to_all_,
                       bool locally_initiated_);
    int xsend (ZmqMessage *msg);
    int xrecv (ZmqMessage *msg);
    bool xhas_in ();
    bool xhas_out ();
    void xread_activated (pipe_t *pipe_);
    void xwrite_activated (pipe_t *pipe_);
    void xpipe_terminated (pipe_t *pipe_);

  // private:
    //  Messages are fair-queued from inbound pipes. And load-balanced to
    //  the outbound pipes.
    fq_t _fq;
    lb_t _lb;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (client_t)
};

client_t::client_t (class ZmqContext *parent_, u32 tid_, sid_: i32) :
    ZmqSocketBase (parent_, tid_, sid_, true)
{
    options.type = ZMQ_CLIENT;
    options.can_send_hello_msg = true;
    options.can_recv_hiccup_msg = true;
}

client_t::~client_t ()
{
}

void client_t::xattach_pipe (pipe_t *pipe_,
                                  bool subscribe_to_all_,
                                  bool locally_initiated_)
{
    LIBZMQ_UNUSED (subscribe_to_all_);
    LIBZMQ_UNUSED (locally_initiated_);

    zmq_assert (pipe_);

    _fq.attach (pipe_);
    _lb.attach (pipe_);
}

int client_t::xsend (ZmqMessage *msg)
{
    //  CLIENT sockets do not allow multipart data (ZMQ_SNDMORE)
    if (msg->flags () & ZmqMessage::more) {
        errno = EINVAL;
        return -1;
    }
    return _lb.sendpipe (msg, NULL);
}

int client_t::xrecv (ZmqMessage *msg)
{
    int rc = _fq.recvpipe (msg, NULL);

    // Drop any messages with more flag
    while (rc == 0 && msg->flags () & ZmqMessage::more) {
        // drop all frames of the current multi-frame message
        rc = _fq.recvpipe (msg, NULL);

        while (rc == 0 && msg->flags () & ZmqMessage::more)
            rc = _fq.recvpipe (msg, NULL);

        // get the new message
        if (rc == 0)
            rc = _fq.recvpipe (msg, NULL);
    }

    return rc;
}

bool client_t::xhas_in ()
{
    return _fq.has_in ();
}

bool client_t::xhas_out ()
{
    return _lb.has_out ();
}

void client_t::xread_activated (pipe_t *pipe_)
{
    _fq.activated (pipe_);
}

void client_t::xwrite_activated (pipe_t *pipe_)
{
    _lb.activated (pipe_);
}

void client_t::xpipe_terminated (pipe_t *pipe_)
{
    _fq.pipe_terminated (pipe_);
    _lb.pipe_terminated (pipe_);
}
