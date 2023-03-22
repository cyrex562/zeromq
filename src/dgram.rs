/*
    Copyright (c) 2016 Contributors as noted in the AUTHORS file

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
// #include "dgram.hpp"
// #include "pipe.hpp"
// #include "wire.hpp"
// #include "random.hpp"
// #include "likely.hpp"
// #include "err.hpp"
pub struct dgram_t ZMQ_FINAL : public ZmqSocketBase
{
// public:
    dgram_t (ZmqContext *parent_, tid: u32, sid_: i32);
    ~dgram_t ();

    //  Overrides of functions from ZmqSocketBase.
    void xattach_pipe (pipe: &mut ZmqPipe,
                       subscribe_to_all_: bool,
                       locally_initiated_: bool);
    int xsend (msg: &mut ZmqMessage);
    int xrecv (msg: &mut ZmqMessage);
    bool xhas_in ();
    bool xhas_out ();
    void xread_activated (pipe: &mut ZmqPipe);
    void xwrite_activated (pipe: &mut ZmqPipe);
    void xpipe_terminated (pipe: &mut ZmqPipe);

  // private:
    ZmqPipe *_pipe;

    //  If true, more outgoing message parts are expected.
    _more_out: bool

    ZMQ_NON_COPYABLE_NOR_MOVABLE (dgram_t)
};
dgram_t::dgram_t (class ZmqContext *parent_, tid: u32, sid_: i32) :
    ZmqSocketBase (parent_, tid, sid_), _pipe (null_mut()), _more_out (false)
{
    options.type = ZMQ_DGRAM;
    options.raw_socket = true;
}

dgram_t::~dgram_t ()
{
    zmq_assert (!_pipe);
}

void dgram_t::xattach_pipe (pipe: &mut ZmqPipe,
                                 subscribe_to_all_: bool,
                                 locally_initiated_: bool)
{
    LIBZMQ_UNUSED (subscribe_to_all_);
    LIBZMQ_UNUSED (locally_initiated_);

    zmq_assert (pipe);

    //  ZMQ_DGRAM socket can only be connected to a single peer.
    //  The socket rejects any further connection requests.
    if (_pipe == null_mut())
        _pipe = pipe;
    else
        pipe.terminate (false);
}

void dgram_t::xpipe_terminated (pipe: &mut ZmqPipe)
{
    if (pipe == _pipe) {
        _pipe = null_mut();
    }
}

void dgram_t::xread_activated (ZmqPipe *)
{
    //  There's just one pipe. No lists of active and inactive pipes.
    //  There's nothing to do here.
}

void dgram_t::xwrite_activated (ZmqPipe *)
{
    //  There's just one pipe. No lists of active and inactive pipes.
    //  There's nothing to do here.
}

int dgram_t::xsend (msg: &mut ZmqMessage)
{
    // If there's no out pipe, just drop it.
    if (!_pipe) {
        let rc: i32 = msg.close ();
        errno_assert (rc == 0);
        return -1;
    }

    //  If this is the first part of the message it's the ID of the
    //  peer to send the message to.
    if (!_more_out) {
        if (!(msg.flags () & ZMQ_MSG_MORE)) {
            errno = EINVAL;
            return -1;
        }
    } else {
        //  dgram messages are two part only, reject part if more is set
        if (msg.flags () & ZMQ_MSG_MORE) {
            errno = EINVAL;
            return -1;
        }
    }

    // Push the message into the pipe.
    if (!_pipe.write (msg)) {
        errno = EAGAIN;
        return -1;
    }

    if (!(msg.flags () & ZMQ_MSG_MORE))
        _pipe.flush ();

    // flip the more flag
    _more_out = !_more_out;

    //  Detach the message from the data buffer.
    let rc: i32 = msg.init ();
    errno_assert (rc == 0);

    return 0;
}

int dgram_t::xrecv (msg: &mut ZmqMessage)
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

bool dgram_t::xhas_in ()
{
    if (!_pipe)
        return false;

    return _pipe.check_read ();
}

bool dgram_t::xhas_out ()
{
    if (!_pipe)
        return false;

    return _pipe.check_write ();
}
