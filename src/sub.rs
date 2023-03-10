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
// #include "sub.hpp"
// #include "msg.hpp"
pub struct sub_t ZMQ_FINAL : public xsub_t
{
// public:
    sub_t (ZmqContext *parent_, u32 tid_, sid_: i32);
    ~sub_t ();

  protected:
    int xsetsockopt (option_: i32, const optval_: *mut c_void, optvallen_: usize);
    int xsend (msg: &mut ZmqMessage);
    bool xhas_out ();

    ZMQ_NON_COPYABLE_NOR_MOVABLE (sub_t)
};

sub_t::sub_t (class ZmqContext *parent_, u32 tid_, sid_: i32) :
    xsub_t (parent_, tid_, sid_)
{
    options.type = ZMQ_SUB;

    //  Switch filtering messages on (as opposed to XSUB which where the
    //  filtering is off).
    options.filter = true;
}

sub_t::~sub_t ()
{
}

int sub_t::xsetsockopt (option_: i32,
                             const optval_: *mut c_void,
                             optvallen_: usize)
{
    if (option_ != ZMQ_SUBSCRIBE && option_ != ZMQ_UNSUBSCRIBE) {
        errno = EINVAL;
        return -1;
    }

    //  Create the subscription message.
    ZmqMessage msg;
    rc: i32;
    const unsigned char *data = static_cast<const unsigned char *> (optval_);
    if (option_ == ZMQ_SUBSCRIBE) {
        rc = msg.init_subscribe (optvallen_, data);
    } else {
        rc = msg.init_cancel (optvallen_, data);
    }
    errno_assert (rc == 0);

    //  Pass it further on in the stack.
    rc = xsub_t::xsend (&msg);
    return close_and_return (&msg, rc);
}

int sub_t::xsend (ZmqMessage *)
{
    //  Override the XSUB's send.
    errno = ENOTSUP;
    return -1;
}

bool sub_t::xhas_out ()
{
    //  Override the XSUB's send.
    return false;
}
