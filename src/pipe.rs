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
// #include <new>
// #include <stddef.h>

// #include "macros.hpp"
// #include "pipe.hpp"
// #include "err.hpp"

// #include "ypipe.hpp"
// #include "ypipe_conflate.hpp"


//  Create a pipepair for bi-directional transfer of messages.
//  First HWM is for messages passed from first pipe to the second pipe.
//  Second HWM is for messages passed from second pipe to the first pipe.
//  Delay specifies how the pipe behaves when the peer terminates. If true
//  pipe receives all the pending messages before terminating, otherwise it
//  terminates straight away.
//  If conflate is true, only the most recently arrived message could be
//  read (older messages are discarded)
int pipepair (zmq::object_t *parents_[2],
              zmq::pipe_t *pipes_[2],
              const int hwms_[2],
              const bool conflate_[2]);

struct i_pipe_events
{
    virtual ~i_pipe_events () ZMQ_DEFAULT;

    virtual void read_activated (zmq::pipe_t *pipe_) = 0;
    virtual void write_activated (zmq::pipe_t *pipe_) = 0;
    virtual void hiccuped (zmq::pipe_t *pipe_) = 0;
    virtual void pipe_terminated (zmq::pipe_t *pipe_) = 0;
};

//  Note that pipe can be stored in three different arrays.
//  The array of inbound pipes (1), the array of outbound pipes (2) and
//  the generic array of pipes to be deallocated (3).

class pipe_t ZMQ_FINAL : public object_t,
                         public array_item_t<1>,
                         public array_item_t<2>,
                         public array_item_t<3>
{
    //  This allows pipepair to create pipe objects.
    friend int pipepair (zmq::object_t *parents_[2],
                         zmq::pipe_t *pipes_[2],
                         const int hwms_[2],
                         const bool conflate_[2]);

// public:
    //  Specifies the object to send events to.
    void set_event_sink (i_pipe_events *sink_);

    //  Pipe endpoint can store an routing ID to be used by its clients.
    void set_server_socket_routing_id (uint32_t server_socket_routing_id_);
    uint32_t get_server_socket_routing_id () const;

    //  Pipe endpoint can store an opaque ID to be used by its clients.
    void set_router_socket_routing_id (const blob_t &router_socket_routing_id_);
    const blob_t &get_routing_id () const;

    //  Returns true if there is at least one message to read in the pipe.
    bool check_read ();

    //  Reads a message to the underlying pipe.
    bool read (msg_t *msg_);

    //  Checks whether messages can be written to the pipe. If the pipe is
    //  closed or if writing the message would cause high watermark the
    //  function returns false.
    bool check_write ();

    //  Writes a message to the underlying pipe. Returns false if the
    //  message does not pass check_write. If false, the message object
    //  retains ownership of its message buffer.
    bool write (const msg_t *msg_);

    //  Remove unfinished parts of the outbound message from the pipe.
    void rollback () const;

    //  Flush the messages downstream.
    void flush ();

    //  Temporarily disconnects the inbound message stream and drops
    //  all the messages on the fly. Causes 'hiccuped' event to be generated
    //  in the peer.
    void hiccup ();

    //  Ensure the pipe won't block on receiving pipe_term.
    void set_nodelay ();

    //  Ask pipe to terminate. The termination will happen asynchronously
    //  and user will be notified about actual deallocation by 'terminated'
    //  event. If delay is true, the pending messages will be processed
    //  before actual shutdown.
    void terminate (bool delay_);

    //  Set the high water marks.
    void set_hwms (inhwm_: i32, outhwm_: i32);

    //  Set the boost to high water marks, used by inproc sockets so total hwm are sum of connect and bind sockets watermarks
    void set_hwms_boost (inhwmboost_: i32, outhwmboost_: i32);

    // send command to peer for notify the change of hwm
    void send_hwms_to_peer (inhwm_: i32, outhwm_: i32);

    //  Returns true if HWM is not reached
    bool check_hwm () const;

    void set_endpoint_pair (endpoint_uri_pair_t endpoint_pair_);
    const endpoint_uri_pair_t &get_endpoint_pair () const;

    void send_stats_to_peer (own_t *socket_base_);

    void send_disconnect_msg ();
    void set_disconnect_msg (const std::vector<unsigned char> &disconnect_);

    void send_hiccup_msg (const std::vector<unsigned char> &hiccup_);

  // private:
    //  Type of the underlying lock-free pipe.
    typedef ypipe_base_t<msg_t> upipe_t;

    //  Command handlers.
    void process_activate_read () ZMQ_OVERRIDE;
    void process_activate_write (uint64_t msgs_read_) ZMQ_OVERRIDE;
    void process_hiccup (pipe_: *mut c_void) ZMQ_OVERRIDE;
    void
    process_pipe_peer_stats (queue_count_: u64,
                             own_t *socket_base_,
                             endpoint_uri_pair_t *endpoint_pair_) ZMQ_OVERRIDE;
    void process_pipe_term () ZMQ_OVERRIDE;
    void process_pipe_term_ack () ZMQ_OVERRIDE;
    void process_pipe_hwm (inhwm_: i32, outhwm_: i32) ZMQ_OVERRIDE;

    //  Handler for delimiter read from the pipe.
    void process_delimiter ();

    //  Constructor is private. Pipe can only be created using
    //  pipepair function.
    pipe_t (object_t *parent_,
            upipe_t *inpipe_,
            upipe_t *outpipe_,
            inhwm_: i32,
            outhwm_: i32,
            bool conflate_);

    //  Pipepair uses this function to let us know about
    //  the peer pipe object.
    void set_peer (pipe_t *peer_);

    //  Destructor is private. Pipe objects destroy themselves.
    ~pipe_t () ZMQ_OVERRIDE;

    //  Underlying pipes for both directions.
    upipe_t *_in_pipe;
    upipe_t *_out_pipe;

    //  Can the pipe be read from / written to?
    bool _in_active;
    bool _out_active;

    //  High watermark for the outbound pipe.
    _hwm: i32;

    //  Low watermark for the inbound pipe.
    _lwm: i32;

    // boosts for high and low watermarks, used with inproc sockets so hwm are sum of send and recv hmws on each side of pipe
    _in_hwm_boost: i32;
    _out_hwm_boost: i32;

    //  Number of messages read and written so far.
    uint64_t _msgs_read;
    uint64_t _msgs_written;

    //  Last received peer's msgs_read. The actual number in the peer
    //  can be higher at the moment.
    uint64_t _peers_msgs_read;

    //  The pipe object on the other side of the pipepair.
    pipe_t *_peer;

    //  Sink to send events to.
    i_pipe_events *_sink;

    //  States of the pipe endpoint:
    //  active: common state before any termination begins,
    //  delimiter_received: delimiter was read from pipe before
    //      term command was received,
    //  waiting_for_delimiter: term command was already received
    //      from the peer but there are still pending messages to read,
    //  term_ack_sent: all pending messages were already read and
    //      all we are waiting for is ack from the peer,
    //  term_req_sent1: 'terminate' was explicitly called by the user,
    //  term_req_sent2: user called 'terminate' and then we've got
    //      term command from the peer as well.
    enum
    {
        active,
        delimiter_received,
        waiting_for_delimiter,
        term_ack_sent,
        term_req_sent1,
        term_req_sent2
    } _state;

    //  If true, we receive all the pending inbound messages before
    //  terminating. If false, we terminate immediately when the peer
    //  asks us to.
    bool _delay;

    //  Routing id of the writer. Used uniquely by the reader side.
    blob_t _router_socket_routing_id;

    //  Routing id of the writer. Used uniquely by the reader side.
    _server_socket_routing_id: i32;

    //  Returns true if the message is delimiter; false otherwise.
    static bool is_delimiter (const msg_t &msg_);

    //  Computes appropriate low watermark from the given high watermark.
    static int compute_lwm (hwm_: i32);

    const bool _conflate;

    // The endpoints of this pipe.
    endpoint_uri_pair_t _endpoint_pair;

    // Disconnect msg
    msg_t _disconnect_msg;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (pipe_t)
};

void send_routing_id (pipe_t *pipe_, const options_t &options_);

void send_hello_msg (pipe_t *pipe_, const options_t &options_);

int zmq::pipepair (object_t *parents_[2],
                   pipe_t *pipes_[2],
                   const int hwms_[2],
                   const bool conflate_[2])
{
    //   Creates two pipe objects. These objects are connected by two ypipes,
    //   each to pass messages in one direction.

    typedef ypipe_t<msg_t, message_pipe_granularity> upipe_normal_t;
    typedef ypipe_conflate_t<msg_t> upipe_conflate_t;

    pipe_t::upipe_t *upipe1;
    if (conflate_[0])
        upipe1 = new (std::nothrow) upipe_conflate_t ();
    else
        upipe1 = new (std::nothrow) upipe_normal_t ();
    alloc_assert (upipe1);

    pipe_t::upipe_t *upipe2;
    if (conflate_[1])
        upipe2 = new (std::nothrow) upipe_conflate_t ();
    else
        upipe2 = new (std::nothrow) upipe_normal_t ();
    alloc_assert (upipe2);

    pipes_[0] = new (std::nothrow)
      pipe_t (parents_[0], upipe1, upipe2, hwms_[1], hwms_[0], conflate_[0]);
    alloc_assert (pipes_[0]);
    pipes_[1] = new (std::nothrow)
      pipe_t (parents_[1], upipe2, upipe1, hwms_[0], hwms_[1], conflate_[1]);
    alloc_assert (pipes_[1]);

    pipes_[0]->set_peer (pipes_[1]);
    pipes_[1]->set_peer (pipes_[0]);

    return 0;
}

void zmq::send_routing_id (pipe_t *pipe_, const options_t &options_)
{
    zmq::msg_t id;
    const int rc = id.init_size (options_.routing_id_size);
    errno_assert (rc == 0);
    memcpy (id.data (), options_.routing_id, options_.routing_id_size);
    id.set_flags (zmq::msg_t::routing_id);
    const bool written = pipe_->write (&id);
    zmq_assert (written);
    pipe_->flush ();
}

void zmq::send_hello_msg (pipe_t *pipe_, const options_t &options_)
{
    zmq::msg_t hello;
    const int rc =
      hello.init_buffer (&options_.hello_msg[0], options_.hello_msg.size ());
    errno_assert (rc == 0);
    const bool written = pipe_->write (&hello);
    zmq_assert (written);
    pipe_->flush ();
}

zmq::pipe_t::pipe_t (object_t *parent_,
                     upipe_t *inpipe_,
                     upipe_t *outpipe_,
                     inhwm_: i32,
                     outhwm_: i32,
                     bool conflate_) :
    object_t (parent_),
    _in_pipe (inpipe_),
    _out_pipe (outpipe_),
    _in_active (true),
    _out_active (true),
    _hwm (outhwm_),
    _lwm (compute_lwm (inhwm_)),
    _in_hwm_boost (-1),
    _out_hwm_boost (-1),
    _msgs_read (0),
    _msgs_written (0),
    _peers_msgs_read (0),
    _peer (NULL),
    _sink (NULL),
    _state (active),
    _delay (true),
    _server_socket_routing_id (0),
    _conflate (conflate_)
{
    _disconnect_msg.init ();
}

zmq::pipe_t::~pipe_t ()
{
    _disconnect_msg.close ();
}

void zmq::pipe_t::set_peer (pipe_t *peer_)
{
    //  Peer can be set once only.
    zmq_assert (!_peer);
    _peer = peer_;
}

void zmq::pipe_t::set_event_sink (i_pipe_events *sink_)
{
    // Sink can be set once only.
    zmq_assert (!_sink);
    _sink = sink_;
}

void zmq::pipe_t::set_server_socket_routing_id (
  uint32_t server_socket_routing_id_)
{
    _server_socket_routing_id = server_socket_routing_id_;
}

uint32_t zmq::pipe_t::get_server_socket_routing_id () const
{
    return _server_socket_routing_id;
}

void zmq::pipe_t::set_router_socket_routing_id (
  const blob_t &router_socket_routing_id_)
{
    _router_socket_routing_id.set_deep_copy (router_socket_routing_id_);
}

const zmq::blob_t &zmq::pipe_t::get_routing_id () const
{
    return _router_socket_routing_id;
}

bool zmq::pipe_t::check_read ()
{
    if (unlikely (!_in_active))
        return false;
    if (unlikely (_state != active && _state != waiting_for_delimiter))
        return false;

    //  Check if there's an item in the pipe.
    if (!_in_pipe->check_read ()) {
        _in_active = false;
        return false;
    }

    //  If the next item in the pipe is message delimiter,
    //  initiate termination process.
    if (_in_pipe->probe (is_delimiter)) {
        msg_t msg;
        const bool ok = _in_pipe->read (&msg);
        zmq_assert (ok);
        process_delimiter ();
        return false;
    }

    return true;
}

bool zmq::pipe_t::read (msg_t *msg_)
{
    if (unlikely (!_in_active))
        return false;
    if (unlikely (_state != active && _state != waiting_for_delimiter))
        return false;

    while (true) {
        if (!_in_pipe->read (msg_)) {
            _in_active = false;
            return false;
        }

        //  If this is a credential, ignore it and receive next message.
        if (unlikely (msg_->is_credential ())) {
            const int rc = msg_->close ();
            zmq_assert (rc == 0);
        } else {
            break;
        }
    }

    //  If delimiter was read, start termination process of the pipe.
    if (msg_->is_delimiter ()) {
        process_delimiter ();
        return false;
    }

    if (!(msg_->flags () & msg_t::more) && !msg_->is_routing_id ())
        _msgs_read++;

    if (_lwm > 0 && _msgs_read % _lwm == 0)
        send_activate_write (_peer, _msgs_read);

    return true;
}

bool zmq::pipe_t::check_write ()
{
    if (unlikely (!_out_active || _state != active))
        return false;

    const bool full = !check_hwm ();

    if (unlikely (full)) {
        _out_active = false;
        return false;
    }

    return true;
}

bool zmq::pipe_t::write (const msg_t *msg_)
{
    if (unlikely (!check_write ()))
        return false;

    const bool more = (msg_->flags () & msg_t::more) != 0;
    const bool is_routing_id = msg_->is_routing_id ();
    _out_pipe->write (*msg_, more);
    if (!more && !is_routing_id)
        _msgs_written++;

    return true;
}

void zmq::pipe_t::rollback () const
{
    //  Remove incomplete message from the outbound pipe.
    msg_t msg;
    if (_out_pipe) {
        while (_out_pipe->unwrite (&msg)) {
            zmq_assert (msg.flags () & msg_t::more);
            const int rc = msg.close ();
            errno_assert (rc == 0);
        }
    }
}

void zmq::pipe_t::flush ()
{
    //  The peer does not exist anymore at this point.
    if (_state == term_ack_sent)
        return;

    if (_out_pipe && !_out_pipe->flush ())
        send_activate_read (_peer);
}

void zmq::pipe_t::process_activate_read ()
{
    if (!_in_active && (_state == active || _state == waiting_for_delimiter)) {
        _in_active = true;
        _sink->read_activated (this);
    }
}

void zmq::pipe_t::process_activate_write (uint64_t msgs_read_)
{
    //  Remember the peer's message sequence number.
    _peers_msgs_read = msgs_read_;

    if (!_out_active && _state == active) {
        _out_active = true;
        _sink->write_activated (this);
    }
}

void zmq::pipe_t::process_hiccup (pipe_: *mut c_void)
{
    //  Destroy old outpipe. Note that the read end of the pipe was already
    //  migrated to this thread.
    zmq_assert (_out_pipe);
    _out_pipe->flush ();
    msg_t msg;
    while (_out_pipe->read (&msg)) {
        if (!(msg.flags () & msg_t::more))
            _msgs_written--;
        const int rc = msg.close ();
        errno_assert (rc == 0);
    }
    LIBZMQ_DELETE (_out_pipe);

    //  Plug in the new outpipe.
    zmq_assert (pipe_);
    _out_pipe = static_cast<upipe_t *> (pipe_);
    _out_active = true;

    //  If appropriate, notify the user about the hiccup.
    if (_state == active)
        _sink->hiccuped (this);
}

void zmq::pipe_t::process_pipe_term ()
{
    zmq_assert (_state == active || _state == delimiter_received
                || _state == term_req_sent1);

    //  This is the simple case of peer-induced termination. If there are no
    //  more pending messages to read, or if the pipe was configured to drop
    //  pending messages, we can move directly to the term_ack_sent state.
    //  Otherwise we'll hang up in waiting_for_delimiter state till all
    //  pending messages are read.
    if (_state == active) {
        if (_delay)
            _state = waiting_for_delimiter;
        else {
            _state = term_ack_sent;
            _out_pipe = NULL;
            send_pipe_term_ack (_peer);
        }
    }

    //  Delimiter happened to arrive before the term command. Now we have the
    //  term command as well, so we can move straight to term_ack_sent state.
    else if (_state == delimiter_received) {
        _state = term_ack_sent;
        _out_pipe = NULL;
        send_pipe_term_ack (_peer);
    }

    //  This is the case where both ends of the pipe are closed in parallel.
    //  We simply reply to the request by ack and continue waiting for our
    //  own ack.
    else if (_state == term_req_sent1) {
        _state = term_req_sent2;
        _out_pipe = NULL;
        send_pipe_term_ack (_peer);
    }
}

void zmq::pipe_t::process_pipe_term_ack ()
{
    //  Notify the user that all the references to the pipe should be dropped.
    zmq_assert (_sink);
    _sink->pipe_terminated (this);

    //  In term_ack_sent and term_req_sent2 states there's nothing to do.
    //  Simply deallocate the pipe. In term_req_sent1 state we have to ack
    //  the peer before deallocating this side of the pipe.
    //  All the other states are invalid.
    if (_state == term_req_sent1) {
        _out_pipe = NULL;
        send_pipe_term_ack (_peer);
    } else
        zmq_assert (_state == term_ack_sent || _state == term_req_sent2);

    //  We'll deallocate the inbound pipe, the peer will deallocate the outbound
    //  pipe (which is an inbound pipe from its point of view).
    //  First, delete all the unread messages in the pipe. We have to do it by
    //  hand because msg_t doesn't have automatic destructor. Then deallocate
    //  the ypipe itself.

    if (!_conflate) {
        msg_t msg;
        while (_in_pipe->read (&msg)) {
            const int rc = msg.close ();
            errno_assert (rc == 0);
        }
    }

    LIBZMQ_DELETE (_in_pipe);

    //  Deallocate the pipe object
    delete this;
}

void zmq::pipe_t::process_pipe_hwm (inhwm_: i32, outhwm_: i32)
{
    set_hwms (inhwm_, outhwm_);
}

void zmq::pipe_t::set_nodelay ()
{
    this->_delay = false;
}

void zmq::pipe_t::terminate (bool delay_)
{
    //  Overload the value specified at pipe creation.
    _delay = delay_;

    //  If terminate was already called, we can ignore the duplicate invocation.
    if (_state == term_req_sent1 || _state == term_req_sent2) {
        return;
    }
    //  If the pipe is in the final phase of async termination, it's going to
    //  closed anyway. No need to do anything special here.
    if (_state == term_ack_sent) {
        return;
    }
    //  The simple sync termination case. Ask the peer to terminate and wait
    //  for the ack.
    if (_state == active) {
        send_pipe_term (_peer);
        _state = term_req_sent1;
    }
    //  There are still pending messages available, but the user calls
    //  'terminate'. We can act as if all the pending messages were read.
    else if (_state == waiting_for_delimiter && !_delay) {
        //  Drop any unfinished outbound messages.
        rollback ();
        _out_pipe = NULL;
        send_pipe_term_ack (_peer);
        _state = term_ack_sent;
    }
    //  If there are pending messages still available, do nothing.
    else if (_state == waiting_for_delimiter) {
    }
    //  We've already got delimiter, but not term command yet. We can ignore
    //  the delimiter and ack synchronously terminate as if we were in
    //  active state.
    else if (_state == delimiter_received) {
        send_pipe_term (_peer);
        _state = term_req_sent1;
    }
    //  There are no other states.
    else {
        zmq_assert (false);
    }

    //  Stop outbound flow of messages.
    _out_active = false;

    if (_out_pipe) {
        //  Drop any unfinished outbound messages.
        rollback ();

        //  Write the delimiter into the pipe. Note that watermarks are not
        //  checked; thus the delimiter can be written even when the pipe is full.
        msg_t msg;
        msg.init_delimiter ();
        _out_pipe->write (msg, false);
        flush ();
    }
}

bool zmq::pipe_t::is_delimiter (const msg_t &msg_)
{
    return msg_.is_delimiter ();
}

int zmq::pipe_t::compute_lwm (hwm_: i32)
{
    //  Compute the low water mark. Following point should be taken
    //  into consideration:
    //
    //  1. LWM has to be less than HWM.
    //  2. LWM cannot be set to very low value (such as zero) as after filling
    //     the queue it would start to refill only after all the messages are
    //     read from it and thus unnecessarily hold the progress back.
    //  3. LWM cannot be set to very high value (such as HWM-1) as it would
    //     result in lock-step filling of the queue - if a single message is
    //     read from a full queue, writer thread is resumed to write exactly one
    //     message to the queue and go back to sleep immediately. This would
    //     result in low performance.
    //
    //  Given the 3. it would be good to keep HWM and LWM as far apart as
    //  possible to reduce the thread switching overhead to almost zero.
    //  Let's make LWM 1/2 of HWM.
    const int result = (hwm_ + 1) / 2;

    return result;
}

void zmq::pipe_t::process_delimiter ()
{
    zmq_assert (_state == active || _state == waiting_for_delimiter);

    if (_state == active)
        _state = delimiter_received;
    else {
        rollback ();
        _out_pipe = NULL;
        send_pipe_term_ack (_peer);
        _state = term_ack_sent;
    }
}

void zmq::pipe_t::hiccup ()
{
    //  If termination is already under way do nothing.
    if (_state != active)
        return;

    //  We'll drop the pointer to the inpipe. From now on, the peer is
    //  responsible for deallocating it.

    //  Create new inpipe.
    _in_pipe =
      _conflate
        ? static_cast<upipe_t *> (new (std::nothrow) ypipe_conflate_t<msg_t> ())
        : new (std::nothrow) ypipe_t<msg_t, message_pipe_granularity> ();

    alloc_assert (_in_pipe);
    _in_active = true;

    //  Notify the peer about the hiccup.
    send_hiccup (_peer, _in_pipe);
}

void zmq::pipe_t::set_hwms (inhwm_: i32, outhwm_: i32)
{
    int in = inhwm_ + std::max (_in_hwm_boost, 0);
    int out = outhwm_ + std::max (_out_hwm_boost, 0);

    // if either send or recv side has hwm <= 0 it means infinite so we should set hwms infinite
    if (inhwm_ <= 0 || _in_hwm_boost == 0)
        in = 0;

    if (outhwm_ <= 0 || _out_hwm_boost == 0)
        out = 0;

    _lwm = compute_lwm (in);
    _hwm = out;
}

void zmq::pipe_t::set_hwms_boost (inhwmboost_: i32, outhwmboost_: i32)
{
    _in_hwm_boost = inhwmboost_;
    _out_hwm_boost = outhwmboost_;
}

bool zmq::pipe_t::check_hwm () const
{
    const bool full =
      _hwm > 0 && _msgs_written - _peers_msgs_read >= uint64_t (_hwm);
    return !full;
}

void zmq::pipe_t::send_hwms_to_peer (inhwm_: i32, outhwm_: i32)
{
    send_pipe_hwm (_peer, inhwm_, outhwm_);
}

void zmq::pipe_t::set_endpoint_pair (zmq::endpoint_uri_pair_t endpoint_pair_)
{
    _endpoint_pair = ZMQ_MOVE (endpoint_pair_);
}

const zmq::endpoint_uri_pair_t &zmq::pipe_t::get_endpoint_pair () const
{
    return _endpoint_pair;
}

void zmq::pipe_t::send_stats_to_peer (own_t *socket_base_)
{
    endpoint_uri_pair_t *ep =
      new (std::nothrow) endpoint_uri_pair_t (_endpoint_pair);
    send_pipe_peer_stats (_peer, _msgs_written - _peers_msgs_read, socket_base_,
                          ep);
}

void zmq::pipe_t::process_pipe_peer_stats (queue_count_: u64,
                                           own_t *socket_base_,
                                           endpoint_uri_pair_t *endpoint_pair_)
{
    send_pipe_stats_publish (socket_base_, queue_count_,
                             _msgs_written - _peers_msgs_read, endpoint_pair_);
}

void zmq::pipe_t::send_disconnect_msg ()
{
    if (_disconnect_msg.size () > 0 && _out_pipe) {
        // Rollback any incomplete message in the pipe, and push the disconnect message.
        rollback ();

        _out_pipe->write (_disconnect_msg, false);
        flush ();
        _disconnect_msg.init ();
    }
}

void zmq::pipe_t::set_disconnect_msg (
  const std::vector<unsigned char> &disconnect_)
{
    _disconnect_msg.close ();
    const int rc =
      _disconnect_msg.init_buffer (&disconnect_[0], disconnect_.size ());
    errno_assert (rc == 0);
}

void zmq::pipe_t::send_hiccup_msg (const std::vector<unsigned char> &hiccup_)
{
    if (!hiccup_.empty () && _out_pipe) {
        msg_t msg;
        const int rc = msg.init_buffer (&hiccup_[0], hiccup_.size ());
        errno_assert (rc == 0);

        _out_pipe->write (msg, false);
        flush ();
    }
}
