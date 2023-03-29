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
// #include "stream.hpp"
// #include "pipe.hpp"
// #include "wire.hpp"
// #include "random.hpp"
// #include "likely.hpp"
// #include "err.hpp"

stream_t::stream_t (class ZmqContext *parent_, tid: u32, sid_: i32) :
    routing_socket_base_t (parent_, tid, sid_),
    _prefetched (false),
    _routing_id_sent (false),
    _current_out (null_mut()),
    _more_out (false),
    _next_integral_routing_id (generate_random ())
{
    options.type = ZMQ_STREAM;
    options.raw_socket = true;

    _prefetched_routing_id.init ();
    _prefetched_msg.init ();
}

stream_t::~stream_t ()
{
    _prefetched_routing_id.close ();
    _prefetched_msg.close ();
}

void stream_t::xattach_pipe (pipe: &mut ZmqPipe,
                                  subscribe_to_all_: bool,
                                  locally_initiated_: bool)
{
    LIBZMQ_UNUSED (subscribe_to_all_);

    zmq_assert (pipe);

    identify_peer (pipe, locally_initiated_);
    _fq.attach (pipe);
}

void stream_t::xpipe_terminated (pipe: &mut ZmqPipe)
{
    erase_out_pipe (pipe);
    _fq.pipe_terminated (pipe);
    // TODO router_t calls pipe_->rollback() here; should this be done here as
    // well? then xpipe_terminated could be pulled up to routing_socket_base_t
    if (pipe == _current_out)
        _current_out = null_mut();
}

void stream_t::xread_activated (pipe: &mut ZmqPipe)
{
    _fq.activated (pipe);
}

int stream_t::xsend (msg: &mut ZmqMessage)
{
    //  If this is the first part of the message it's the ID of the
    //  peer to send the message to.
    if (!_more_out) {
        zmq_assert (!_current_out);

        //  If we have malformed message (prefix with no subsequent message)
        //  then just silently ignore it.
        //  TODO: The connections should be killed instead.
        if (msg.flags () & ZMQ_MSG_MORE) {
            //  Find the pipe associated with the routing id stored in the prefix.
            //  If there's no such pipe return an error

            out_pipe_t *out_pipe = lookup_out_pipe (
              Blob (static_cast<unsigned char *> (msg.data ()),
                      msg.size (), ReferenceTag ()));

            if (out_pipe) {
                _current_out = out_pipe.pipe;
                if (!_current_out.check_write ()) {
                    out_pipe.active = false;
                    _current_out = null_mut();
                    errno = EAGAIN;
                    return -1;
                }
            } else {
                errno = EHOSTUNREACH;
                return -1;
            }
        }

        //  Expect one more message frame.
        _more_out = true;

        int rc = msg.close ();
        errno_assert (rc == 0);
        rc = msg.init ();
        errno_assert (rc == 0);
        return 0;
    }

    //  Ignore the MORE flag
    msg.reset_flags (ZMQ_MSG_MORE);

    //  This is the last part of the message.
    _more_out = false;

    //  Push the message into the pipe. If there's no out pipe, just drop it.
    if (_current_out) {
        // Close the remote connection if user has asked to do so
        // by sending zero length message.
        // Pending messages in the pipe will be dropped (on receiving term- ack)
        if (msg.size () == 0) {
            _current_out.terminate (false);
            int rc = msg.close ();
            errno_assert (rc == 0);
            rc = msg.init ();
            errno_assert (rc == 0);
            _current_out = null_mut();
            return 0;
        }
        const bool ok = _current_out.write (msg);
        if (likely (ok))
            _current_out.flush ();
        _current_out = null_mut();
    } else {
        let rc: i32 = msg.close ();
        errno_assert (rc == 0);
    }

    //  Detach the message from the data buffer.
    let rc: i32 = msg.init ();
    errno_assert (rc == 0);

    return 0;
}

use std::_ =>:default;
use bincode::options;
use crate::options::set_opt_bool;
use crate::socket_base::routing_socket_base_t;
use crate::zmq_hdr::ZMQ_STREAM_NOTIFY;

pub fn xsetsockopt (opt_kind: i32,
                    optval_: &[u8]) -> anyhow::Result<()>
{
    match (opt_kind) {
        case ZMQ_STREAM_NOTIFY =>{
        // return do_setsockopt_int_as_bool_strict (optval_, optvallen_,
        //                                          &options.raw_notify);
        return set_opt_bool(optval_, & mut options.raw_notify);}

        // _ =>
        _ => {
            return routing_socket_base_t::xsetsockopt(self, opt_kind, optval_);
        }
    }
}

int stream_t::xrecv (msg: &mut ZmqMessage)
{
    if (_prefetched) {
        if (!_routing_id_sent) {
            let rc: i32 = msg.move (_prefetched_routing_id);
            errno_assert (rc == 0);
            _routing_id_sent = true;
        } else {
            let rc: i32 = msg.move (_prefetched_msg);
            errno_assert (rc == 0);
            _prefetched = false;
        }
        return 0;
    }

    ZmqPipe *pipe = null_mut();
    int rc = _fq.recvpipe (&_prefetched_msg, &pipe);
    if (rc != 0)
        return -1;

    zmq_assert (pipe != null_mut());
    zmq_assert ((_prefetched_msg.flags () & ZMQ_MSG_MORE) == 0);

    //  We have received a frame with TCP data.
    //  Rather than sending this frame, we keep it in prefetched
    //  buffer and send a frame with peer's ID.
    const Blob &routing_id = pipe.get_routing_id ();
    rc = msg.close ();
    errno_assert (rc == 0);
    rc = msg.init_size (routing_id.size ());
    errno_assert (rc == 0);

    // forward metadata (if any)
    ZmqMetadata *metadata = _prefetched_msg.metadata ();
    if (metadata)
        msg.set_metadata (metadata);

    memcpy (msg.data (), routing_id.data (), routing_id.size ());
    msg.set_flags (ZMQ_MSG_MORE);

    _prefetched = true;
    _routing_id_sent = true;

    return 0;
}

bool stream_t::xhas_in ()
{
    //  We may already have a message pre-fetched.
    if (_prefetched)
        return true;

    //  Try to read the next message.
    //  The message, if read, is kept in the pre-fetch buffer.
    ZmqPipe *pipe = null_mut();
    int rc = _fq.recvpipe (&_prefetched_msg, &pipe);
    if (rc != 0)
        return false;

    zmq_assert (pipe != null_mut());
    zmq_assert ((_prefetched_msg.flags () & ZMQ_MSG_MORE) == 0);

    const Blob &routing_id = pipe.get_routing_id ();
    rc = _prefetched_routing_id.init_size (routing_id.size ());
    errno_assert (rc == 0);

    // forward metadata (if any)
    ZmqMetadata *metadata = _prefetched_msg.metadata ();
    if (metadata)
        _prefetched_routing_id.set_metadata (metadata);

    memcpy (_prefetched_routing_id.data (), routing_id.data (),
            routing_id.size ());
    _prefetched_routing_id.set_flags (ZMQ_MSG_MORE);

    _prefetched = true;
    _routing_id_sent = false;

    return true;
}

bool stream_t::xhas_out ()
{
    //  In theory, STREAM socket is always ready for writing. Whether actual
    //  attempt to write succeeds depends on which pipe the message is going
    //  to be routed to.
    return true;
}

void stream_t::identify_peer (pipe: &mut ZmqPipe, locally_initiated_: bool)
{
    //  Always assign routing id for raw-socket
    unsigned char buffer[5];
    buffer[0] = 0;
    Blob routing_id;
    if (locally_initiated_ && connect_routing_id_is_set ()) {
        const std::string connect_routing_id = extract_connect_routing_id ();
        routing_id.set (
          reinterpret_cast<const unsigned char *> (connect_routing_id.c_str ()),
          connect_routing_id.length ());
        //  Not allowed to duplicate an existing rid
        zmq_assert (!has_out_pipe (routing_id));
    } else {
        put_u32 (buffer + 1, _next_integral_routing_id++);
        routing_id.set (buffer, sizeof buffer);
        memcpy (options.routing_id, routing_id.data (), routing_id.size ());
        options.routing_id_size =
          static_cast<unsigned char> (routing_id.size ());
    }
    pipe.set_router_socket_routing_id (routing_id);
    add_out_pipe (ZMQ_MOVE (routing_id), pipe);
}
pub struct stream_t ZMQ_FINAL : public routing_socket_base_t
{
// public:
    stream_t (ZmqContext *parent_, tid: u32, sid_: i32);
    ~stream_t ();

    //  Overrides of functions from ZmqSocketBase.
    void xattach_pipe (pipe: &mut ZmqPipe,
                       subscribe_to_all_: bool,
                       locally_initiated_: bool);
    int xsend (msg: &mut ZmqMessage);
    int xrecv (msg: &mut ZmqMessage);
    bool xhas_in ();
    bool xhas_out ();
    void xread_activated (pipe: &mut ZmqPipe);
    void xpipe_terminated (pipe: &mut ZmqPipe);
    int xsetsockopt (option_: i32, const optval_: &mut [u8], optvallen_: usize);

  // private:
    //  Generate peer's id and update lookup map
    void identify_peer (pipe: &mut ZmqPipe, locally_initiated_: bool);

    //  Fair queueing object for inbound pipes.
    fq_t _fq;

    //  True iff there is a message held in the pre-fetch buffer.
    _prefetched: bool

    //  If true, the receiver got the message part with
    //  the peer's identity.
    _routing_id_sent: bool

    //  Holds the prefetched identity.
    ZmqMessage _prefetched_routing_id;

    //  Holds the prefetched message.
    ZmqMessage _prefetched_msg;

    //  The pipe we are currently writing to.
    ZmqPipe *_current_out;

    //  If true, more outgoing message parts are expected.
    _more_out: bool

    //  Routing IDs are generated. It's a simple increment and wrap-over
    //  algorithm. This value is the next ID to use (if not used already).
    u32 _next_integral_routing_id;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (stream_t)
};
