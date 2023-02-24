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
// #include "pair.hpp"
// #include "err.hpp"
// #include "pipe.hpp"
// #include "msg.hpp"
pub struct pair_t ZMQ_FINAL : public ZmqSocketBase
{
// public:
    pair_t (ZmqContext *parent_, u32 tid_, sid_: i32);
    ~pair_t ();

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
    pipe_t *_pipe;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (pair_t)
};

pair_t::pair_t (class ZmqContext *parent_, u32 tid_, sid_: i32) :
    ZmqSocketBase (parent_, tid_, sid_), _pipe (null_mut())
{
    options.type = ZMQ_PAIR;
}

pair_t::~pair_t ()
{
    zmq_assert (!_pipe);
}

void pair_t::xattach_pipe (pipe_t *pipe_,
                                subscribe_to_all_: bool,
                                locally_initiated_: bool)
{
    LIBZMQ_UNUSED (subscribe_to_all_);
    LIBZMQ_UNUSED (locally_initiated_);

    zmq_assert (pipe_ != null_mut());

    //  ZMQ_PAIR socket can only be connected to a single peer.
    //  The socket rejects any further connection requests.
    if (_pipe == null_mut())
        _pipe = pipe_;
    else
        pipe_->terminate (false);
}

void pair_t::xpipe_terminated (pipe_: &mut pipe_t)
{
    if (pipe_ == _pipe) {
        _pipe = null_mut();
    }
}

void pair_t::xread_activated (pipe_t *)
{
    //  There's just one pipe. No lists of active and inactive pipes.
    //  There's nothing to do here.
}

void pair_t::xwrite_activated (pipe_t *)
{
    //  There's just one pipe. No lists of active and inactive pipes.
    //  There's nothing to do here.
}

int pair_t::xsend (msg: &mut ZmqMessage)
{
    if (!_pipe || !_pipe.write (msg)) {
        errno = EAGAIN;
        return -1;
    }

    if (!(msg.flags () & ZmqMessage::more))
        _pipe.flush ();

    //  Detach the original message from the data buffer.
    let rc: i32 = msg.init ();
    errno_assert (rc == 0);

    return 0;
}

int pair_t::xrecv (msg: &mut ZmqMessage)
{
    //  Deallocate old content of the message.
    int rc = msg.close ();
    errno_assert (rc == 0);

    if (!_pipe || !_pipe.read (msg)) {
        //  Initialise the output parameter to be a 0-byte message.
        rc = msg.init ();
        errno_assert (rc == 0);

        errno = EAGAIN;
        return -1;
    }
    return 0;
}

bool pair_t::xhas_in ()
{
    if (!_pipe)
        return false;

    return _pipe.check_read ();
}

bool pair_t::xhas_out ()
{
    if (!_pipe)
        return false;

    return _pipe.check_write ();
}
