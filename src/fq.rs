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
// #include "fq.hpp"
// #include "pipe.hpp"
// #include "err.hpp"
// #include "msg.hpp"
pub struct fq_t
{
// public:
    fq_t ();
    ~fq_t ();

    void attach (pipe_: &mut pipe_t);
    void activated (pipe_: &mut pipe_t);
    void pipe_terminated (pipe_: &mut pipe_t);

    int recv (msg: &mut ZmqMessage);
    int recvpipe (msg: &mut ZmqMessage pipe_t **pipe_);
    bool has_in ();

  // private:
    //  Inbound pipes.
    typedef array_t<pipe_t, 1> pipes_t;
    pipes_t _pipes;

    //  Number of active pipes. All the active pipes are located at the
    //  beginning of the pipes array.
    pipes_t::size_type _active;

    //  Index of the next bound pipe to read a message from.
    pipes_t::size_type _current;

    //  If true, part of a multipart message was already received, but
    //  there are following parts still waiting in the current pipe.
    _more: bool

    ZMQ_NON_COPYABLE_NOR_MOVABLE (fq_t)
};

fq_t::fq_t () : _active (0), _current (0), _more (false)
{
}

fq_t::~fq_t ()
{
    zmq_assert (_pipes.empty ());
}

void fq_t::attach (pipe_: &mut pipe_t)
{
    _pipes.push_back (pipe_);
    _pipes.swap (_active, _pipes.size () - 1);
    _active++;
}

void fq_t::pipe_terminated (pipe_: &mut pipe_t)
{
    const pipes_t::size_type index = _pipes.index (pipe_);

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

void fq_t::activated (pipe_: &mut pipe_t)
{
    //  Move the pipe to the list of active pipes.
    _pipes.swap (_pipes.index (pipe_), _active);
    _active++;
}

int fq_t::recv (msg: &mut ZmqMessage)
{
    return recvpipe (msg, null_mut());
}

int fq_t::recvpipe (msg: &mut ZmqMessage pipe_t **pipe_)
{
    //  Deallocate old content of the message.
    int rc = msg.close ();
    errno_assert (rc == 0);

    //  Round-robin over the pipes to get the next message.
    while (_active > 0) {
        //  Try to fetch new message. If we've already read part of the message
        //  subsequent part should be immediately available.
        const bool fetched = _pipes[_current]->read (msg);

        //  Note that when message is not fetched, current pipe is deactivated
        //  and replaced by another active pipe. Thus we don't have to increase
        //  the 'current' pointer.
        if (fetched) {
            if (pipe_)
                *pipe_ = _pipes[_current];
            _more = (msg.flags () & ZMQ_MSG_MORE) != 0;
            if (!_more) {
                _current = (_current + 1) % _active;
            }
            return 0;
        }

        //  Check the atomicity of the message.
        //  If we've already received the first part of the message
        //  we should get the remaining parts without blocking.
        zmq_assert (!_more);

        _active--;
        _pipes.swap (_current, _active);
        if (_current == _active)
            _current = 0;
    }

    //  No message is available. Initialise the output parameter
    //  to be a 0-byte message.
    rc = msg.init ();
    errno_assert (rc == 0);
    errno = EAGAIN;
    return -1;
}

bool fq_t::has_in ()
{
    //  There are subsequent parts of the partly-read message available.
    if (_more)
        return true;

    //  Note that messing with current doesn't break the fairness of fair
    //  queueing algorithm. If there are no messages available current will
    //  get back to its original value. Otherwise it'll point to the first
    //  pipe holding messages, skipping only pipes with no messages available.
    while (_active > 0) {
        if (_pipes[_current]->check_read ())
            return true;

        //  Deactivate the pipe.
        _active--;
        _pipes.swap (_current, _active);
        if (_current == _active)
            _current = 0;
    }

    return false;
}
