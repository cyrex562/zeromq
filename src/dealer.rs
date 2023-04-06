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
// #include "dealer.hpp"
// #include "err.hpp"
// #include "msg.hpp"
// pub struct ZmqDealer : public ZmqSocketBase

use std::collections::VecDeque;

use crate::lb::lb_t;

#[derive(Default,Debug,Clone)]
pub struct ZmqDealer
{
    pub socket_base: ZmqSocketBase,


  // private:
    //  Messages are fair-queued from inbound pipes. And load-balanced to
    //  the outbound pipes.
    // fq_t _fq;
    pub _fq: VecDeque<ZmqMessage>,

    // lb_t _lb;
    pub _lb: lb_t,

    // if true, send an empty message to every connected router peer
    pub _probe_router: bool

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqDealer)
}

impl ZmqDealer {
    // public:
    // ZmqDealer (ZmqContext *parent_, tid: u32, sid_: i32);
    // ZmqSocketBase (parent_, tid, sid_), _probe_router (false)
pub fn new(options: &mut ZmqOptions, parent: &mut ZmqContext, tid: u32, sid_: i32) -> ZmqDealer

{
    let mut out = Self::default();
    let mut base = ZmqSocketBase::new2(options, parent, tid, sid_);

    options.type_ = ZMQ_DEALER;
    options.can_send_hello_msg = true;
    options.can_recv_hiccup_msg = true;
    out
}


    // ~ZmqDealer () ZMQ_OVERRIDE;

//   protected:
    //  Overrides of functions from ZmqSocketBase.
    // void xattach_pipe (pipe: &mut ZmqPipe,
                    //    subscribe_to_all_: bool,
                    //    locally_initiated_: bool) ZMQ_FINAL;

pub fn xattach_pipe (&mut self, pipe: &mut ZmqPipe,
    subscribe_to_all_: bool,
    locally_initiated_: bool) -> anyhow::Result<()>
{
// LIBZMQ_UNUSED (subscribe_to_all_);
// LIBZMQ_UNUSED (locally_initiated_);

// zmq_assert (pipe);

if (self, _probe_router) {
let probe_msg: ZmqMessage = ZmqMessage::default();
probe_msg.init ()?;
// errno_assert (rc == 0);

rc = pipe.write (&probe_msg);
// zmq_assert (rc) is not applicable here, since it is not a bug.
LIBZMQ_UNUSED (rc);

pipe.flush ();

rc = probe_msg.close ();
errno_assert (rc == 0);
}

self._fq.attach (pipe);
self._lb.attach (pipe);
Ok(())
}



    // int xsetsockopt (option_: i32,

    //                  const optval_: &mut [u8],
    //                  optvallen_: usize) ZMQ_OVERRIDE;

pub fn xsetsockopt (option_: i32,
    optval_: &mut [u8],
    optvallen_: usize) -> anyhow::Result<()>
{
let is_int = (optvallen_ == mem::size_of::<int>());
let mut value = 0;
if (is_int){
let mut val_bytes: [u8;4] = [0;4];
// memcpy (&value, optval_, mem::size_of::<int>());
copy_bytes(&mut val_bytes, 0, optval_, 0, 4);
value = i32::from_le_bytes(val_bytes);
}

match (option_) {
ZMQ_PROBE_ROUTER =>{
if (is_int && value >= 0) {
_probe_router = (value != 0);
return Ok(());
}}
// break;

_ =>{}
// break;
}

// errno = EINVAL;
// return -1;
return anyhow!("EINVAL");
}

    // int xsend (msg: &mut ZmqMessage) ZMQ_OVERRIDE;


    // int xrecv (msg: &mut ZmqMessage) ZMQ_OVERRIDE;


    // bool xhas_in () ZMQ_OVERRIDE;


    // bool xhas_out () ZMQ_OVERRIDE;


    // void xread_activated (pipe: &mut ZmqPipe) ZMQ_FINAL;


    // void xwrite_activated (pipe: &mut ZmqPipe) ZMQ_FINAL;


    // void xpipe_terminated (pipe: &mut ZmqPipe) ZMQ_OVERRIDE;

    //  Send and recv - knowing which pipe was used.


    // int sendpipe (msg: &mut ZmqMessage ZmqPipe **pipe);


    // int recvpipe (msg: &mut ZmqMessage ZmqPipe **pipe);
}


// ZmqDealer::~ZmqDealer ()
// {
// }


int ZmqDealer::xsend (msg: &mut ZmqMessage)
{
    return sendpipe (msg, null_mut());
}

int ZmqDealer::xrecv (msg: &mut ZmqMessage)
{
    return recvpipe (msg, null_mut());
}

bool ZmqDealer::xhas_in ()
{
    return _fq.has_in ();
}

bool ZmqDealer::xhas_out ()
{
    return _lb.has_out ();
}

void ZmqDealer::xread_activated (pipe: &mut ZmqPipe)
{
    _fq.activated (pipe);
}

void ZmqDealer::xwrite_activated (pipe: &mut ZmqPipe)
{
    _lb.activated (pipe);
}

void ZmqDealer::xpipe_terminated (pipe: &mut ZmqPipe)
{
    _fq.pipe_terminated (pipe);
    _lb.pipe_terminated (pipe);
}

int ZmqDealer::sendpipe (msg: &mut ZmqMessage ZmqPipe **pipe)
{
    return _lb.sendpipe (msg, pipe);
}

int ZmqDealer::recvpipe (msg: &mut ZmqMessage ZmqPipe **pipe)
{
    return _fq.recvpipe (msg, pipe);
}
