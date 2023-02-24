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
// #include "server.hpp"
// #include "pipe.hpp"
// #include "wire.hpp"
// #include "random.hpp"
// #include "likely.hpp"
// #include "err.hpp"


//  TODO: This class uses O(n) scheduling. Rewrite it to use O(1) algorithm.
pub struct server_t : public ZmqSocketBase
{
// public:
    server_t (ZmqContext *parent_, u32 tid_, sid_: i32);
    ~server_t ();

    //  Overrides of functions from ZmqSocketBase.
    void xattach_pipe (pipe_t *pipe_,
                       subscribe_to_all_: bool,
                       locally_initiated_: bool);
    int xsend (msg: &mut ZmqMessage);
    int xrecv (msg: &mut ZmqMessage);
    bool xhas_in ();
    bool xhas_out ();
    void xread_activated (pipe_: &mut pipe_t);
    void xwrite_activated (pipe_: &mut pipe_t);
    void xpipe_terminated (pipe_: &mut pipe_t);

  // private:
    //  Fair queueing object for inbound pipes.
    fq_t _fq;

    struct outpipe_t
    {
        pipe_t *pipe;
        active: bool
    };

    //  Outbound pipes indexed by the peer IDs.
    typedef std::map<u32, outpipe_t> out_pipes_t;
    out_pipes_t _out_pipes;

    //  Routing IDs are generated. It's a simple increment and wrap-over
    //  algorithm. This value is the next ID to use (if not used already).
    u32 _next_routing_id;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (server_t)
};

server_t::server_t (class ZmqContext *parent_, u32 tid_, sid_: i32) :
    ZmqSocketBase (parent_, tid_, sid_, true),
    _next_routing_id (generate_random ())
{
    options.type = ZMQ_SERVER;
    options.can_send_hello_msg = true;
    options.can_recv_disconnect_msg = true;
}

server_t::~server_t ()
{
    zmq_assert (_out_pipes.empty ());
}

void server_t::xattach_pipe (pipe_t *pipe_,
                                  subscribe_to_all_: bool,
                                  locally_initiated_: bool)
{
    LIBZMQ_UNUSED (subscribe_to_all_);
    LIBZMQ_UNUSED (locally_initiated_);

    zmq_assert (pipe_);

    u32 routing_id = _next_routing_id++;
    if (!routing_id)
        routing_id = _next_routing_id++; //  Never use Routing ID zero

    pipe_->set_server_socket_routing_id (routing_id);
    //  Add the record into output pipes lookup table
    outpipe_t outpipe = {pipe_, true};
    const bool ok =
      _out_pipes.ZMQ_MAP_INSERT_OR_EMPLACE (routing_id, outpipe).second;
    zmq_assert (ok);

    _fq.attach (pipe_);
}

void server_t::xpipe_terminated (pipe_: &mut pipe_t)
{
    const out_pipes_t::iterator it =
      _out_pipes.find (pipe_->get_server_socket_routing_id ());
    zmq_assert (it != _out_pipes.end ());
    _out_pipes.erase (it);
    _fq.pipe_terminated (pipe_);
}

void server_t::xread_activated (pipe_: &mut pipe_t)
{
    _fq.activated (pipe_);
}

void server_t::xwrite_activated (pipe_: &mut pipe_t)
{
    const out_pipes_t::iterator end = _out_pipes.end ();
    out_pipes_t::iterator it;
    for (it = _out_pipes.begin (); it != end; ++it)
        if (it->second.pipe == pipe_)
            break;

    zmq_assert (it != _out_pipes.end ());
    zmq_assert (!it->second.active);
    it->second.active = true;
}

int server_t::xsend (msg: &mut ZmqMessage)
{
    //  SERVER sockets do not allow multipart data (ZMQ_SNDMORE)
    if (msg.flags () & ZmqMessage::more) {
        errno = EINVAL;
        return -1;
    }
    //  Find the pipe associated with the routing stored in the message.
    const u32 routing_id = msg.get_routing_id ();
    out_pipes_t::iterator it = _out_pipes.find (routing_id);

    if (it != _out_pipes.end ()) {
        if (!it->second.pipe.check_write ()) {
            it->second.active = false;
            errno = EAGAIN;
            return -1;
        }
    } else {
        errno = EHOSTUNREACH;
        return -1;
    }

    //  Message might be delivered over inproc, so we reset routing id
    int rc = msg.reset_routing_id ();
    errno_assert (rc == 0);

    const bool ok = it->second.pipe.write (msg);
    if (unlikely (!ok)) {
        // Message failed to send - we must close it ourselves.
        rc = msg.close ();
        errno_assert (rc == 0);
    } else
        it->second.pipe.flush ();

    //  Detach the message from the data buffer.
    rc = msg.init ();
    errno_assert (rc == 0);

    return 0;
}

int server_t::xrecv (msg: &mut ZmqMessage)
{
    pipe_t *pipe = null_mut();
    int rc = _fq.recvpipe (msg, &pipe);

    // Drop any messages with more flag
    while (rc == 0 && msg.flags () & ZmqMessage::more) {
        // drop all frames of the current multi-frame message
        rc = _fq.recvpipe (msg, null_mut());

        while (rc == 0 && msg.flags () & ZmqMessage::more)
            rc = _fq.recvpipe (msg, null_mut());

        // get the new message
        if (rc == 0)
            rc = _fq.recvpipe (msg, &pipe);
    }

    if (rc != 0)
        return rc;

    zmq_assert (pipe != null_mut());

    const u32 routing_id = pipe.get_server_socket_routing_id ();
    msg.set_routing_id (routing_id);

    return 0;
}

bool server_t::xhas_in ()
{
    return _fq.has_in ();
}

bool server_t::xhas_out ()
{
    //  In theory, SERVER socket is always ready for writing. Whether actual
    //  attempt to write succeeds depends on which pipe the message is going
    //  to be routed to.
    return true;
}
