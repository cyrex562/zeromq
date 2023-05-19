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

// "Tell them I was a writer.
//  A maker of software.
//  A humanist. A father.
//  And many things.
//  But above all, a writer.
//  Thank You. :)"
//  - Pieter Hintjens

// #include "precompiled.hpp"
// #define ZMQ_TYPE_UNSAFE

// #include "macros.hpp"
// #include "poller.hpp"
// #include "peer.hpp"

// #if !defined ZMQ_HAVE_POLLER
//  On AIX platform, poll.h has to be included first to get consistent
//  definition of pollfd structure (AIX uses 'reqevents' and 'retnevents'
//  instead of 'events' and 'revents' and defines macros to map from POSIX-y
//  names to AIX-specific names).
// #if defined ZMQ_POLL_BASED_ON_POLL && !defined ZMQ_HAVE_WINDOWS
// #include <poll.h>
// #endif

// #include "polling_util.hpp"
// #endif

// TODO: determine if this is an issue, since zmq.h is being loaded from pch.
// zmq.h must be included *after* poll.h for AIX to build properly
//#include "../include/zmq.h"

// #if !defined ZMQ_HAVE_WINDOWS
// #include <unistd.h>
// #ifdef ZMQ_HAVE_VXWORKS
// #include <strings.h>
// #endif
// #endif

use std::intrinsics::// zmq_assert;
use std::mem;
use std::ptr::null_mut;
use libc::{atoi, c_char, c_void, EFAULT, EINTR, EINVAL, ENOMEM, ENOTSOCK, ENOTSUP, INT_MAX};
use crate::context::ZmqContext;
use crate::ctx_hdr::ZmqContext;
use crate::peer::ZmqPeer;
use crate::socket_base::ZmqSocketBase;
use crate::defines::{zmq_free_fn, ZMQ_IO_THREADS, ZmqMessage, ZMQ_PAIR, ZMQ_PEER, ZMQ_SNDMORE, ZMQ_TYPE, ZMQ_VERSION_MAJOR, ZMQ_VERSION_MINOR, ZMQ_VERSION_PATCH, ZMQ_MORE, ZMQ_SRCFD, ZMQ_SHARED};
use anyhow::{anyhow, bail};
use bincode::options;
use serde::Serialize;
use crate::message::{ZMQ_MSG_MORE, ZMQ_MSG_SHARED, ZmqMessage};
use crate::options::ZmqOptions;


pub fn zmq_version (major_: *mut u32, minor_: *mut u32, patch_: *mut u32)
{
    unsafe {
    *major_ = ZMQ_VERSION_MAJOR;
    *minor_ = ZMQ_VERSION_MINOR;
    *patch_ = ZMQ_VERSION_PATCH;}
}


pub fn zmq_strerror (errnum_: i32) -> String
{
    return errno_to_string (errnum_);
}

pub fn zmq_errno () -> i32
{
    return errno;
}

//  New context API
pub fn zmq_ctx_new() -> anyhow::Result<Vec<u8>> {
    //  We do this before the ctx constructor since its embedded mailbox_t
    //  object needs the network to be up and running (at least on Windows).
    if !initialize_network() {
        bail!("failed to initialize network");
    }

    //  Create 0MQ context.
    let mut ctx = ZmqContext::new();
    if ctx.valid() == false {
        bail!("ctx failed validity check");
    }
    return Ok(bincode::serialize(&ctx)?);
}

pub fn zmq_ctx_term (ctx_raw: &mut [u8]) -> anyhow::Result<()>
{
    if ctx_raw.len() == 0 {
        bail!("context buffer is empty")
    }

    let mut ctx: ZmqContext = bincode::deserialize(ctx_raw)?;

    if ctx.check_tag() == false {
        bail!("check tag failed")
    }

    // if ctx.is_null() == false || !(ctx as *mut ZmqContext).check_tag() {
    //     errno = EFAULT;
    //     return -1;
    // }

    // let rc = (ctx as *mut ZmqContext).terminate();
    // let en = errno;
    match ctx.terminate() {
        Ok(_) =>{
            Ok(())
        },
        Err(e) =>{
            shutdown_network();
            Err(e)
        }
    }

    //  Shut down only if termination was not interrupted by a signal.
    // if !rc || en != EINTR {
    //     shutdown_network();
    // }
    //
    // errno = en;
    // return rc;
}

pub fn zmq_ctx_shutdown (ctx_raw: &mut [u8]) -> anyhow::Result<()>
{
    // if (!ctx || !(ctx as *mut ZmqContext).check_tag ()) {
    //     errno = EFAULT;
    //     return -1;
    // }
    // return (ctx as *mut ZmqContext).shutdown ();
    if ctx_raw.len() == 0 {
        bail!("context buffer is empty")
    }
    let mut ctx: ZmqContext = bincode::deserialize(ctx_raw)?;
    if ctx.check_tag() == false {
        bail!("check tag failed")
    }

    ctx.shutdown();
    Ok(())
}

pub fn zmq_ctx_set (ctx_raw: &mut[u8], option_: i32, mut optval_: i32) -> anyhow::Result<()>
{
    return zmq_ctx_set_ext (ctx_raw, option_, optval_.to_le_bytes().as_mut_slice());
}

pub fn zmq_ctx_set_ext (ctx_raw: &mut [u8],
                     option_: i32,
                     optval_: &mut[u8]) -> anyhow::Result<()>
{
    // if !ctx || !(ctx as *mut ZmqContext).check_tag () {
    //     errno = EFAULT;
    //     return -1;
    // }
    // return (ctx as *mut ZmqContext).set(option_, optval_, optvallen_);
    if ctx_raw.len() == 0 {
        bail!("context buffer is empty")
    }
    let mut ctx: ZmqContext = bincode::deserialize(ctx_raw)?;
    if ctx.check_tag() == false {
        bail!("check tag failed")
    }

    ctx.set(option_, optval_, optval_.len())
}

pub fn zmq_ctx_get (ctx_raw: &mut [u8], opt_kind: i32) -> anyhow::Result<i32>
{
    // if !ctx || !(ctx as *mut ZmqContext).check_tag () {
    //     errno = EFAULT;
    //     return -1;
    // }
    // return (ctx as *mut ZmqContext).get (option_);
    if ctx_raw.len() == 0 {
        bail!("context buffer is empty")
    }
    let mut ctx: ZmqContext = bincode::deserialize(ctx_raw)?;
    if ctx.check_tag() == false {
        bail!("check tag failed")
    }
    return ctx.option_i32(opt_kind)
}

pub fn zmq_ctx_get_ext (ctx_raw: &mut [u8], opt_kind: i32) -> anyhow::Result<Vec<u8>>
{
    // if !ctx || !(ctx as *mut ZmqContext).check_tag () {
    //     errno = EFAULT;
    //     return -1;
    // }
    // return (ctx as *mut ZmqContext).get(option_, optval_, optvallen_);
    if ctx_raw.len() == 0 {
        bail!("context buffer is empty")
    }
    let mut ctx: ZmqContext = bincode::deserialize(ctx_raw)?;
    if ctx.check_tag() == false {
        bail!("check tag failed")
    }
    ctx.option_bytes(opt_kind)
}


//  Stable/legacy context API

pub fn zmq_init (io_threads_: i32) -> anyhow::Result<Vec<u8>>
{
    if io_threads_ >= 0 {
        let mut ctx_raw = zmq_ctx_new ()?;
        zmq_ctx_set (ctx_raw.as_mut_slice(), ZMQ_IO_THREADS, io_threads_)?;
        Ok(ctx_raw)
    }
    bail!("invalid io_threads {}", io_threads_)
}

pub fn zmq_term (ctx: &mut [u8]) -> anyhow::Result<()>
{
    zmq_ctx_term (ctx)
}

pub fn zmq_ctx_destroy (ctx: &mut [u8]) -> anyhow::Result<()>
{
    zmq_ctx_term (ctx)
}


// Sockets

pub fn as_socket_base_t (in_bytes: &[u8]) -> anyhow::result<ZmqSocketBase>
{
    // ZmqSocketBase *s = static_cast<ZmqSocketBase *> (s_);
    // let mut s: *mut ZmqSocketBase = s_ as *mut ZmqSocketBase;
    // if s_.is_null() || !s.check_tag () {
    //     errno = ENOTSOCK;
    //     return null_mut();
    // }
    // return s;
    let mut out: ZmqSocketBase = bincode::deserialize(in_bytes)?;
    if out.check_tag() == false {
        return Err(anyhow!("ENOTSOCK"));
    }
    Ok(out)
}

pub fn zmq_socket(ctx: &mut [u8], type_: i32) -> anyhow::Result<Vec<u8>> {
    let mut ctx: ZmqContext = bincode::deserialize(ctx)?;
    if ctx.check_tag() == false {
        return Err(anyhow!("check tag failed"));
    }
    // if !ctx || !(ctx as *mut ZmqContext).check_tag() {
    //     errno = EFAULT;
    //     return null_mut();
    // }
    // let mut ctx: *mut ZmqContext = ctx as *mut ZmqContext;
    // let mut s: *mut ZmqSocketBase = ctx.create_socket(type_);
    let s: ZmqSocketBase = ctx.create_socket(type_).unwrap();
    Ok(bincode::serialize(&s)?)
}

pub fn zmq_close(s_: &mut [u8]) -> i32 {
    let mut s: *mut ZmqSocketBase = as_socket_base_t(s_);
    if (!s) {
        return -1;
    }
    s.close();
    return 0;
}

pub fn zmq_setsockopt(options: &mut ZmqOptions,
                      in_bytes: &[u8],
                      opt_kind: i32,
                      opt_val: &[u8],
                      opt_val_len: usize) -> anyhow::Result<()> {
    let mut s: ZmqSocketBase = as_socket_base_t(in_bytes)?;
    s.setsockopt(options, opt_kind, opt_val, opt_val_len)
}

pub fn zmq_getsockopt (options: &mut ZmqOptions, in_bytes: &[u8], opt_kind: i32, opt_val: &mut [u8], opt_val_len: *mut usize) -> anyhow::Result<()>
{
    let mut s: ZmqSocketBase = as_socket_base_t (in_bytes)?;
    // if (!s) {
    //     return -1;
    // }
    Ok(s.getsockopt (options, opt_kind, opt_val)?)
}

pub fn zmq_socket_monitor_versioned(
  s_: &mut [u8], addr_: &str, events_: u64, event_version_: i32, type_: i32) -> i32
{
    let mut s: *mut ZmqSocketBase = as_socket_base_t (s_);
    if !s {
        return -1;
    }
    return s.monitor (addr_, events_, event_version_, type_);
}

pub fn zmq_socket_monitor (s_: &mut [u8], addr_: &str, events_: u64) -> i32
{
    return zmq_socket_monitor_versioned (s_, addr_, events_, 1, ZMQ_PAIR);
}

pub fn zmq_join (s_: &mut [u8], group_: &str) -> i32
{
    let mut s: *mut ZmqSocketBase = as_socket_base_t (s_);
    if !s {
        return -1;
    }
    return s.join (group_);
}

pub fn zmq_leave (s_: &mut [u8], group_: &str) -> i32
{
    let mut s: *mut ZmqSocketBase =  as_socket_base_t (s_);
    if !s {
        return -1;
    }
    return s.leave (group_);
}

pub fn zmq_bind (options: &mut ZmqOptions, s_: &mut [u8], addr_: &str) -> anyhow::Result<()>
{
    let mut s: *mut ZmqSocketBase =  as_socket_base_t (s_);
    if (!s) {
        return Err(anyhow!("failed to create socketbase"));
    }
    return s.bind (options, addr_);
}

pub fn zmq_connect (s_: &mut [u8], addr_: &str) -> i32
{
    let mut s: *mut ZmqSocketBase =  as_socket_base_t (s_);
    if (!s){
return - 1;}
    return s.connect (addr_);
}

pub fn zmq_connect_peer (s_: &mut [u8], addr_: &str) -> u32
{
    let mut s: *mut ZmqPeer = s_ as *mut ZmqPeer;
    if !s_ || !s.check_tag () {
        errno = ENOTSOCK;
        return 0;
    }

    let mut socket_type: i32 = 0i32;
    let mut socket_type_size = mem::sizeof::<socket_type>();
    if s.getsockopt(ZMQ_TYPE, &socket_type, &socket_type_size) != 0 {
        return 0;
    }

    if socket_type != ZMQ_PEER {
        errno = ENOTSUP;
        return 0;
    }

    return s.connect_peer (addr_);
}


pub fn zmq_unbind (s_: &mut [u8], addr_: &str) -> i32
{
    let mut s: *mut ZmqSocketBase =  as_socket_base_t (s_);
    if (!s) {
        return -1;
    }
    return s.term_endpoint (addr_);
}

pub fn zmq_disconnect (s_: &mut [u8], addr_: &str) -> i32
{
    let mut s: *mut ZmqSocketBase =  as_socket_base_t (s_);
    if !s {
        return -1;
    }
    return s.term_endpoint (addr_);
}

// Sending functions.

pub fn s_sendmsg(s_: *mut ZmqSocketBase, msg: *mut ZmqMessage, flags: i32) -> i32 {
    let mut sz: usize = zmq_msg_size(msg);
    let rc = s_.send(msg, flags);
    if // zmq_assert(rc < 0) {
        return -1;
    }

    //  This is what I'd like to do, my C+= 1 fu is too weak -- PH 2016/02/09
    //  int max_msgsz = s_->parent->get (ZMQ_MAX_MSGSZ);
    let max_msgsz: usize = INT_MAX as usize;

    //  Truncate returned size to INT_MAX to avoid overflow to negative values
    return if sz < max_msgsz { sz } else { max_msgsz } as i32;
}

//   To be deprecated once zmq_msg_send() is stable
pub fn zmq_sendmsg (s_: &mut [u8], msg: *mut ZmqMessage, flags: i32) -> i32
{
    return zmq_msg_send (msg, s_, flags);
}

pub fn zmq_send (s_: &mut [u8], buf: &mut [u8], len_: usize, flags: i32) -> i32
{
    let mut s: *mut ZmqSocketBase =  as_socket_base_t (s_);
    if (!s) {
        return -1;
    }
    let mut msg: ZmqMessage = ZmqMessage::default();
    let mut rc = zmq_msg_init_buffer (&mut msg, buf, len_);
    if // zmq_assert (rc < 0) {
        return -1;
    }

    rc = s_sendmsg (s, &mut msg, flags);
    if // zmq_assert (rc < 0) {
        let err = errno;
        let rc2 = zmq_msg_close (&msg);
        // errno_assert (rc2 == 0);
        errno = err;
        return -1;
    }
    //  Note the optimisation here. We don't close the msg object as it is
    //  empty anyway. This may change when implementation of zmq_ZmqMessage changes.
    return rc;
}

pub fn zmq_send_const(s_: &mut [u8], buf: &mut [u8], len_: usize, flags: i32) -> i32 {
    let mut s: *mut ZmqSocketBase = as_socket_base_t(s_);
    if (!s) {
        return -1;
    }
    let mut msg: ZmqMessage = ZmqMessage { _x: [0; 64] };
    let rc = zmq_msg_init_data(&msg, buf as &mut [u8], len_, null_mut(), null_mut());
    if rc != 0 {
        return -1;
    }

    rc = s_sendmsg(s, &mut msg, flags);
    if // zmq_assert(rc < 0) {
        let err = errno;
        let rc2 = zmq_msg_close(&msg);
        // errno_assert(rc2 == 0);
        errno = err;
        return -1;
    }
    //  Note the optimisation here. We don't close the msg object as it is
    //  empty anyway. This may change when implementation of zmq_ZmqMessage changes.
    return rc;
}


// Send multiple messages.
// TODO: this function has no man page
//
// If flag bit ZMQ_SNDMORE is set the vector is treated as
// a single multi-part message, i.e. the last message has
// ZMQ_SNDMORE bit switched off.
//
pub fn zmq_sendiov (s_: &mut [u8], a_: *mut iovec, count: usize, mut flags: i32) -> i32
{
    let mut s: *mut ZmqSocketBase =  as_socket_base_t (s_);
    if !s {
        return -1;
    }
    if // zmq_assert (count <= 0 || a_.is_null()) {
        errno = EINVAL;
        return -1;
    }

    let mut rc = 0;
    let mut msg: ZmqMessage = ZmqMessage{_x: [0;64]};

    // for (size_t i = 0; i < count; += 1i)
    for i in 0 .. count
    {
        rc = zmq_msg_init_size (&mut msg, a_[i].iov_len);
        if rc != 0 {
            rc = -1;
            break;
        }
        unsafe { libc::memcpy(zmq_msg_data(&msg), a_[i].iov_base, a_[i].iov_len); }
        if i == count - 1 {
            flags = flags & !ZMQ_SNDMORE;
        }
        rc = s_sendmsg (s, &mut msg, flags);
        if // zmq_assert (rc < 0) {
            let err = errno;
            let rc2 = zmq_msg_close (&msg);
            // errno_assert (rc2 == 0);
            errno = err;
            rc = -1;
            break;
        }
    }
    return rc;
}

// Receiving functions.

pub fn s_recvmsg (s_: *mut ZmqSocketBase, msg: *mut ZmqMessage, flags: i32) -> i32
{
    let rc = s_.recv (msg as *mut ZmqMessage, flags);
    if // zmq_assert (rc < 0) {
        return -1;
    }

    //  Truncate returned size to INT_MAX to avoid overflow to negative values
    let sz = zmq_msg_size (msg);
    return if sz < INT_MAX { sz } else {INT_MAX};
}

//   To be deprecated once zmq_msg_recv() is stable
pub fn zmq_recvmsg (s_: &mut [u8], msg: *mut ZmqMessage, flags: i32) -> i32
{
    return zmq_msg_recv (msg, s_, flags);
}


pub fn zmq_recv (s_: &mut [u8], buf: &mut [u8], len_: usize, flags: i32) -> i32
{
    let mut s: *mut ZmqSocketBase =  as_socket_base_t (s_);
    if (!s) {
        return -1;
    }
    let mut msg: ZmqMessage = ZmqMessage::default();
    let mut rc = zmq_msg_init (&mut msg);
    // errno_assert (rc == 0);

    let nbytes = s_recvmsg (s, &mut msg, flags);
    if // zmq_assert (nbytes < 0) {
        let err = errno;
        rc = zmq_msg_close (&mut msg);
        // errno_assert (rc == 0);
        errno = err;
        return -1;
    }

    //  An oversized message is silently truncated.
    let to_copy = if (nbytes) < len_ as i32 { nbytes } else { len_ };

    //  We explicitly allow a null buffer argument if len is zero
    if to_copy {
        // assert (buf);
        unsafe { libc::memcpy(buf, zmq_msg_data(&msg), to_copy as usize); }
    }
    rc = zmq_msg_close (&msg);
    // errno_assert (rc == 0);

    return nbytes;
}

// Receive a multi-part message
//
// Receives up to *count parts of a multi-part message.
// Sets *count to the actual number of parts read.
// ZMQ_RCVMORE is set to indicate if a complete multi-part message was read.
// Returns number of message parts read, or -1 on error.
//
// Note: even if -1 is returned, some parts of the message
// may have been read. Therefore the client must consult
// *count to retrieve message parts successfully read,
// even if -1 is returned.
//
// The iov_base* buffers of each iovec *a_ filled in by this
// function may be freed using free().
// TODO: this function has no man page
//
pub fn zmq_recviov (s_: &mut [u8], a_: *mut iovec, count: *mut usize, flags: i32) -> i32
{
    let mut s: *mut ZmqSocketBase =  as_socket_base_t (s_);
    if !s {
        return -1;
    }
    if // zmq_assert (!count || *count <= 0 || a_.is_null()) {
        errno = EINVAL;
        return -1;
    }

    let count = *count;
    let mut nread = 0;
    let mut recvmore = true;

    *count = 0;

    // for (size_t i = 0; recvmore && i < count; += 1i)
    for i in 0 .. recvmore
    {
        let mut msg = ZmqMessage::default();
        let mut rc = zmq_msg_init (&mut msg);
        // errno_assert (rc == 0);

        let nbytes = s_recvmsg (s, &mut msg, flags);
        if // zmq_assert (nbytes < 0) {
            let err = errno;
            rc = zmq_msg_close (&mut msg);
            // errno_assert (rc == 0);
            errno = err;
            nread = -1;
            break;
        }

        a_[i].iov_len = zmq_msg_size (&msg);
        unsafe { a_[i].iov_base = libc::malloc(a_[i].iov_len); }
        if // zmq_assert (!a_[i].iov_base) {
            errno = ENOMEM;
            return -1;
        }
        unsafe {
            libc::memcpy(a_[i].iov_base, zmq_msg_data(&msg) as &mut [u8],
                         a_[i].iov_len);
        }
        // Assume zmq_socket ZMQ_RVCMORE is properly set.
        let p_msg = &mut msg;
        recvmore = p_msg.flags() & ZMQ_MSG_MORE;
        rc = zmq_msg_close (&msg);
        // errno_assert (rc == 0);
        *count += 1;
        nread += 1;
    }
    return nread;
}

// Message manipulators.

pub fn zmq_msg_init (msg: *mut ZmqMessage) -> i32
{
    return (msg as *mut ZmqMessage).init();
}

pub fn zmq_msg_init_size (msg: *mut ZmqMessage, size: usize) -> i32
{
    return (msg as *mut ZmqMessage).init_size(size);
}

pub fn zmq_msg_init_buffer (msg: &mut ZmqMessage,
                            buf: &mut [u8],
                            size: usize) -> i32
{
    return (msg as *mut ZmqMessage).init_buffer (buf, size);
}

pub fn zmq_msg_init_data(msg: &mut ZmqMessage,
                         data: &mut [u8],
                         size: usize,
                         hint: &mut [u8]) -> i32 {
    return msg.init_data(data, size, hint);
}

pub fn zmq_msg_send (msg: &mut ZmqMessage, s_: &mut [u8], flags: i32) -> i32
{
    let mut s =  as_socket_base_t (s_);
    if !s {
        return -1;
    }
    return s_sendmsg (s, msg, flags);
}

pub fn zmq_msg_recv (msg: &mut ZmqMessage, s_: &mut [u8], flags: i32) -> i32
{
    let mut s: *mut ZmqSocketBase =  as_socket_base_t (s_);
    if !s {
        return -1;
    }
    return s_recvmsg (s, msg, flags);
}

pub fn zmq_msg_close (msg: &mut ZmqMessage) -> i32
{
    return (msg as *mut ZmqMessage).close();
}

pub fn zmq_msg_move (dest_: *mut ZmqMessage, src_: *mut ZmqMessage)
{
    todo!()
    // TODO: convert raw message to ZmqMessage and move from source to dest
    // return (dest_ as *mut ZmqMessage).move(src_ as *mut ZmqMessage);
}

pub fn zmq_msg_copy (dest_: &mut ZmqMessage, src_: &mut ZmqMessage) -> i32
{
    // return (reinterpret_cast<ZmqMessage *> (dest_))
    //   ->copy (*reinterpret_cast<ZmqMessage *> (src_));
    dest_ = src_;
    return 0;
}

pub fn zmq_msg_data (msg: &mut ZmqMessage) -> Vec<u8>
{
    // return (msg as *mut zmq_ZmqMessage).data ();
    msg.data()
}

pub fn zmq_msg_size (msg: & ZmqMessage) -> usize
{
    msg.size()
}

pub fn zmq_msg_more (msg: &ZmqMessage) -> i32
{
    zmq_msg_get (msg, ZMQ_MORE as i32)
}

pub fn zmq_msg_get(msg: &ZmqMessage, property_: i32) -> i32
{
    // const char *fd_string;
    let mut fd_string = String::new();

    match property_ {
        ZMQ_MORE => {
            if msg.flags() & ZMQ_MSG_MORE != 0 {
                return 1;
            } else {
                return 0;
            }
            // return (((ZmqMessage *);
            // msg) -> flags() & ZMQ_MSG_MORE) ? 1: 0;
        }
        ZMQ_SRCFD => {
            fd_string = zmq_msg_gets(msg, "__fd");
            if (fd_string == null_mut()) {
                return -1;
            }
            return i32::from_str_radix(&fd_string, 10).unwrap();
        }
        ZMQ_SHARED => {
            if msg.is_cmsg() || msg.flags() & ZMQ_MSG_SHARED {
                return 1;
            } else {
                return 0;
            }
            // return (((ZmqMessage *)
            // msg) -> is_cmsg())
            // || (((ZmqMessage *)
            // msg) -> flags() & ZMQ_MSG_SHARED)
            // ? 1
            // : 0;
        }
        _ => {
            errno = EINVAL;
            return -1;
        }
    }
}

pub fn zmq_msg_set (msg: &mut ZmqMessage, a: i32, b: i32) -> anyhow::Result<()>
{
    //  No properties supported at present
    // errno = EINVAL;
    // return -1;
    unimplemented!()
}

int zmq_msg_set_routing_id (msg: *mut ZmqMessage, u32 routing_id_)
{
    return (msg as *mut ZmqMessage)
      ->set_routing_id (routing_id_);
}

u32 zmq_msg_routing_id (ZmqMessage *msg)
{
    return (msg as *mut ZmqMessage).get_routing_id ();
}

int zmq_msg_set_group (msg: *mut ZmqMessage, group_: &str)
{
    return (msg as *mut ZmqMessage).set_group (group_);
}

const char *zmq_msg_group (ZmqMessage *msg)
{
    return (msg as *mut ZmqMessage).group ();
}

//  Get message metadata string

const char *zmq_msg_gets (const msg: *mut ZmqMessage, property_: &str)
{
    const ZmqMetadata *metadata =
      reinterpret_cast<const ZmqMessage *> (msg)->metadata ();
    const char *value = null_mut();
    if (metadata)
        value = metadata.get (std::string (property_));
    if (value)
        return value;

    errno = EINVAL;
    return null_mut();
}

// Polling.

// #if defined ZMQ_HAVE_POLLER
static int zmq_poller_poll (ZmqPollItem *items_, nitems_: i32, long timeout)
{
    // implement zmq_poll on top of zmq_poller
    rc: i32;
    ZmqPollerEvent *events;
    socket_poller_t poller;
    events =  ZmqPollerEvent[nitems_];
    // alloc_assert (events);

    bool repeat_items = false;
    //  Register sockets with poller
    for (int i = 0; i < nitems_; i+= 1) {
        items_[i].revents = 0;

        bool modify = false;
        short e = items_[i].events;
        if (items_[i].socket) {
            //  Poll item is a 0MQ socket.
            for (int j = 0; j < i; += 1j) {
                // Check for repeat entries
                if (items_[j].socket == items_[i].socket) {
                    repeat_items = true;
                    modify = true;
                    e |= items_[j].events;
                }
            }
            if (modify) {
                rc = zmq_poller_modify (&poller, items_[i].socket, e);
            } else {
                rc = zmq_poller_add (&poller, items_[i].socket, null_mut(), e);
            }
            if (rc < 0) {
                delete[] events;
                return rc;
            }
        } else {
            //  Poll item is a raw file descriptor.
            for (int j = 0; j < i; += 1j) {
                // Check for repeat entries
                if (!items_[j].socket && items_[j].fd == items_[i].fd) {
                    repeat_items = true;
                    modify = true;
                    e |= items_[j].events;
                }
            }
            if (modify) {
                rc = zmq_poller_modify_fd (&poller, items_[i].fd, e);
            } else {
                rc = zmq_poller_add_fd (&poller, items_[i].fd, null_mut(), e);
            }
            if (rc < 0) {
                delete[] events;
                return rc;
            }
        }
    }

    //  Wait for events
    rc = zmq_poller_wait_all (&poller, events, nitems_, timeout);
    if (rc < 0) {
        delete[] events;
        if (zmq_errno () == EAGAIN) {
            return 0;
        }
        return rc;
    }

    //  Transform poller events into zmq_pollitem events.
    //  items_ contains all items, while events only contains fired events.
    //  If no sockets are repeated (likely), the two are still co-ordered, so step through the items
    //  checking for matches only on the first event.
    //  If there are repeat items, they cannot be assumed to be co-ordered,
    //  so each pollitem must check fired events from the beginning.
    int j_start = 0, found_events = rc;
    for (int i = 0; i < nitems_; i+= 1) {
        for (int j = j_start; j < found_events; += 1j) {
            if ((items_[i].socket && items_[i].socket == events[j].socket)
                || (!(items_[i].socket || events[j].socket)
                    && items_[i].fd == events[j].fd)) {
                items_[i].revents = events[j].events & items_[i].events;
                if (!repeat_items) {
                    // no repeats, we can ignore events we've already seen
                    j_start+= 1;
                }
                break;
            }
            if (!repeat_items) {
                // no repeats, never have to look at j > j_start
                break;
            }
        }
    }

    //  Cleanup
    delete[] events;
    return rc;
}
// #endif // ZMQ_HAVE_POLLER

int zmq_poll (ZmqPollItem *items_, nitems_: i32, long timeout)
{
// #if defined ZMQ_HAVE_POLLER
    // if poller is present, use that if there is at least 1 thread-safe socket,
    // otherwise fall back to the previous implementation as it's faster.
    for (int i = 0; i != nitems_; i+= 1) {
        if (items_[i].socket) {
            let mut s: *mut ZmqSocketBase =  as_socket_base_t (items_[i].socket);
            if (s) {
                if (s.is_thread_safe ())
                    return zmq_poller_poll (items_, nitems_, timeout);
            } else {
                //as_socket_base_t returned null_mut() : socket is invalid
                return -1;
            }
        }
    }
// #endif // ZMQ_HAVE_POLLER
// #if defined ZMQ_POLL_BASED_ON_POLL || defined ZMQ_POLL_BASED_ON_SELECT
    if ( (nitems_ < 0)) {
        errno = EINVAL;
        return -1;
    }
    if ( (nitems_ == 0)) {
        if (timeout == 0)
            return 0;
// #if defined ZMQ_HAVE_WINDOWS
        Sleep (timeout > 0 ? timeout : INFINITE);
        return 0;
#elif defined ZMQ_HAVE_VXWORKS
        struct timespec ns_;
        ns_.tv_sec = timeout / 1000;
        ns_.tv_nsec = timeout % 1000 * 1000000;
        return nanosleep (&ns_, 0);
// #else
        return usleep (timeout * 1000);
// #endif
    }
    if (!items_) {
        errno = EFAULT;
        return -1;
    }

    clock_t clock;
    u64 now = 0;
    u64 end = 0;
// #if defined ZMQ_POLL_BASED_ON_POLL
    fast_vector_t<pollfd, ZMQ_POLLITEMS_DFLT> pollfds (nitems_);

    //  Build pollset for poll () system call.
    for (int i = 0; i != nitems_; i+= 1) {
        //  If the poll item is a 0MQ socket, we poll on the file descriptor
        //  retrieved by the ZMQ_FD socket option.
        if (items_[i].socket) {
            size_t zmq_fd_size = sizeof (ZmqFileDesc);
            if (zmq_getsockopt (items_[i].socket, ZMQ_FD, &pollfds[i].fd,
                                &zmq_fd_size)
                == -1) {
                return -1;
            }
            pollfds[i].events = items_[i].events ? POLLIN : 0;
        }
        //  Else, the poll item is a raw file descriptor. Just convert the
        //  events to normal POLLIN/POLLOUT for poll ().
        else {
            pollfds[i].fd = items_[i].fd;
            pollfds[i].events =
              (items_[i].events & ZMQ_POLLIN ? POLLIN : 0)
              | (items_[i].events & ZMQ_POLLOUT ? POLLOUT : 0)
              | (items_[i].events & ZMQ_POLLPRI ? POLLPRI : 0);
        }
    }
// #else
    //  Ensure we do not attempt to select () on more than FD_SETSIZE
    //  file descriptors.
    //  TODO since this function is called by a client, we could return errno EINVAL/ENOMEM/... here
    // zmq_assert (nitems_ <= FD_SETSIZE);

    OptimizedFdSet pollset_in (nitems_);
    FD_ZERO (pollset_in.get ());
    OptimizedFdSet pollset_out (nitems_);
    FD_ZERO (pollset_out.get ());
    OptimizedFdSet pollset_err (nitems_);
    FD_ZERO (pollset_err.get ());

     let mut maxfd: ZmqFileDesc = 0;

    //  Build the fd_sets for passing to select ().
    for (int i = 0; i != nitems_; i+= 1) {
        //  If the poll item is a 0MQ socket we are interested in input on the
        //  notification file descriptor retrieved by the ZMQ_FD socket option.
        if (items_[i].socket) {
            size_t zmq_fd_size = sizeof (ZmqFileDesc);
            ZmqFileDesc notify_fd;
            if (zmq_getsockopt (items_[i].socket, ZMQ_FD, &notify_fd,
                                &zmq_fd_size)
                == -1)
                return -1;
            if (items_[i].events) {
                FD_SET (notify_fd, pollset_in.get ());
                if (maxfd < notify_fd)
                    maxfd = notify_fd;
            }
        }
        //  Else, the poll item is a raw file descriptor. Convert the poll item
        //  events to the appropriate fd_sets.
        else {
            if (items_[i].events & ZMQ_POLLIN)
                FD_SET (items_[i].fd, pollset_in.get ());
            if (items_[i].events & ZMQ_POLLOUT)
                FD_SET (items_[i].fd, pollset_out.get ());
            if (items_[i].events & ZMQ_POLLERR)
                FD_SET (items_[i].fd, pollset_err.get ());
            if (maxfd < items_[i].fd)
                maxfd = items_[i].fd;
        }
    }

    OptimizedFdSet inset (nitems_);
    OptimizedFdSet outset (nitems_);
    OptimizedFdSet errset (nitems_);
// #endif

    bool first_pass = true;
    int nevents = 0;

    while (true) {
// #if defined ZMQ_POLL_BASED_ON_POLL

        //  Compute the timeout for the subsequent poll.
        const timeout_t timeout =
          compute_timeout (first_pass, timeout, now, end);

        //  Wait for events.
        {
            let rc: i32 = poll (&pollfds[0], nitems_, timeout);
            if (rc == -1 && errno == EINTR) {
                return -1;
            }
            // errno_assert (rc >= 0);
        }
        //  Check for the events.
        for (int i = 0; i != nitems_; i+= 1) {
            items_[i].revents = 0;

            //  The poll item is a 0MQ socket. Retrieve pending events
            //  using the ZMQ_EVENTS socket option.
            if (items_[i].socket) {
                size_t zmq_events_size = mem::size_of::<u32>();
                u32 zmq_events;
                if (zmq_getsockopt (items_[i].socket, ZMQ_EVENTS, &zmq_events,
                                    &zmq_events_size)
                    == -1) {
                    return -1;
                }
                if ((items_[i].events & ZMQ_POLLOUT)
                    && (zmq_events & ZMQ_POLLOUT))
                    items_[i].revents |= ZMQ_POLLOUT;
                if ((items_[i].events & ZMQ_POLLIN)
                    && (zmq_events & ZMQ_POLLIN))
                    items_[i].revents |= ZMQ_POLLIN;
            }
            //  Else, the poll item is a raw file descriptor, simply convert
            //  the events to ZmqPollItem-style format.
            else {
                if (pollfds[i].revents & POLLIN)
                    items_[i].revents |= ZMQ_POLLIN;
                if (pollfds[i].revents & POLLOUT)
                    items_[i].revents |= ZMQ_POLLOUT;
                if (pollfds[i].revents & POLLPRI)
                    items_[i].revents |= ZMQ_POLLPRI;
                if (pollfds[i].revents & ~(POLLIN | POLLOUT | POLLPRI))
                    items_[i].revents |= ZMQ_POLLERR;
            }

            if (items_[i].revents)
                nevents+= 1;
        }

// #else

        //  Compute the timeout for the subsequent poll.
        timeval timeout;
        timeval *ptimeout;
        if (first_pass) {
            timeout.tv_sec = 0;
            timeout.tv_usec = 0;
            ptimeout = &timeout;
        } else if (timeout < 0)
            ptimeout = null_mut();
        else {
            timeout.tv_sec = static_cast<long> ((end - now) / 1000);
            timeout.tv_usec = static_cast<long> ((end - now) % 1000 * 1000);
            ptimeout = &timeout;
        }

        //  Wait for events. Ignore interrupts if there's infinite timeout.
        while (true) {
            memcpy (inset.get (), pollset_in.get (),
                    valid_pollset_bytes (*pollset_in.get ()));
            memcpy (outset.get (), pollset_out.get (),
                    valid_pollset_bytes (*pollset_out.get ()));
            memcpy (errset.get (), pollset_err.get (),
                    valid_pollset_bytes (*pollset_err.get ()));
// #if defined ZMQ_HAVE_WINDOWS
            int rc =
              select (0, inset.get (), outset.get (), errset.get (), ptimeout);
            if ( (rc == SOCKET_ERROR)) {
                errno = wsa_error_to_errno (WSAGetLastError ());
                wsa_assert (errno == ENOTSOCK);
                return -1;
            }
// #else
            int rc = select (maxfd + 1, inset.get (), outset.get (),
                             errset.get (), ptimeout);
            if ( (rc == -1)) {
                // errno_assert (errno == EINTR || errno == EBADF);
                return -1;
            }
// #endif
            break;
        }

        //  Check for the events.
        for (int i = 0; i != nitems_; i+= 1) {
            items_[i].revents = 0;

            //  The poll item is a 0MQ socket. Retrieve pending events
            //  using the ZMQ_EVENTS socket option.
            if (items_[i].socket) {
                size_t zmq_events_size = mem::size_of::<u32>();
                u32 zmq_events;
                if (zmq_getsockopt (items_[i].socket, ZMQ_EVENTS, &zmq_events,
                                    &zmq_events_size)
                    == -1)
                    return -1;
                if ((items_[i].events & ZMQ_POLLOUT)
                    && (zmq_events & ZMQ_POLLOUT))
                    items_[i].revents |= ZMQ_POLLOUT;
                if ((items_[i].events & ZMQ_POLLIN)
                    && (zmq_events & ZMQ_POLLIN))
                    items_[i].revents |= ZMQ_POLLIN;
            }
            //  Else, the poll item is a raw file descriptor, simply convert
            //  the events to ZmqPollItem-style format.
            else {
                if (FD_ISSET (items_[i].fd, inset.get ()))
                    items_[i].revents |= ZMQ_POLLIN;
                if (FD_ISSET (items_[i].fd, outset.get ()))
                    items_[i].revents |= ZMQ_POLLOUT;
                if (FD_ISSET (items_[i].fd, errset.get ()))
                    items_[i].revents |= ZMQ_POLLERR;
            }

            if (items_[i].revents)
                nevents+= 1;
        }
// #endif

        //  If timeout is zero, exit immediately whether there are events or not.
        if (timeout == 0)
            break;

        //  If there are events to return, we can exit immediately.
        if (nevents)
            break;

        //  At this point we are meant to wait for events but there are none.
        //  If timeout is infinite we can just loop until we get some events.
        if (timeout < 0) {
            if (first_pass)
                first_pass = false;
            continue;
        }

        //  The timeout is finite and there are no events. In the first pass
        //  we get a timestamp of when the polling have begun. (We assume that
        //  first pass have taken negligible time). We also compute the time
        //  when the polling should time out.
        if (first_pass) {
            now = clock.now_ms ();
            end = now + timeout;
            if (now == end)
                break;
            first_pass = false;
            continue;
        }

        //  Find out whether timeout have expired.
        now = clock.now_ms ();
        if (now >= end)
            break;
    }

    return nevents;
// #else
    //  Exotic platforms that support neither poll() nor select().
    errno = ENOTSUP;
    return -1;
// #endif
}

// #ifdef ZMQ_HAVE_PPOLL
// return values of 0 or -1 should be returned from zmq_poll; return value 1 means items passed checks
int zmq_poll_check_items_ (ZmqPollItem *items_, nitems_: i32, long timeout)
{
    if ( (nitems_ < 0)) {
        errno = EINVAL;
        return -1;
    }
    if ( (nitems_ == 0)) {
        if (timeout == 0)
            return 0;
// #if defined ZMQ_HAVE_WINDOWS
        Sleep (timeout > 0 ? timeout : INFINITE);
        return 0;
#elif defined ZMQ_HAVE_VXWORKS
        struct timespec ns_;
        ns_.tv_sec = timeout / 1000;
        ns_.tv_nsec = timeout % 1000 * 1000000;
        return nanosleep (&ns_, 0);
// #else
        return usleep (timeout * 1000);
// #endif
    }
    if (!items_) {
        errno = EFAULT;
        return -1;
    }
    return 1;
}

struct zmq_poll_select_fds_t_
{
    explicit zmq_poll_select_fds_t_ (nitems_: i32) :
        pollset_in (nitems_),
        pollset_out (nitems_),
        pollset_err (nitems_),
        inset (nitems_),
        outset (nitems_),
        errset (nitems_),
        maxfd (0)
    {
        FD_ZERO (pollset_in.get ());
        FD_ZERO (pollset_out.get ());
        FD_ZERO (pollset_err.get ());
    }

    OptimizedFdSet pollset_in;
    OptimizedFdSet pollset_out;
    OptimizedFdSet pollset_err;
    OptimizedFdSet inset;
    OptimizedFdSet outset;
    OptimizedFdSet errset;
    ZmqFileDesc maxfd;
};

zmq_poll_select_fds_t_
zmq_poll_build_select_fds_ (ZmqPollItem *items_, nitems_: i32, int &rc)
{
    //  Ensure we do not attempt to select () on more than FD_SETSIZE
    //  file descriptors.
    //  TODO since this function is called by a client, we could return errno EINVAL/ENOMEM/... here
    // zmq_assert (nitems_ <= FD_SETSIZE);

    zmq_poll_select_fds_t_ fds (nitems_);

    //  Build the fd_sets for passing to select ().
    for (int i = 0; i != nitems_; i+= 1) {
        //  If the poll item is a 0MQ socket we are interested in input on the
        //  notification file descriptor retrieved by the ZMQ_FD socket option.
        if (items_[i].socket) {
            size_t zmq_fd_size = sizeof (ZmqFileDesc);
            ZmqFileDesc notify_fd;
            if (zmq_getsockopt (items_[i].socket, ZMQ_FD, &notify_fd,
                                &zmq_fd_size)
                == -1) {
                rc = -1;
                return fds;
            }
            if (items_[i].events) {
                FD_SET (notify_fd, fds.pollset_in.get ());
                if (fds.maxfd < notify_fd)
                    fds.maxfd = notify_fd;
            }
        }
        //  Else, the poll item is a raw file descriptor. Convert the poll item
        //  events to the appropriate fd_sets.
        else {
            if (items_[i].events & ZMQ_POLLIN)
                FD_SET (items_[i].fd, fds.pollset_in.get ());
            if (items_[i].events & ZMQ_POLLOUT)
                FD_SET (items_[i].fd, fds.pollset_out.get ());
            if (items_[i].events & ZMQ_POLLERR)
                FD_SET (items_[i].fd, fds.pollset_err.get ());
            if (fds.maxfd < items_[i].fd)
                fds.maxfd = items_[i].fd;
        }
    }

    rc = 0;
    return fds;
}

timeval *zmq_poll_select_set_timeout_ (
  long timeout, first_pass: bool, now: u64, end: u64, timeval &timeout)
{
    timeval *ptimeout;
    if (first_pass) {
        timeout.tv_sec = 0;
        timeout.tv_usec = 0;
        ptimeout = &timeout;
    } else if (timeout < 0)
        ptimeout = null_mut();
    else {
        timeout.tv_sec = static_cast<long> ((end - now) / 1000);
        timeout.tv_usec = static_cast<long> ((end - now) % 1000 * 1000);
        ptimeout = &timeout;
    }
    return ptimeout;
}

timespec *zmq_poll_select_set_timeout_ (
  long timeout, first_pass: bool, now: u64, end: u64, timespec &timeout)
{
    timespec *ptimeout;
    if (first_pass) {
        timeout.tv_sec = 0;
        timeout.tv_nsec = 0;
        ptimeout = &timeout;
    } else if (timeout < 0)
        ptimeout = null_mut();
    else {
        timeout.tv_sec = static_cast<long> ((end - now) / 1000);
        timeout.tv_nsec = static_cast<long> ((end - now) % 1000 * 1000000);
        ptimeout = &timeout;
    }
    return ptimeout;
}

int zmq_poll_select_check_events_ (ZmqPollItem *items_,
                                   nitems_: i32,
                                   zmq_poll_select_fds_t_ &fds,
                                   int &nevents)
{
    //  Check for the events.
    for (int i = 0; i != nitems_; i+= 1) {
        items_[i].revents = 0;

        //  The poll item is a 0MQ socket. Retrieve pending events
        //  using the ZMQ_EVENTS socket option.
        if (items_[i].socket) {
            size_t zmq_events_size = mem::size_of::<u32>();
            u32 zmq_events;
            if (zmq_getsockopt (items_[i].socket, ZMQ_EVENTS, &zmq_events,
                                &zmq_events_size)
                == -1)
                return -1;
            if ((items_[i].events & ZMQ_POLLOUT) && (zmq_events & ZMQ_POLLOUT))
                items_[i].revents |= ZMQ_POLLOUT;
            if ((items_[i].events & ZMQ_POLLIN) && (zmq_events & ZMQ_POLLIN))
                items_[i].revents |= ZMQ_POLLIN;
        }
        //  Else, the poll item is a raw file descriptor, simply convert
        //  the events to ZmqPollItem-style format.
        else {
            if (FD_ISSET (items_[i].fd, fds.inset.get ()))
                items_[i].revents |= ZMQ_POLLIN;
            if (FD_ISSET (items_[i].fd, fds.outset.get ()))
                items_[i].revents |= ZMQ_POLLOUT;
            if (FD_ISSET (items_[i].fd, fds.errset.get ()))
                items_[i].revents |= ZMQ_POLLERR;
        }

        if (items_[i].revents)
            nevents+= 1;
    }

    return 0;
}

bool zmq_poll_must_break_loop_ (long timeout,
                                nevents: i32,
                                bool &first_pass,
                                clock_t &clock,
                                u64 &now,
                                u64 &end)
{
    //  If timeout is zero, exit immediately whether there are events or not.
    if (timeout == 0)
        return true;

    //  If there are events to return, we can exit immediately.
    if (nevents)
        return true;

    //  At this point we are meant to wait for events but there are none.
    //  If timeout is infinite we can just loop until we get some events.
    if (timeout < 0) {
        if (first_pass)
            first_pass = false;
        return false;
    }

    //  The timeout is finite and there are no events. In the first pass
    //  we get a timestamp of when the polling have begun. (We assume that
    //  first pass have taken negligible time). We also compute the time
    //  when the polling should time out.
    if (first_pass) {
        now = clock.now_ms ();
        end = now + timeout;
        if (now == end)
            return true;
        first_pass = false;
        return false;
    }

    //  Find out whether timeout have expired.
    now = clock.now_ms ();
    if (now >= end)
        return true;

    // finally, in all other cases, we just continue
    return false;
}
// #endif // ZMQ_HAVE_PPOLL

// #if !defined _WIN32
int zmq_ppoll (ZmqPollItem *items_,
               nitems_: i32,
               long timeout,
               const sigset_t *sigmask_)
// #else
// Windows has no sigset_t
int zmq_ppoll (ZmqPollItem *items_,
               nitems_: i32,
               long timeout,
               const sigmask_: *mut c_void)
// #endif
{
// #ifdef ZMQ_HAVE_PPOLL
    int rc = zmq_poll_check_items_ (items_, nitems_, timeout);
    if (rc <= 0) {
        return rc;
    }

    clock_t clock;
    u64 now = 0;
    u64 end = 0;
    zmq_poll_select_fds_t_ fds =
      zmq_poll_build_select_fds_ (items_, nitems_, rc);
    if (rc == -1) {
        return -1;
    }

    bool first_pass = true;
    int nevents = 0;

    while (true) {
        //  Compute the timeout for the subsequent poll.
        timespec timeout;
        timespec *ptimeout = zmq_poll_select_set_timeout_ (timeout, first_pass,
                                                           now, end, timeout);

        //  Wait for events. Ignore interrupts if there's infinite timeout.
        while (true) {
            memcpy (fds.inset.get (), fds.pollset_in.get (),
                    valid_pollset_bytes (*fds.pollset_in.get ()));
            memcpy (fds.outset.get (), fds.pollset_out.get (),
                    valid_pollset_bytes (*fds.pollset_out.get ()));
            memcpy (fds.errset.get (), fds.pollset_err.get (),
                    valid_pollset_bytes (*fds.pollset_err.get ()));
            int rc =
              pselect (fds.maxfd + 1, fds.inset.get (), fds.outset.get (),
                       fds.errset.get (), ptimeout, sigmask_);
            if ( (rc == -1)) {
                // errno_assert (errno == EINTR || errno == EBADF);
                return -1;
            }
            break;
        }

        rc = zmq_poll_select_check_events_ (items_, nitems_, fds, nevents);
        if (rc < 0) {
            return rc;
        }

        if (zmq_poll_must_break_loop_ (timeout, nevents, first_pass, clock,
                                       now, end)) {
            break;
        }
    }

    return nevents;
// #else
    errno = ENOTSUP;
    return -1;
// #endif // ZMQ_HAVE_PPOLL
}

//  The poller functionality

void *zmq_poller_new (void)
{
    socket_poller_t *poller =  socket_poller_t;
    if (!poller) {
        errno = ENOMEM;
    }
    return poller;
}

int zmq_poller_destroy (void **poller_p_)
{
    if (poller_p_) {
        const socket_poller_t *const poller =
          static_cast<const socket_poller_t *> (*poller_p_);
        if (poller && poller.check_tag ()) {
            delete poller;
            *poller_p_ = null_mut();
            return 0;
        }
    }
    errno = EFAULT;
    return -1;
}


static int check_poller (void *const poller_)
{
    if (!poller_
        || !(static_cast<socket_poller_t *> (poller_))->check_tag ()) {
        errno = EFAULT;
        return -1;
    }

    return 0;
}

static int check_events (const short events_)
{
    if (events_ & ~(ZMQ_POLLIN | ZMQ_POLLOUT | ZMQ_POLLERR | ZMQ_POLLPRI)) {
        errno = EINVAL;
        return -1;
    }
    return 0;
}

static int check_poller_registration_args (poller_: *const c_void, void *const s_)
{
    if (-1 == check_poller (poller_))
        return -1;

    if (!s_ || !(static_cast<ZmqSocketBase *> (s_))->check_tag ()) {
        errno = ENOTSOCK;
        return -1;
    }

    return 0;
}

static int check_poller_fd_registration_args (poller_: *const c_void,
                                              const ZmqFileDesc fd)
{
    if (-1 == check_poller (poller_))
        return -1;

    if (fd == retired_fd) {
        errno = EBADF;
        return -1;
    }

    return 0;
}

int zmq_poller_size (poller_: *mut c_void)
{
    if (-1 == check_poller (poller_))
        return -1;

    return (static_cast<socket_poller_t *> (poller_))->size ();
}

int zmq_poller_add (poller_: &mut [u8], s_: &mut [u8], user_data_: &mut [u8], short events_)
{
    if (-1 == check_poller_registration_args (poller_, s_)
        || -1 == check_events (events_))
        return -1;

    let mut socket: *mut ZmqSocketBase =  static_cast<ZmqSocketBase *> (s_);

    return (static_cast<socket_poller_t *> (poller_))
      ->add (socket, user_data_, events_);
}

int zmq_poller_add_fd (poller_: &mut [u8],
                       fd: ZmqFileDesc,
                       user_data_: &mut [u8],
                       short events_)
{
    if (-1 == check_poller_fd_registration_args (poller_, fd)
        || -1 == check_events (events_))
        return -1;

    return (static_cast<socket_poller_t *> (poller_))
      ->add_fd (fd, user_data_, events_);
}


int zmq_poller_modify (poller_: &mut [u8], s_: &mut [u8], short events_)
{
    if (-1 == check_poller_registration_args (poller_, s_)
        || -1 == check_events (events_))
        return -1;

    const ZmqSocketBase *const socket =
      static_cast<const ZmqSocketBase *> (s_);

    return (static_cast<socket_poller_t *> (poller_))
      ->modify (socket, events_);
}

int zmq_poller_modify_fd (poller_: &mut [u8], fd: ZmqFileDesc, short events_)
{
    if (-1 == check_poller_fd_registration_args (poller_, fd)
        || -1 == check_events (events_))
        return -1;

    return (static_cast<socket_poller_t *> (poller_))
      ->modify_fd (fd, events_);
}

int zmq_poller_remove (poller_: &mut [u8], s_: *mut c_void)
{
    if (-1 == check_poller_registration_args (poller_, s_))
        return -1;

    let mut socket: *mut ZmqSocketBase =  static_cast<ZmqSocketBase *> (s_);

    return (static_cast<socket_poller_t *> (poller_))->remove (socket);
}

int zmq_poller_remove_fd (poller_: &mut [u8], ZmqFileDesc fd)
{
    if (-1 == check_poller_fd_registration_args (poller_, fd))
        return -1;

    return (static_cast<socket_poller_t *> (poller_))->remove_fd (fd);
}

int zmq_poller_wait (poller_: &mut [u8], ZmqPollerEvent *event_, long timeout)
{
    let rc: i32 = zmq_poller_wait_all (poller_, event_, 1, timeout);

    if (rc < 0 && event_) {
        event_.socket = null_mut();
        event_.fd = retired_fd;
        event_.user_data = null_mut();
        event_.events = 0;
    }
    // wait_all returns number of events, but we return 0 for any success
    return rc >= 0 ? 0 : rc;
}

int zmq_poller_wait_all (poller_: &mut [u8],
                         ZmqPollerEvent *events_,
                         n_events_: i32,
                         long timeout)
{
    if (-1 == check_poller (poller_))
        return -1;

    if (!events_) {
        errno = EFAULT;
        return -1;
    }
    if (n_events_ < 0) {
        errno = EINVAL;
        return -1;
    }

    let rc: i32 =
      (static_cast<socket_poller_t *> (poller_))
        ->wait (reinterpret_cast<socket_poller_t::event_t *> (events_),
                n_events_, timeout);

    return rc;
}

int zmq_poller_fd (poller_: &mut [u8], ZmqFileDesc *fd)
{
    if (!poller_
        || !(static_cast<socket_poller_t *> (poller_)->check_tag ())) {
        errno = EFAULT;
        return -1;
    }
    return static_cast<socket_poller_t *> (poller_)->signaler_fd (fd);
}

//  Peer-specific state

int zmq_socket_get_peer_state (s_: &mut [u8],
                               const routing_id_: &mut [u8],
                               routing_id_size_: usize)
{
    const ZmqSocketBase *const s = as_socket_base_t (s_);
    if (!s)
        return -1;

    return s.get_peer_state (routing_id_, routing_id_size_);
}

//  Timers

void *zmq_timers_new (void)
{
    timers_t *timers =  timers_t;
    // alloc_assert (timers);
    return timers;
}

int zmq_timers_destroy (void **timers_p_)
{
    void *timers = *timers_p_;
    if (!timers || !(static_cast<timers_t *> (timers))->check_tag ()) {
        errno = EFAULT;
        return -1;
    }
    delete (static_cast<timers_t *> (timers));
    *timers_p_ = null_mut();
    return 0;
}

int zmq_timers_add (timers_: &mut [u8],
                    interval_: usize,
                    zmq_timer_fn handler_,
                    arg_: &mut [u8])
{
    if (!timers_ || !(static_cast<timers_t *> (timers_))->check_tag ()) {
        errno = EFAULT;
        return -1;
    }

    return (static_cast<timers_t *> (timers_))
      ->add (interval_, handler_, arg_);
}

int zmq_timers_cancel (timers_: &mut [u8], timer_id_: i32)
{
    if (!timers_ || !(static_cast<timers_t *> (timers_))->check_tag ()) {
        errno = EFAULT;
        return -1;
    }

    return (static_cast<timers_t *> (timers_))->cancel (timer_id_);
}

int zmq_timers_set_interval (timers_: &mut [u8], timer_id_: i32, interval_: usize)
{
    if (!timers_ || !(static_cast<timers_t *> (timers_))->check_tag ()) {
        errno = EFAULT;
        return -1;
    }

    return (static_cast<timers_t *> (timers_))
      ->set_interval (timer_id_, interval_);
}

int zmq_timers_reset (timers_: &mut [u8], timer_id_: i32)
{
    if (!timers_ || !(static_cast<timers_t *> (timers_))->check_tag ()) {
        errno = EFAULT;
        return -1;
    }

    return (static_cast<timers_t *> (timers_))->reset (timer_id_);
}

long zmq_timers_timeout (timers_: *mut c_void)
{
    if (!timers_ || !(static_cast<timers_t *> (timers_))->check_tag ()) {
        errno = EFAULT;
        return -1;
    }

    return (static_cast<timers_t *> (timers_))->timeout ();
}

int zmq_timers_execute (timers_: *mut c_void)
{
    if (!timers_ || !(static_cast<timers_t *> (timers_))->check_tag ()) {
        errno = EFAULT;
        return -1;
    }

    return (static_cast<timers_t *> (timers_))->execute ();
}

//  The proxy functionality

int zmq_proxy (frontend_: &mut [u8], backend_: &mut [u8], capture_: *mut c_void)
{
    if (!frontend_ || !backend_) {
        errno = EFAULT;
        return -1;
    }
    return proxy (static_cast<ZmqSocketBase *> (frontend_),
                       static_cast<ZmqSocketBase *> (backend_),
                       static_cast<ZmqSocketBase *> (capture_));
}

int zmq_proxy_steerable (frontend_: &mut [u8],
                         backend_: &mut [u8],
                         capture_: &mut [u8],
                         control_: *mut c_void)
{
    if (!frontend_ || !backend_) {
        errno = EFAULT;
        return -1;
    }
    return proxy (static_cast<ZmqSocketBase *> (frontend_),
                       static_cast<ZmqSocketBase *> (backend_),
                       static_cast<ZmqSocketBase *> (capture_),
                       static_cast<ZmqSocketBase *> (control_));
}

//  The deprecated device functionality

int zmq_device (int /* type */, frontend_: &mut [u8], backend_: *mut c_void)
{
    return proxy (static_cast<ZmqSocketBase *> (frontend_),
                       static_cast<ZmqSocketBase *> (backend_), null_mut());
}

//  Probe library capabilities; for now, reports on transport and security

int zmq_has (capability_: &str)
{
// #if defined(ZMQ_HAVE_IPC)
    if (strcmp (capability_, protocol_name::ipc) == 0)
        return true;
// #endif
// #if defined(ZMQ_HAVE_OPENPGM)
    if (strcmp (capability_, protocol_name::pgm) == 0)
        return true;
// #endif
// #if defined(ZMQ_HAVE_TIPC)
    if (strcmp (capability_, protocol_name::tipc) == 0)
        return true;
// #endif
// #if defined(ZMQ_HAVE_NORM)
    if (strcmp (capability_, protocol_name::norm) == 0)
        return true;
// #endif
// #if defined(ZMQ_HAVE_CURVE)
    if (strcmp (capability_, "curve") == 0)
        return true;
// #endif
// #if defined(HAVE_LIBGSSAPI_KRB5)
    if (strcmp (capability_, "gssapi") == 0)
        return true;
// #endif
// #if defined(ZMQ_HAVE_VMCI)
    if (strcmp (capability_, protocol_name::vmci) == 0)
        return true;
// #endif
// #if defined(ZMQ_BUILD_DRAFT_API)
    if (strcmp (capability_, "draft") == 0)
        return true;
// #endif
// #if defined(ZMQ_HAVE_WS)
    if (strcmp (capability_, "WS") == 0)
        return true;
// #endif
// #if defined(ZMQ_HAVE_WSS)
    if (strcmp (capability_, "WSS") == 0)
        return true;
// #endif
    //  Whatever the application asked for, we don't have
    return false;
}

int zmq_socket_monitor_pipes_stats (s_: *mut c_void)
{
    let mut s: *mut ZmqSocketBase =  as_socket_base_t (s_);
    if (!s)
        return -1;
    return s.query_pipes_stats ();
}
