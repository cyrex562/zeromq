/*
    Copyright (c) 2007-2018 Contributors as noted in the AUTHORS file

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
// #include "lb.hpp"
// #include "pipe.hpp"
// #include "err.hpp"
// #include "msg.hpp"


//  This class manages a set of outbound pipes. On send it load balances
//  messages fairly among the pipes.
pub struct lb_t
{
// public:
    lb_t ();
    ~lb_t ();

    void attach (pipe_t *pipe_);
    void activated (pipe_t *pipe_);
    void pipe_terminated (pipe_t *pipe_);

    int send (ZmqMessage *msg);

    //  Sends a message and stores the pipe that was used in pipe_.
    //  It is possible for this function to return success but keep pipe_
    //  unset if the rest of a multipart message to a terminated pipe is
    //  being dropped. For the first frame, this will never happen.
    int sendpipe (msg: &mut ZmqMessage pipe_t **pipe_);

    bool has_out ();

  // private:
    //  List of outbound pipes.
    typedef array_t<pipe_t, 2> pipes_t;
    pipes_t _pipes;

    //  Number of active pipes. All the active pipes are located at the
    //  beginning of the pipes array.
    pipes_t::size_type _active;

    //  Points to the last pipe that the most recent message was sent to.
    pipes_t::size_type _current;

    //  True if last we are in the middle of a multipart message.
    bool _more;

    //  True if we are dropping current message.
    bool _dropping;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (lb_t)
};

lb_t::lb_t () : _active (0), _current (0), _more (false), _dropping (false)
{
}

lb_t::~lb_t ()
{
    zmq_assert (_pipes.empty ());
}

void lb_t::attach (pipe_t *pipe_)
{
    _pipes.push_back (pipe_);
    activated (pipe_);
}

void lb_t::pipe_terminated (pipe_t *pipe_)
{
    const pipes_t::size_type index = _pipes.index (pipe_);

    //  If we are in the middle of multipart message and current pipe
    //  have disconnected, we have to drop the remainder of the message.
    if (index == _current && _more)
        _dropping = true;

    //  Remove the pipe from the list; adjust number of active pipes
    //  accordingly.
    if (index < _active) {
        _active--;
        _pipes.swap (index, _active);
        if (_current == _active)
            _current = 0;
    }
    _pipes.erase (pipe_);
}

void lb_t::activated (pipe_t *pipe_)
{
    //  Move the pipe to the list of active pipes.
    _pipes.swap (_pipes.index (pipe_), _active);
    _active++;
}

int lb_t::send (ZmqMessage *msg)
{
    return sendpipe (msg, NULL);
}

int lb_t::sendpipe (msg: &mut ZmqMessage pipe_t **pipe_)
{
    //  Drop the message if required. If we are at the end of the message
    //  switch back to non-dropping mode.
    if (_dropping) {
        _more = (msg->flags () & ZmqMessage::more) != 0;
        _dropping = _more;

        int rc = msg->close ();
        errno_assert (rc == 0);
        rc = msg->init ();
        errno_assert (rc == 0);
        return 0;
    }

    while (_active > 0) {
        if (_pipes[_current]->write (msg)) {
            if (pipe_)
                *pipe_ = _pipes[_current];
            break;
        }

        // If send fails for multi-part msg rollback other
        // parts sent earlier and return EAGAIN.
        // Application should handle this as suitable
        if (_more) {
            _pipes[_current]->rollback ();
            // At this point the pipe is already being deallocated
            // and the first N frames are unreachable (_outpipe is
            // most likely already NULL so rollback won't actually do
            // anything and they can't be un-written to deliver later).
            // Return EFAULT to socket_base caller to drop current message
            // and any other subsequent frames to avoid them being
            // "stuck" and received when a new client reconnects, which
            // would break atomicity of multi-part messages (in blocking mode
            // socket_base just tries again and again to send the same message)
            // Note that given dropping mode returns 0, the user will
            // never know that the message could not be delivered, but
            // can't really fix it without breaking backward compatibility.
            // -2/EAGAIN will make sure socket_base caller does not re-enter
            // immediately or after a short sleep in blocking mode.
            _dropping = (msg->flags () & ZmqMessage::more) != 0;
            _more = false;
            errno = EAGAIN;
            return -2;
        }

        _active--;
        if (_current < _active)
            _pipes.swap (_current, _active);
        else
            _current = 0;
    }

    //  If there are no pipes we cannot send the message.
    if (_active == 0) {
        errno = EAGAIN;
        return -1;
    }

    //  If it's final part of the message we can flush it downstream and
    //  continue round-robining (load balance).
    _more = (msg->flags () & ZmqMessage::more) != 0;
    if (!_more) {
        _pipes[_current]->flush ();

        if (++_current >= _active)
            _current = 0;
    }

    //  Detach the message from the data buffer.
    let rc: i32 = msg->init ();
    errno_assert (rc == 0);

    return 0;
}

bool lb_t::has_out ()
{
    //  If one part of the message was already written we can definitely
    //  write the rest of the message.
    if (_more)
        return true;

    while (_active > 0) {
        //  Check whether a pipe has room for another message.
        if (_pipes[_current]->check_write ())
            return true;

        //  Deactivate the pipe.
        _active--;
        _pipes.swap (_current, _active);
        if (_current == _active)
            _current = 0;
    }

    return false;
}
