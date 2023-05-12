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
// #include "macros.hpp"
// #include "router.hpp"
// #include "pipe.hpp"
// #include "wire.hpp"
// #include "random.hpp"
// #include "likely.hpp"
// #include "err.hpp"


//  TODO: This class uses O(n) scheduling. Rewrite it to use O(1) algorithm.
pub struct router_t : public routing_socket_base_t
{
//
    router_t (ZmqContext *parent_, tid: u32, sid_: i32);
    ~router_t () ;

    //  Overrides of functions from ZmqSocketBase.
    void xattach_pipe (pipe: &mut ZmqPipe,
                       subscribe_to_all_: bool,
                       locally_initiated_: bool) ;
    int
    xsetsockopt (option_: i32, const optval_: &mut [u8], optvallen_: usize) ;
    int xsend (msg: &mut ZmqMessage) ;
    int xrecv (msg: &mut ZmqMessage) ;
    bool xhas_in () ;
    bool xhas_out () ;
    void xread_activated (pipe: &mut ZmqPipe) ;
    void xpipe_terminated (pipe: &mut ZmqPipe) ;
    int get_peer_state (const routing_id_: &mut [u8],
                        routing_id_size_: usize) const ;


    //  Rollback any message parts that were sent but not yet flushed.
    int rollback ();

  //
    //  Receive peer id and update lookup map
    bool identify_peer (pipe: &mut ZmqPipe, locally_initiated_: bool);

    //  Fair queueing object for inbound pipes.
    ZmqFq fair_queue;

    //  True iff there is a message held in the pre-fetch buffer.
    _prefetched: bool

    //  If true, the receiver got the message part with
    //  the peer's identity.
    _routing_id_sent: bool

    //  Holds the prefetched identity.
    ZmqMessage _prefetched_id;

    //  Holds the prefetched message.
    ZmqMessage _prefetched_msg;

    //  The pipe we are currently reading from
    ZmqPipe *_current_in;

    //  Should current_in should be terminate after all parts received?
    _terminate_current_in: bool

    //  If true, more incoming message parts are expected.
    _more_in: bool

    //  We keep a set of pipes that have not been identified yet.
    std::set<ZmqPipe *> _anonymous_pipes;

    //  The pipe we are currently writing to.
    ZmqPipe *_current_out;

    //  If true, more outgoing message parts are expected.
    _more_out: bool

    //  Routing IDs are generated. It's a simple increment and wrap-over
    //  algorithm. This value is the next ID to use (if not used already).
    u32 _next_integral_routing_id;

    // If true, report EAGAIN to the caller instead of silently dropping
    // the message targeting an unknown peer.
    _mandatory: bool
    _raw_socket: bool

    // if true, send an empty message to every connected router peer
    probe_router: bool

    // If true, the router will reassign an identity upon encountering a
    // name collision. The new pipe will take the identity, the old pipe
    // will be terminated.
    _handover: bool

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (router_t)
};

router_t::router_t (parent: &mut ZmqContext, tid: u32, sid_: i32) :
    routing_socket_base_t (parent_, tid, sid_),
    _prefetched (false),
    _routing_id_sent (false),
    _current_in (null_mut()),
    _terminate_current_in (false),
    _more_in (false),
    _current_out (null_mut()),
    _more_out (false),
    _next_integral_routing_id (generate_random ()),
    _mandatory (false),
    //  raw_socket functionality in ROUTER is deprecated
    _raw_socket (false),
    probe_router (false),
    _handover (false)
{
    options.type_ = ZMQ_ROUTER;
    options.recv_routing_id = true;
    options.raw_socket = false;
    options.can_send_hello_msg = true;
    options.can_recv_disconnect_msg = true;

    _prefetched_id.init ();
    _prefetched_msg.init ();
}

router_t::~router_t ()
{
    // zmq_assert (_anonymous_pipes.empty ());
    _prefetched_id.close ();
    _prefetched_msg.close ();
}

void router_t::xattach_pipe (pipe: &mut ZmqPipe,
                                  subscribe_to_all_: bool,
                                  locally_initiated_: bool)
{
    LIBZMQ_UNUSED (subscribe_to_all_);

    // zmq_assert (pipe);

    if (probe_router) {
        ZmqMessage probe_msg;
        int rc = probe_msg.init ();
        // errno_assert (rc == 0);

        rc = pipe.write (&probe_msg);
        // zmq_assert (rc) is not applicable here, since it is not a bug.
        LIBZMQ_UNUSED (rc);

        pipe.flush ();

        rc = probe_msg.close ();
        // errno_assert (rc == 0);
    }

    const bool routing_id_ok = identify_peer (pipe, locally_initiated_);
    if (routing_id_ok)
        fair_queue.attach (pipe);
    else
        _anonymous_pipes.insert (pipe);
}

int router_t::xsetsockopt (option_: i32,
                                const optval_: &mut [u8],
                                optvallen_: usize)
{
    const bool is_int = (optvallen_ == mem::size_of::<int>());
    int value = 0;
    if (is_int)
        memcpy (&value, optval_, mem::size_of::<int>());

    switch (option_) {
        case ZMQ_ROUTER_RAW:
            if (is_int && value >= 0) {
                _raw_socket = (value != 0);
                if (_raw_socket) {
                    options.recv_routing_id = false;
                    options.raw_socket = true;
                }
                return 0;
            }
            break;

        case ZMQ_ROUTER_MANDATORY:
            if (is_int && value >= 0) {
                _mandatory = (value != 0);
                return 0;
            }
            break;

        case ZMQ_PROBE_ROUTER:
            if (is_int && value >= 0) {
                probe_router = (value != 0);
                return 0;
            }
            break;

        case ZMQ_ROUTER_HANDOVER:
            if (is_int && value >= 0) {
                _handover = (value != 0);
                return 0;
            }
            break;

// #ifdef ZMQ_BUILD_DRAFT_API
        case ZMQ_ROUTER_NOTIFY:
            if (is_int && value >= 0
                && value <= (ZMQ_NOTIFY_CONNECT | ZMQ_NOTIFY_DISCONNECT)) {
                options.router_notify = value;
                return 0;
            }
            break;
// #endif

        _ =>
            return routing_socket_base_t::xsetsockopt (option_, optval_,
                                                       optvallen_);
    }
    errno = EINVAL;
    return -1;
}


void router_t::xpipe_terminated (pipe: &mut ZmqPipe)
{
    if (0 == _anonymous_pipes.erase (pipe)) {
        erase_out_pipe (pipe);
        fair_queue.pipe_terminated (pipe);
        pipe.rollback ();
        if (pipe == _current_out)
            _current_out = null_mut();
    }
}

void router_t::xread_activated (pipe: &mut ZmqPipe)
{
    const std::set<ZmqPipe *>::iterator it = _anonymous_pipes.find (pipe);
    if (it == _anonymous_pipes.end ())
        fair_queue.activated (pipe);
    else {
        const bool routing_id_ok = identify_peer (pipe, false);
        if (routing_id_ok) {
            _anonymous_pipes.erase (it);
            fair_queue.attach (pipe);
        }
    }
}

int router_t::xsend (msg: &mut ZmqMessage)
{
    //  If this is the first part of the message it's the ID of the
    //  peer to send the message to.
    if (!_more_out) {
        // zmq_assert (!_current_out);

        //  If we have malformed message (prefix with no subsequent message)
        //  then just silently ignore it.
        //  TODO: The connections should be killed instead.
        if (msg.flags () & ZMQ_MSG_MORE) {
            _more_out = true;

            //  Find the pipe associated with the routing id stored in the prefix.
            //  If there's no such pipe just silently ignore the message, unless
            //  router_mandatory is set.
            ZmqOutPipe *out_pipe = lookup_out_pipe (
              Blob ( (msg.data ()),
                      msg.size (), ReferenceTag ()));

            if (out_pipe) {
                _current_out = out_pipe.pipe;

                // Check whether pipe is closed or not
                if (!_current_out.check_write ()) {
                    // Check whether pipe is full or not
                    const bool pipe_full = !_current_out.check_hwm ();
                    out_pipe.active = false;
                    _current_out = null_mut();

                    if (_mandatory) {
                        _more_out = false;
                        if (pipe_full)
                            errno = EAGAIN;
                        else
                            errno = EHOSTUNREACH;
                        return -1;
                    }
                }
            } else if (_mandatory) {
                _more_out = false;
                errno = EHOSTUNREACH;
                return -1;
            }
        }

        int rc = msg.close ();
        // errno_assert (rc == 0);
        rc = msg.init ();
        // errno_assert (rc == 0);
        return 0;
    }

    //  Ignore the MORE flag for raw-sock or assert?
    if (options.raw_socket)
        msg.reset_flags (ZMQ_MSG_MORE);

    //  Check whether this is the last part of the message.
    _more_out = (msg.flags () & ZMQ_MSG_MORE) != 0;

    //  Push the message into the pipe. If there's no out pipe, just drop it.
    if (_current_out) {
        // Close the remote connection if user has asked to do so
        // by sending zero length message.
        // Pending messages in the pipe will be dropped (on receiving term- ack)
        if (_raw_socket && msg.size () == 0) {
            _current_out.terminate (false);
            int rc = msg.close ();
            // errno_assert (rc == 0);
            rc = msg.init ();
            // errno_assert (rc == 0);
            _current_out = null_mut();
            return 0;
        }

        const bool ok = _current_out.write (msg);
        if ( (!ok)) {
            // Message failed to send - we must close it ourselves.
            let rc: i32 = msg.close ();
            // errno_assert (rc == 0);
            // HWM was checked before, so the pipe must be gone. Roll back
            // messages that were piped, for example REP labels.
            _current_out.rollback ();
            _current_out = null_mut();
        } else {
            if (!_more_out) {
                _current_out.flush ();
                _current_out = null_mut();
            }
        }
    } else {
        let rc: i32 = msg.close ();
        // errno_assert (rc == 0);
    }

    //  Detach the message from the data buffer.
    let rc: i32 = msg.init ();
    // errno_assert (rc == 0);

    return 0;
}

int router_t::xrecv (msg: &mut ZmqMessage)
{
    if (_prefetched) {
        if (!_routing_id_sent) {
            let rc: i32 = msg.move (_prefetched_id);
            // errno_assert (rc == 0);
            _routing_id_sent = true;
        } else {
            let rc: i32 = msg.move (_prefetched_msg);
            // errno_assert (rc == 0);
            _prefetched = false;
        }
        _more_in = (msg.flags () & ZMQ_MSG_MORE) != 0;

        if (!_more_in) {
            if (_terminate_current_in) {
                _current_in.terminate (true);
                _terminate_current_in = false;
            }
            _current_in = null_mut();
        }
        return 0;
    }

    ZmqPipe *pipe = null_mut();
    int rc = fair_queue.recvpipe (msg, &pipe);

    //  It's possible that we receive peer's routing id. That happens
    //  after reconnection. The current implementation assumes that
    //  the peer always uses the same routing id.
    while (rc == 0 && msg.is_routing_id ())
        rc = fair_queue.recvpipe (msg, &pipe);

    if (rc != 0)
        return -1;

    // zmq_assert (pipe != null_mut());

    //  If we are in the middle of reading a message, just return the next part.
    if (_more_in) {
        _more_in = (msg.flags () & ZMQ_MSG_MORE) != 0;

        if (!_more_in) {
            if (_terminate_current_in) {
                _current_in.terminate (true);
                _terminate_current_in = false;
            }
            _current_in = null_mut();
        }
    } else {
        //  We are at the beginning of a message.
        //  Keep the message part we have in the prefetch buffer
        //  and return the ID of the peer instead.
        rc = _prefetched_msg.move (*msg);
        // errno_assert (rc == 0);
        _prefetched = true;
        _current_in = pipe;

        const Blob &routing_id = pipe.get_routing_id ();
        rc = msg.init_size (routing_id.size ());
        // errno_assert (rc == 0);
        memcpy (msg.data (), routing_id.data (), routing_id.size ());
        msg.set_flags (ZMQ_MSG_MORE);
        if (_prefetched_msg.metadata ())
            msg.set_metadata (_prefetched_msg.metadata ());
        _routing_id_sent = true;
    }

    return 0;
}

int router_t::rollback ()
{
    if (_current_out) {
        _current_out.rollback ();
        _current_out = null_mut();
        _more_out = false;
    }
    return 0;
}

bool router_t::xhas_in ()
{
    //  If we are in the middle of reading the messages, there are
    //  definitely more parts available.
    if (_more_in)
        return true;

    //  We may already have a message pre-fetched.
    if (_prefetched)
        return true;

    //  Try to read the next message.
    //  The message, if read, is kept in the pre-fetch buffer.
    ZmqPipe *pipe = null_mut();
    int rc = fair_queue.recvpipe (&_prefetched_msg, &pipe);

    //  It's possible that we receive peer's routing id. That happens
    //  after reconnection. The current implementation assumes that
    //  the peer always uses the same routing id.
    //  TODO: handle the situation when the peer changes its routing id.
    while (rc == 0 && _prefetched_msg.is_routing_id ())
        rc = fair_queue.recvpipe (&_prefetched_msg, &pipe);

    if (rc != 0)
        return false;

    // zmq_assert (pipe != null_mut());

    const Blob &routing_id = pipe.get_routing_id ();
    rc = _prefetched_id.init_size (routing_id.size ());
    // errno_assert (rc == 0);
    memcpy (_prefetched_id.data (), routing_id.data (), routing_id.size ());
    _prefetched_id.set_flags (ZMQ_MSG_MORE);
    if (_prefetched_msg.metadata ())
        _prefetched_id.set_metadata (_prefetched_msg.metadata ());

    _prefetched = true;
    _routing_id_sent = false;
    _current_in = pipe;

    return true;
}

static bool check_pipe_hwm (const ZmqPipe &pipe)
{
    return pipe.check_hwm ();
}

bool router_t::xhas_out ()
{
    //  In theory, ROUTER socket is always ready for writing (except when
    //  MANDATORY is set). Whether actual attempt to write succeeds depends
    //  on which pipe the message is going to be routed to.

    if (!_mandatory)
        return true;

    return any_of_out_pipes (check_pipe_hwm);
}

int router_t::get_peer_state (const routing_id_: &mut [u8],
                                   routing_id_size_: usize) const
{
    int res = 0;

    // TODO remove the const_cast, see comment in lookup_out_pipe
    const Blob routing_id_blob (
       (const_cast<void *> (routing_id_)),
      routing_id_size_, ReferenceTag ());
    const ZmqOutPipe *out_pipe = lookup_out_pipe (routing_id_blob);
    if (!out_pipe) {
        errno = EHOSTUNREACH;
        return -1;
    }

    if (out_pipe.pipe.check_hwm ())
        res |= ZMQ_POLLOUT;

    /** \todo does it make any sense to check the inpipe as well? */

    return res;
}

bool router_t::identify_peer (pipe: &mut ZmqPipe, locally_initiated_: bool)
{
let mut msg = ZmqMessage::default();
    Blob routing_id;

    if (locally_initiated_ && connect_routing_id_is_set ()) {
        const std::string connect_routing_id = extract_connect_routing_id ();
        routing_id.set (
          reinterpret_cast<const unsigned char *> (connect_routing_id.c_str ()),
          connect_routing_id.length ());
        //  Not allowed to duplicate an existing rid
        // zmq_assert (!has_out_pipe (routing_id));
    } else if (
      options
        .raw_socket) { //  Always assign an integral routing id for raw-socket
        unsigned char buf[5];
        buf[0] = 0;
        put_u32 (buf + 1, _next_integral_routing_id+= 1);
        routing_id.set (buf, sizeof buf);
    } else if (!options.raw_socket) {
        //  Pick up handshake cases and also case where next integral routing id is set
        msg.init ();
        const bool ok = pipe.read (&msg);
        if (!ok)
            return false;

        if (msg.size () == 0) {
            //  Fall back on the auto-generation
            unsigned char buf[5];
            buf[0] = 0;
            put_u32 (buf + 1, _next_integral_routing_id+= 1);
            routing_id.set (buf, sizeof buf);
            msg.close ();
        } else {
            routing_id.set ( (msg.data ()),
                            msg.size ());
            msg.close ();

            //  Try to remove an existing routing id entry to allow the new
            //  connection to take the routing id.
            const ZmqOutPipe *const existing_outpipe =
              lookup_out_pipe (routing_id);

            if (existing_outpipe) {
                if (!_handover)
                    //  Ignore peers with duplicate ID
                    return false;

                //  We will allow the new connection to take over this
                //  routing id. Temporarily assign a new routing id to the
                //  existing pipe so we can terminate it asynchronously.
                unsigned char buf[5];
                buf[0] = 0;
                put_u32 (buf + 1, _next_integral_routing_id+= 1);
                Blob new_routing_id (buf, sizeof buf);

                ZmqPipe *const old_pipe = existing_outpipe.pipe;

                erase_out_pipe (old_pipe);
                old_pipe.set_router_socket_routing_id (new_routing_id);
                add_out_pipe (ZMQ_MOVE (new_routing_id), old_pipe);

                if (old_pipe == _current_in)
                    _terminate_current_in = true;
                else
                    old_pipe.terminate (true);
            }
        }
    }

    pipe.set_router_socket_routing_id (routing_id);
    add_out_pipe (ZMQ_MOVE (routing_id), pipe);

    return true;
}
