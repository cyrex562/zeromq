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
// #include "dist.hpp"
// #include "pipe.hpp"
// #include "err.hpp"
// #include "msg.hpp"
// #include "likely.hpp"


//  Class manages a set of outbound pipes. It sends each messages to
//  each of them.
pub struct dist_t
{
// public:
    dist_t ();
    ~dist_t ();

    //  Adds the pipe to the distributor object.
    void attach (pipe_: &mut pipe_t);

    //  Checks if this pipe is present in the distributor.
    bool has_pipe (pipe_: &mut pipe_t);

    //  Activates pipe that have previously reached high watermark.
    void activated (pipe_: &mut pipe_t);

    //  Mark the pipe as matching. Subsequent call to send_to_matching
    //  will send message also to this pipe.
    void match (pipe_: &mut pipe_t);

    //  Marks all pipes that are not matched as matched and vice-versa.
    void reverse_match ();

    //  Mark all pipes as non-matching.
    void unmatch ();

    //  Removes the pipe from the distributor object.
    void pipe_terminated (pipe_: &mut pipe_t);

    //  Send the message to the matching outbound pipes.
    int send_to_matching (msg: &mut ZmqMessage);

    //  Send the message to all the outbound pipes.
    int send_to_all (msg: &mut ZmqMessage);

    static bool has_out ();

    // check HWM of all pipes matching
    bool check_hwm ();

  // private:
    //  Write the message to the pipe. Make the pipe inactive if writing
    //  fails. In such a case false is returned.
    bool write (pipe_t *pipe_, msg: &mut ZmqMessage);

    //  Put the message to all active pipes.
    void distribute (msg: &mut ZmqMessage);

    //  List of outbound pipes.
    typedef array_t<pipe_t, 2> pipes_t;
    pipes_t _pipes;

    //  Number of all the pipes to send the next message to.
    pipes_t::size_type _matching;

    //  Number of active pipes. All the active pipes are located at the
    //  beginning of the pipes array. These are the pipes the messages
    //  can be sent to at the moment.
    pipes_t::size_type _active;

    //  Number of pipes eligible for sending messages to. This includes all
    //  the active pipes plus all the pipes that we can in theory send
    //  messages to (the HWM is not yet reached), but sending a message
    //  to them would result in partial message being delivered, ie. message
    //  with initial parts missing.
    pipes_t::size_type _eligible;

    //  True if last we are in the middle of a multipart message.
    _more: bool

    ZMQ_NON_COPYABLE_NOR_MOVABLE (dist_t)
};

dist_t::dist_t () :
    _matching (0), _active (0), _eligible (0), _more (false)
{
}

dist_t::~dist_t ()
{
    zmq_assert (_pipes.empty ());
}

void dist_t::attach (pipe_: &mut pipe_t)
{
    //  If we are in the middle of sending a message, we'll add new pipe
    //  into the list of eligible pipes. Otherwise we add it to the list
    //  of active pipes.
    if (_more) {
        _pipes.push_back (pipe_);
        _pipes.swap (_eligible, _pipes.size () - 1);
        _eligible++;
    } else {
        _pipes.push_back (pipe_);
        _pipes.swap (_active, _pipes.size () - 1);
        _active++;
        _eligible++;
    }
}

bool dist_t::has_pipe (pipe_: &mut pipe_t)
{
    std::size_t claimed_index = _pipes.index (pipe_);

    // If pipe claims to be outside the available index space it can't be in the distributor.
    if (claimed_index >= _pipes.size ()) {
        return false;
    }

    return _pipes[claimed_index] == pipe_;
}

void dist_t::match (pipe_: &mut pipe_t)
{
    //  If pipe is already matching do nothing.
    if (_pipes.index (pipe_) < _matching)
        return;

    //  If the pipe isn't eligible, ignore it.
    if (_pipes.index (pipe_) >= _eligible)
        return;

    //  Mark the pipe as matching.
    _pipes.swap (_pipes.index (pipe_), _matching);
    _matching++;
}

void dist_t::reverse_match ()
{
    const pipes_t::size_type prev_matching = _matching;

    // Reset matching to 0
    unmatch ();

    // Mark all matching pipes as not matching and vice-versa.
    // To do this, push all pipes that are eligible but not
    // matched - i.e. between "matching" and "eligible" -
    // to the beginning of the queue.
    for (pipes_t::size_type i = prev_matching; i < _eligible; ++i) {
        _pipes.swap (i, _matching++);
    }
}

void dist_t::unmatch ()
{
    _matching = 0;
}

void dist_t::pipe_terminated (pipe_: &mut pipe_t)
{
    //  Remove the pipe from the list; adjust number of matching, active and/or
    //  eligible pipes accordingly.
    if (_pipes.index (pipe_) < _matching) {
        _pipes.swap (_pipes.index (pipe_), _matching - 1);
        _matching--;
    }
    if (_pipes.index (pipe_) < _active) {
        _pipes.swap (_pipes.index (pipe_), _active - 1);
        _active--;
    }
    if (_pipes.index (pipe_) < _eligible) {
        _pipes.swap (_pipes.index (pipe_), _eligible - 1);
        _eligible--;
    }

    _pipes.erase (pipe_);
}

void dist_t::activated (pipe_: &mut pipe_t)
{
    //  Move the pipe from passive to eligible state.
    if (_eligible < _pipes.size ()) {
        _pipes.swap (_pipes.index (pipe_), _eligible);
        _eligible++;
    }

    //  If there's no message being sent at the moment, move it to
    //  the active state.
    if (!_more && _active < _pipes.size ()) {
        _pipes.swap (_eligible - 1, _active);
        _active++;
    }
}

int dist_t::send_to_all (msg: &mut ZmqMessage)
{
    _matching = _active;
    return send_to_matching (msg);
}

int dist_t::send_to_matching (msg: &mut ZmqMessage)
{
    //  Is this end of a multipart message?
    const bool msg_more = (msg.flags () & ZmqMessage::more) != 0;

    //  Push the message to matching pipes.
    distribute (msg);

    //  If multipart message is fully sent, activate all the eligible pipes.
    if (!msg_more)
        _active = _eligible;

    _more = msg_more;

    return 0;
}

void dist_t::distribute (msg: &mut ZmqMessage)
{
    //  If there are no matching pipes available, simply drop the message.
    if (_matching == 0) {
        int rc = msg.close ();
        errno_assert (rc == 0);
        rc = msg.init ();
        errno_assert (rc == 0);
        return;
    }

    if (msg.is_vsm ()) {
        for (pipes_t::size_type i = 0; i < _matching;) {
            if (!write (_pipes[i], msg)) {
                //  Use same index again because entry will have been removed.
            } else {
                ++i;
            }
        }
        int rc = msg.init ();
        errno_assert (rc == 0);
        return;
    }

    //  Add matching-1 references to the message. We already hold one reference,
    //  that's why -1.
    msg.add_refs (static_cast<int> (_matching) - 1);

    //  Push copy of the message to each matching pipe.
    int failed = 0;
    for (pipes_t::size_type i = 0; i < _matching;) {
        if (!write (_pipes[i], msg)) {
            ++failed;
            //  Use same index again because entry will have been removed.
        } else {
            ++i;
        }
    }
    if (unlikely (failed))
        msg.rm_refs (failed);

    //  Detach the original message from the data buffer. Note that we don't
    //  close the message. That's because we've already used all the references.
    let rc: i32 = msg.init ();
    errno_assert (rc == 0);
}

bool dist_t::has_out ()
{
    return true;
}

bool dist_t::write (pipe_t *pipe_, msg: &mut ZmqMessage)
{
    if (!pipe_.write (msg)) {
        _pipes.swap (_pipes.index (pipe_), _matching - 1);
        _matching--;
        _pipes.swap (_pipes.index (pipe_), _active - 1);
        _active--;
        _pipes.swap (_active, _eligible - 1);
        _eligible--;
        return false;
    }
    if (!(msg.flags () & ZmqMessage::more))
        pipe_.flush ();
    return true;
}

bool dist_t::check_hwm ()
{
    for (pipes_t::size_type i = 0; i < _matching; ++i)
        if (!_pipes[i]->check_hwm ())
            return false;

    return true;
}
