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

use std::intrinsics::unlikely;
use std::mem;
use std::ptr::null_mut;
use libc::{c_char, c_void, EFAULT, EINTR, EINVAL, ENOMEM, ENOTSOCK, ENOTSUP, INT_MAX};
use crate::ctx_hdr::ZmqContext;
use crate::zmq_hdr::{zmq_free_fn, ZMQ_IO_THREADS, zmq_msg_t, ZMQ_PAIR, ZMQ_PEER, ZMQ_SNDMORE, ZMQ_TYPE, ZMQ_VERSION_MAJOR, ZMQ_VERSION_MINOR, ZMQ_VERSION_PATCH};

// XSI vector I/O
// #if defined ZMQ_HAVE_UIO
// #include <sys/uio.h>
// #else
#[derive(Default,Debug,Clone)]
pub struct iovec
{
    iov_base: *mut c_void,
    iov_len: usize,
}
// #endif

// #include <string.h>
// #include <stdlib.h>
// #include <new>
// #include <climits>

// #include "proxy.hpp"
// #include "socket_base.hpp"
// #include "stdint.hpp"
// #include "config.hpp"
// #include "likely.hpp"
// #include "clock.hpp"
// #include "ctx.hpp"
// #include "err.hpp"
// #include "msg.hpp"
// #include "fd.hpp"
// #include "metadata.hpp"
// #include "socket_poller.hpp"
// #include "timers.hpp"
// #include "ip.hpp"
// #include "address.hpp"

// #ifdef ZMQ_HAVE_PPOLL
// #include "polling_util.hpp"
// #include <sys/select.h>
// #endif

// #if defined ZMQ_HAVE_OPENPGM
// #define __PGM_WININT_H__
// #include <pgm/pgm.h>
// #endif

//  Compile time check whether msg_t fits into zmq_msg_t.
// typedef char
//   check_msg_t_size[sizeof (zmq::msg_t) == mem::size_of::<zmq_msg_t>() ? 1 : -1];


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
pub fn zmq_ctx_new () -> *mut c_void
{
    //  We do this before the ctx constructor since its embedded mailbox_t
    //  object needs the network to be up and running (at least on Windows).
    if !initialize_network () {
        return null_mut();
    }

    //  Create 0MQ context.
    let mut ctx: *mut ZmqContext = ZmqContext::new();
    if ctx {
        if !ctx.valid () {
            // delete ctx;
            return null_mut();
        }
    }
    return ctx as *mut c_void;
}

pub fn zmq_ctx_term (ctx_: *mut c_void) -> i32
{
    if ctx_.is_null() == false || !(ctx_ as *mut ZmqContext).check_tag() {
        errno = EFAULT;
        return -1;
    }

    let rc = (ctx_ as *mut ZmqContext).terminate();
    let en = errno;

    //  Shut down only if termination was not interrupted by a signal.
    if !rc || en != EINTR {
        shutdown_network();
    }

    errno = en;
    return rc;
}

pub fn zmq_ctx_shutdown (ctx_: *mut c_void) -> i32
{
    if (!ctx_ || !(ctx_ as *mut ZmqContext).check_tag ()) {
        errno = EFAULT;
        return -1;
    }
    return (ctx_ as *mut ZmqContext).shutdown ();
}

pub fn zmq_ctx_set (ctx_: *mut c_void, option_: i32, mut optval_: i32) -> i32
{
    return zmq_ctx_set_ext (ctx_, option_, &mut optval_ as *mut c_void, mem::size_of::<i32>());
}

pub fn zmq_ctx_set_ext (ctx_: *mut c_void,
                     option_: i32,
                     optval_: *mut c_void,
                     optvallen_: usize) -> i32
{
    if !ctx_ || !(ctx_ as *mut ZmqContext).check_tag () {
        errno = EFAULT;
        return -1;
    }
    return (ctx_ as *mut ZmqContext).set(option_, optval_, optvallen_);
}

pub fn zmq_ctx_get (ctx_: *mut c_void, option_: i32) -> i32
{
    if !ctx_ || !(ctx_ as *mut ZmqContext).check_tag () {
        errno = EFAULT;
        return -1;
    }
    return (ctx_ as *mut ZmqContext).get (option_);
}

pub fn zmq_ctx_get_ext (ctx_: *mut c_void, option_: i32, optval_: *mut c_void, optvallen_: *mut usize) -> i32
{
    if !ctx_ || !(ctx_ as *mut ZmqContext).check_tag () {
        errno = EFAULT;
        return -1;
    }
    return (ctx_ as *mut ZmqContext).get(option_, optval_, optvallen_);
}


//  Stable/legacy context API

pub fn zmq_init (io_threads_: i32) -> *mut c_void
{
    if io_threads_ >= 0 {
        void *ctx = zmq_ctx_new ();
        zmq_ctx_set (ctx, ZMQ_IO_THREADS, io_threads_);
        return ctx;
    }
    errno = EINVAL;
    return null_mut();
}

pub fn zmq_term (ctx_: *mut c_void) -> i32
{
    return zmq_ctx_term (ctx_);
}

pub fn zmq_ctx_destroy (ctx_: *mut c_void) -> i32
{
    return zmq_ctx_term (ctx_);
}


// Sockets

pub fn as_socket_base_t (s_: *mut c_void) -> *mut ZmqSocketBase
{
    // zmq::ZmqSocketBase *s = static_cast<zmq::ZmqSocketBase *> (s_);
    let mut s: *mut ZmqSocketBase = s_ as *mut ZmqSocketBase;
    if s_.is_null() || !s.check_tag () {
        errno = ENOTSOCK;
        return null_mut();
    }
    return s;
}

pub fn zmq_socket(ctx_: *mut c_void, type_: i32) -> *mut c_void {
    if !ctx_ || !(ctx_ as *mut ZmqContext).check_tag() {
        errno = EFAULT;
        return null_mut();
    }
    let mut ctx: *mut ZmqContext = ctx_ as *mut ZmqContext;
    let mut s: *mut ZmqSocketBase = ctx.create_socket(type_);
    return s as *mut c_void;
}

pub fn zmq_close(s_: *mut c_void) -> i32 {
    let mut s: *mut ZmqSocketBase = as_socket_base_t(s_);
    if (!s) {
        return -1;
    }
    s.close();
    return 0;
}

pub fn zmq_setsockopt(s_: *mut c_void,
                      option_: i32,
                      optval_: *mut c_void,
                      optvallen_: usize) -> i32 {
    let mut s: *mut ZmqSocketBase = as_socket_base_t(s_);
    if (!s) {
        return -1;
    }
    return s.setsockopt(option_, optval_, optvallen_);
}

pub fn zmq_getsockopt (s_: *mut c_void, option_: i32, optval_: *mut c_void, optvallen_: *mut usize) -> i32
{
    let mut s: *mut ZmqSocketBase = as_socket_base_t (s_);
    if (!s) {
        return -1;
    }
    return s.getsockopt (option_, optval_, optvallen_);
}

pub fn zmq_socket_monitor_versioned(
  s_: *mut c_void, addr_: *const c_char, events_: u64, event_version_: i32, type_: i32) -> i32
{
    let mut s: *mut ZmqSocketBase = as_socket_base_t (s_);
    if !s {
        return -1;
    }
    return s.monitor (addr_, events_, event_version_, type_);
}

pub fn zmq_socket_monitor (s_: *mut c_void, addr_: *const c_char, events_: u64) -> i32
{
    return zmq_socket_monitor_versioned (s_, addr_, events_, 1, ZMQ_PAIR);
}

pub fn zmq_join (s_: *mut c_void, group_: *const c_char) -> i32
{
    let mut s: *mut ZmqSocketBase = as_socket_base_t (s_);
    if !s {
        return -1;
    }
    return s.join (group_);
}

pub fn zmq_leave (s_: *mut c_void, group_: *const c_char) -> i32
{
    let mut s: *mut ZmqSocketBase =  as_socket_base_t (s_);
    if !s {
        return -1;
    }
    return s.leave (group_);
}

pub fn zmq_bind (s_: *mut c_void, addr_: *const c_char) -> i32
{
    let mut s: *mut ZmqSocketBase =  as_socket_base_t (s_);
    if (!s) {
        return -1;
    }
    return s.bind (addr_);
}

pub fn zmq_connect (s_: *mut c_void, addr_: *const c_char) -> i32
{
    let mut s: *mut ZmqSocketBase =  as_socket_base_t (s_);
    if (!s){
return - 1;}
    return s.connect (addr_);
}

pub fn zmq_connect_peer (s_: *mut c_void, addr_: *const c_char) -> u32
{
    let mut s: *mut peer_t = s_ as *mut peer_t;
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


pub fn zmq_unbind (s_: *mut c_void, addr_: *const c_char) -> i32
{
    let mut s: *mut ZmqSocketBase =  as_socket_base_t (s_);
    if (!s) {
        return -1;
    }
    return s.term_endpoint (addr_);
}

pub fn zmq_disconnect (s_: *mut c_void, addr_: *const c_char) -> i32
{
    let mut s: *mut ZmqSocketBase =  as_socket_base_t (s_);
    if !s {
        return -1;
    }
    return s.term_endpoint (addr_);
}

// Sending functions.

pub fn s_sendmsg(s_: *mut ZmqSocketBase, msg_: *mut zmq_msg_t, flags_: i32) -> i32 {
    let mut sz: usize = zmq_msg_size(msg_);
    let rc = s_.send(msg_ as *mut msg_t, flags_);
    if unlikely(rc < 0) {
        return -1;
    }

    //  This is what I'd like to do, my C++ fu is too weak -- PH 2016/02/09
    //  int max_msgsz = s_->parent->get (ZMQ_MAX_MSGSZ);
    let max_msgsz: usize = INT_MAX as usize;

    //  Truncate returned size to INT_MAX to avoid overflow to negative values
    return if sz < max_msgsz { sz } else { max_msgsz } as i32;
}

/*  To be deprecated once zmq_msg_send() is stable                           */
pub fn zmq_sendmsg (s_: *mut c_void, msg_: *mut zmq_msg_t, flags_: i32) -> i32
{
    return zmq_msg_send (msg_, s_, flags_);
}

pub fn zmq_send (s_: *mut c_void, buf_: *mut c_void, len_: usize, flags_: i32) -> i32
{
    let mut s: *mut ZmqSocketBase =  as_socket_base_t (s_);
    if (!s) {
        return -1;
    }
    let mut msg: zmq_msg_t = zmq_msg_t::default();
    let mut rc = zmq_msg_init_buffer (&mut msg, buf_, len_);
    if unlikely (rc < 0) {
        return -1;
    }

    rc = s_sendmsg (s, &mut msg, flags_);
    if unlikely (rc < 0) {
        let err = errno;
        let rc2 = zmq_msg_close (&msg);
        errno_assert (rc2 == 0);
        errno = err;
        return -1;
    }
    //  Note the optimisation here. We don't close the msg object as it is
    //  empty anyway. This may change when implementation of zmq_msg_t changes.
    return rc;
}

pub fn zmq_send_const(s_: *mut c_void, buf_: *mut c_void, len_: usize, flags_: i32) -> i32 {
    let mut s: *mut ZmqSocketBase = as_socket_base_t(s_);
    if (!s) {
        return -1;
    }
    let mut msg: zmq_msg_t = zmq_msg_t { _x: [0; 64] };
    let rc = zmq_msg_init_data(&msg, buf as *mut c_void, len_, null_mut(), null_mut());
    if rc != 0 {
        return -1;
    }

    rc = s_sendmsg(s, &mut msg, flags_);
    if unlikely(rc < 0) {
        let err = errno;
        let rc2 = zmq_msg_close(&msg);
        errno_assert(rc2 == 0);
        errno = err;
        return -1;
    }
    //  Note the optimisation here. We don't close the msg object as it is
    //  empty anyway. This may change when implementation of zmq_msg_t changes.
    return rc;
}


// Send multiple messages.
// TODO: this function has no man page
//
// If flag bit ZMQ_SNDMORE is set the vector is treated as
// a single multi-part message, i.e. the last message has
// ZMQ_SNDMORE bit switched off.
//
pub fn zmq_sendiov (s_: *mut c_void, a_: *mut iovec, count_: usize, mut flags_: i32) -> i32
{
    let mut s: *mut ZmqSocketBase =  as_socket_base_t (s_);
    if !s {
        return -1;
    }
    if unlikely (count_ <= 0 || a_.is_null()) {
        errno = EINVAL;
        return -1;
    }

    let mut rc = 0;
    let mut msg: zmq_msg_t = zmq_msg_t{_x: [0;64]};

    // for (size_t i = 0; i < count_; ++i)
    for i in 0 .. count_
    {
        rc = zmq_msg_init_size (&mut msg, a_[i].iov_len);
        if rc != 0 {
            rc = -1;
            break;
        }
        unsafe { libc::memcpy(zmq_msg_data(&msg), a_[i].iov_base, a_[i].iov_len); }
        if i == count_ - 1 {
            flags_ = flags_ & !ZMQ_SNDMORE;
        }
        rc = s_sendmsg (s, &mut msg, flags_);
        if unlikely (rc < 0) {
            let err = errno;
            let rc2 = zmq_msg_close (&msg);
            errno_assert (rc2 == 0);
            errno = err;
            rc = -1;
            break;
        }
    }
    return rc;
}

// Receiving functions.

pub fn s_recvmsg (s_: *mut ZmqSocketBase, msg_: *mut zmq_msg_t, flags_: i32) -> i32
{
    let rc = s_.recv (msg_ as *mut msg_t, flags_);
    if unlikely (rc < 0) {
        return -1;
    }

    //  Truncate returned size to INT_MAX to avoid overflow to negative values
    let sz = zmq_msg_size (msg_);
    return if sz < INT_MAX { sz } else {INT_MAX};
}

/*  To be deprecated once zmq_msg_recv() is stable                           */
pub fn zmq_recvmsg (s_: *mut c_void, msg_: *mut zmq_msg_t, flags_: i32) -> i32
{
    return zmq_msg_recv (msg_, s_, flags_);
}


pub fn zmq_recv (s_: *mut c_void, buf_: *mut c_void, len_: usize, flags_: i32) -> i32
{
    let mut s: *mut ZmqSocketBase =  as_socket_base_t (s_);
    if (!s) {
        return -1;
    }
    let mut msg: zmq_msg_t = zmq_msg_t::default();
    let mut rc = zmq_msg_init (&mut msg);
    errno_assert (rc == 0);

    let nbytes = s_recvmsg (s, &mut msg, flags_);
    if unlikely (nbytes < 0) {
        let err = errno;
        rc = zmq_msg_close (&mut msg);
        errno_assert (rc == 0);
        errno = err;
        return -1;
    }

    //  An oversized message is silently truncated.
    let to_copy = if (nbytes) < len_ as i32 { nbytes } else { len_ };

    //  We explicitly allow a null buffer argument if len is zero
    if to_copy {
        // assert (buf_);
        unsafe { libc::memcpy(buf_, zmq_msg_data(&msg), to_copy as usize); }
    }
    rc = zmq_msg_close (&msg);
    errno_assert (rc == 0);

    return nbytes;
}

// Receive a multi-part message
//
// Receives up to *count_ parts of a multi-part message.
// Sets *count_ to the actual number of parts read.
// ZMQ_RCVMORE is set to indicate if a complete multi-part message was read.
// Returns number of message parts read, or -1 on error.
//
// Note: even if -1 is returned, some parts of the message
// may have been read. Therefore the client must consult
// *count_ to retrieve message parts successfully read,
// even if -1 is returned.
//
// The iov_base* buffers of each iovec *a_ filled in by this
// function may be freed using free().
// TODO: this function has no man page
//
pub fn zmq_recviov (s_: *mut c_void, a_: *mut iovec, count: *mut usize, flags_: i32) -> i32
{
    let mut s: *mut ZmqSocketBase =  as_socket_base_t (s_);
    if !s {
        return -1;
    }
    if unlikely (!count_ || *count_ <= 0 || a_.is_null()) {
        errno = EINVAL;
        return -1;
    }

    let count = *count_;
    let mut nread = 0;
    let mut recvmore = true;

    *count_ = 0;

    // for (size_t i = 0; recvmore && i < count; ++i)
    for i in 0 .. recvmore
    {
        let mut msg = zmq_msg_t::default();
        let mut rc = zmq_msg_init (&mut msg);
        errno_assert (rc == 0);

        let nbytes = s_recvmsg (s, &mut msg, flags_);
        if unlikely (nbytes < 0) {
            let err = errno;
            rc = zmq_msg_close (&mut msg);
            errno_assert (rc == 0);
            errno = err;
            nread = -1;
            break;
        }

        a_[i].iov_len = zmq_msg_size (&msg);
        unsafe { a_[i].iov_base = libc::malloc(a_[i].iov_len); }
        if unlikely (!a_[i].iov_base) {
            errno = ENOMEM;
            return -1;
        }
        unsafe {
            libc::memcpy(a_[i].iov_base, zmq_msg_data(&msg) as *mut c_void,
                         a_[i].iov_len);
        }
        // Assume zmq_socket ZMQ_RVCMORE is properly set.
        let p_msg = &mut msg;
        recvmore = p_msg.flags() & msg_t::more;
        rc = zmq_msg_close (&msg);
        errno_assert (rc == 0);
        *count_ += 1;
        nread += 1;
    }
    return nread;
}

// Message manipulators.

pub fn zmq_msg_init (msg_: *mut zmq_msg_t) -> i32
{
    return (msg_ as *mut zmq_msg_t).init();
}

pub fn zmq_msg_init_size (msg_: *mut zmq_msg_t, size_: usize) -> i32
{
    return (msg_ as *mut zmq_msg_t).init_size(size_);
}

pub fn zmq_msg_init_buffer (msg_: *mut zmq_msg_t, buf_: *mut c_void, size_: usize) -> i32
{
    return (msg_ as *mut zmq_msg_t).init_buffer (buf_, size_);
}

pub fn zmq_msg_init_data (
  msg_: *mut zmq_msg_t, data_: *mut c_void, size_: usize, ffn_: zmq_free_fn, hint_: *mut c_void) -> i32
{
    return (msg_ as *mut zmq_msg_t).init_data (data_, size_, ffn_, hint_);
}

pub fn zmq_msg_send (msg_: *mut zmq_msg_t, s_: *mut c_void, flags_: i32) -> i32
{
    let mut s: *mut ZmqSocketBase =  as_socket_base_t (s_);
    if (!s) {
        return -1;
    }
    return s_sendmsg (s, msg_, flags_);
}

pub fn zmq_msg_recv (msg_: *mut zmq_msg_t, s_: *mut c_void, flags_: i32) -> i32
{
    let mut s: *mut ZmqSocketBase =  as_socket_base_t (s_);
    if (!s) {
        return -1;
    }
    return s_recvmsg (s, msg_, flags_);
}

pub fn zmq_msg_close (msg_: *mut zmq_msg_t) -> i32
{
    return (msg_ as *mut msg_t).close();
}

pub fn zmq_msg_move (dest_: *mut zmq_msg_t, src_: *mut zmq_msg_t)
{
    return (dest_ as *mut msg_t).move(src_ as *mut zmq_msg_t);
}

int zmq_msg_copy (dest_: *mut zmq_msg_t, zmq_msg_t *src_)
{
    return (reinterpret_cast<zmq::msg_t *> (dest_))
      ->copy (*reinterpret_cast<zmq::msg_t *> (src_));
}

void *zmq_msg_data (zmq_msg_t *msg_)
{
    return (msg_ as *mut zmq_msg_t).data ();
}

size_t zmq_msg_size (const zmq_msg_t *msg_)
{
    return ((zmq::msg_t *) msg_)->size ();
}

int zmq_msg_more (const zmq_msg_t *msg_)
{
    return zmq_msg_get (msg_, ZMQ_MORE);
}

int zmq_msg_get (const msg_: *mut zmq_msg_t, property_: i32)
{
    const char *fd_string;

    switch (property_) {
        case ZMQ_MORE:
            return (((zmq::msg_t *) msg_)->flags () & zmq::msg_t::more) ? 1 : 0;
        case ZMQ_SRCFD:
            fd_string = zmq_msg_gets (msg_, "__fd");
            if (fd_string == null_mut())
                return -1;

            return atoi (fd_string);
        case ZMQ_SHARED:
            return (((zmq::msg_t *) msg_)->is_cmsg ())
                       || (((zmq::msg_t *) msg_)->flags () & zmq::msg_t::shared)
                     ? 1
                     : 0;
        default:
            errno = EINVAL;
            return -1;
    }
}

int zmq_msg_set (zmq_msg_t *, int, int)
{
    //  No properties supported at present
    errno = EINVAL;
    return -1;
}

int zmq_msg_set_routing_id (msg_: *mut zmq_msg_t, uint32_t routing_id_)
{
    return (msg_ as *mut zmq_msg_t)
      ->set_routing_id (routing_id_);
}

uint32_t zmq_msg_routing_id (zmq_msg_t *msg_)
{
    return (msg_ as *mut zmq_msg_t).get_routing_id ();
}

int zmq_msg_set_group (msg_: *mut zmq_msg_t, group_: *const c_char)
{
    return (msg_ as *mut zmq_msg_t).set_group (group_);
}

const char *zmq_msg_group (zmq_msg_t *msg_)
{
    return (msg_ as *mut zmq_msg_t).group ();
}

//  Get message metadata string

const char *zmq_msg_gets (const msg_: *mut zmq_msg_t, property_: *const c_char)
{
    const zmq::metadata_t *metadata =
      reinterpret_cast<const zmq::msg_t *> (msg_)->metadata ();
    const char *value = null_mut();
    if (metadata)
        value = metadata->get (std::string (property_));
    if (value)
        return value;

    errno = EINVAL;
    return null_mut();
}

// Polling.

// #if defined ZMQ_HAVE_POLLER
static int zmq_poller_poll (zmq_pollitem_t *items_, nitems_: i32, long timeout_)
{
    // implement zmq_poll on top of zmq_poller
    rc: i32;
    zmq_poller_event_t *events;
    zmq::socket_poller_t poller;
    events = new (std::nothrow) zmq_poller_event_t[nitems_];
    alloc_assert (events);

    bool repeat_items = false;
    //  Register sockets with poller
    for (int i = 0; i < nitems_; i++) {
        items_[i].revents = 0;

        bool modify = false;
        short e = items_[i].events;
        if (items_[i].socket) {
            //  Poll item is a 0MQ socket.
            for (int j = 0; j < i; ++j) {
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
            for (int j = 0; j < i; ++j) {
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
    rc = zmq_poller_wait_all (&poller, events, nitems_, timeout_);
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
    for (int i = 0; i < nitems_; i++) {
        for (int j = j_start; j < found_events; ++j) {
            if ((items_[i].socket && items_[i].socket == events[j].socket)
                || (!(items_[i].socket || events[j].socket)
                    && items_[i].fd == events[j].fd)) {
                items_[i].revents = events[j].events & items_[i].events;
                if (!repeat_items) {
                    // no repeats, we can ignore events we've already seen
                    j_start++;
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

int zmq_poll (zmq_pollitem_t *items_, nitems_: i32, long timeout_)
{
// #if defined ZMQ_HAVE_POLLER
    // if poller is present, use that if there is at least 1 thread-safe socket,
    // otherwise fall back to the previous implementation as it's faster.
    for (int i = 0; i != nitems_; i++) {
        if (items_[i].socket) {
            let mut s: *mut ZmqSocketBase =  as_socket_base_t (items_[i].socket);
            if (s) {
                if (s.is_thread_safe ())
                    return zmq_poller_poll (items_, nitems_, timeout_);
            } else {
                //as_socket_base_t returned null_mut() : socket is invalid
                return -1;
            }
        }
    }
// #endif // ZMQ_HAVE_POLLER
// #if defined ZMQ_POLL_BASED_ON_POLL || defined ZMQ_POLL_BASED_ON_SELECT
    if (unlikely (nitems_ < 0)) {
        errno = EINVAL;
        return -1;
    }
    if (unlikely (nitems_ == 0)) {
        if (timeout_ == 0)
            return 0;
// #if defined ZMQ_HAVE_WINDOWS
        Sleep (timeout_ > 0 ? timeout_ : INFINITE);
        return 0;
#elif defined ZMQ_HAVE_VXWORKS
        struct timespec ns_;
        ns_.tv_sec = timeout_ / 1000;
        ns_.tv_nsec = timeout_ % 1000 * 1000000;
        return nanosleep (&ns_, 0);
// #else
        return usleep (timeout_ * 1000);
// #endif
    }
    if (!items_) {
        errno = EFAULT;
        return -1;
    }

    zmq::clock_t clock;
    u64 now = 0;
    u64 end = 0;
// #if defined ZMQ_POLL_BASED_ON_POLL
    zmq::fast_vector_t<pollfd, ZMQ_POLLITEMS_DFLT> pollfds (nitems_);

    //  Build pollset for poll () system call.
    for (int i = 0; i != nitems_; i++) {
        //  If the poll item is a 0MQ socket, we poll on the file descriptor
        //  retrieved by the ZMQ_FD socket option.
        if (items_[i].socket) {
            size_t zmq_fd_size = sizeof (zmq::fd_t);
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
    zmq_assert (nitems_ <= FD_SETSIZE);

    zmq::optimized_fd_set_t pollset_in (nitems_);
    FD_ZERO (pollset_in.get ());
    zmq::optimized_fd_set_t pollset_out (nitems_);
    FD_ZERO (pollset_out.get ());
    zmq::optimized_fd_set_t pollset_err (nitems_);
    FD_ZERO (pollset_err.get ());

    zmq::fd_t maxfd = 0;

    //  Build the fd_sets for passing to select ().
    for (int i = 0; i != nitems_; i++) {
        //  If the poll item is a 0MQ socket we are interested in input on the
        //  notification file descriptor retrieved by the ZMQ_FD socket option.
        if (items_[i].socket) {
            size_t zmq_fd_size = sizeof (zmq::fd_t);
            zmq::fd_t notify_fd;
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

    zmq::optimized_fd_set_t inset (nitems_);
    zmq::optimized_fd_set_t outset (nitems_);
    zmq::optimized_fd_set_t errset (nitems_);
// #endif

    bool first_pass = true;
    int nevents = 0;

    while (true) {
// #if defined ZMQ_POLL_BASED_ON_POLL

        //  Compute the timeout for the subsequent poll.
        const zmq::timeout_t timeout =
          zmq::compute_timeout (first_pass, timeout_, now, end);

        //  Wait for events.
        {
            const int rc = poll (&pollfds[0], nitems_, timeout);
            if (rc == -1 && errno == EINTR) {
                return -1;
            }
            errno_assert (rc >= 0);
        }
        //  Check for the events.
        for (int i = 0; i != nitems_; i++) {
            items_[i].revents = 0;

            //  The poll item is a 0MQ socket. Retrieve pending events
            //  using the ZMQ_EVENTS socket option.
            if (items_[i].socket) {
                size_t zmq_events_size = mem::size_of::<uint32_t>();
                uint32_t zmq_events;
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
            //  the events to zmq_pollitem_t-style format.
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
                nevents++;
        }

// #else

        //  Compute the timeout for the subsequent poll.
        timeval timeout;
        timeval *ptimeout;
        if (first_pass) {
            timeout.tv_sec = 0;
            timeout.tv_usec = 0;
            ptimeout = &timeout;
        } else if (timeout_ < 0)
            ptimeout = null_mut();
        else {
            timeout.tv_sec = static_cast<long> ((end - now) / 1000);
            timeout.tv_usec = static_cast<long> ((end - now) % 1000 * 1000);
            ptimeout = &timeout;
        }

        //  Wait for events. Ignore interrupts if there's infinite timeout.
        while (true) {
            memcpy (inset.get (), pollset_in.get (),
                    zmq::valid_pollset_bytes (*pollset_in.get ()));
            memcpy (outset.get (), pollset_out.get (),
                    zmq::valid_pollset_bytes (*pollset_out.get ()));
            memcpy (errset.get (), pollset_err.get (),
                    zmq::valid_pollset_bytes (*pollset_err.get ()));
// #if defined ZMQ_HAVE_WINDOWS
            int rc =
              select (0, inset.get (), outset.get (), errset.get (), ptimeout);
            if (unlikely (rc == SOCKET_ERROR)) {
                errno = zmq::wsa_error_to_errno (WSAGetLastError ());
                wsa_assert (errno == ENOTSOCK);
                return -1;
            }
// #else
            int rc = select (maxfd + 1, inset.get (), outset.get (),
                             errset.get (), ptimeout);
            if (unlikely (rc == -1)) {
                errno_assert (errno == EINTR || errno == EBADF);
                return -1;
            }
// #endif
            break;
        }

        //  Check for the events.
        for (int i = 0; i != nitems_; i++) {
            items_[i].revents = 0;

            //  The poll item is a 0MQ socket. Retrieve pending events
            //  using the ZMQ_EVENTS socket option.
            if (items_[i].socket) {
                size_t zmq_events_size = mem::size_of::<uint32_t>();
                uint32_t zmq_events;
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
            //  the events to zmq_pollitem_t-style format.
            else {
                if (FD_ISSET (items_[i].fd, inset.get ()))
                    items_[i].revents |= ZMQ_POLLIN;
                if (FD_ISSET (items_[i].fd, outset.get ()))
                    items_[i].revents |= ZMQ_POLLOUT;
                if (FD_ISSET (items_[i].fd, errset.get ()))
                    items_[i].revents |= ZMQ_POLLERR;
            }

            if (items_[i].revents)
                nevents++;
        }
// #endif

        //  If timeout is zero, exit immediately whether there are events or not.
        if (timeout_ == 0)
            break;

        //  If there are events to return, we can exit immediately.
        if (nevents)
            break;

        //  At this point we are meant to wait for events but there are none.
        //  If timeout is infinite we can just loop until we get some events.
        if (timeout_ < 0) {
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
            end = now + timeout_;
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
int zmq_poll_check_items_ (zmq_pollitem_t *items_, nitems_: i32, long timeout_)
{
    if (unlikely (nitems_ < 0)) {
        errno = EINVAL;
        return -1;
    }
    if (unlikely (nitems_ == 0)) {
        if (timeout_ == 0)
            return 0;
// #if defined ZMQ_HAVE_WINDOWS
        Sleep (timeout_ > 0 ? timeout_ : INFINITE);
        return 0;
#elif defined ZMQ_HAVE_VXWORKS
        struct timespec ns_;
        ns_.tv_sec = timeout_ / 1000;
        ns_.tv_nsec = timeout_ % 1000 * 1000000;
        return nanosleep (&ns_, 0);
// #else
        return usleep (timeout_ * 1000);
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

    zmq::optimized_fd_set_t pollset_in;
    zmq::optimized_fd_set_t pollset_out;
    zmq::optimized_fd_set_t pollset_err;
    zmq::optimized_fd_set_t inset;
    zmq::optimized_fd_set_t outset;
    zmq::optimized_fd_set_t errset;
    zmq::fd_t maxfd;
};

zmq_poll_select_fds_t_
zmq_poll_build_select_fds_ (zmq_pollitem_t *items_, nitems_: i32, int &rc)
{
    //  Ensure we do not attempt to select () on more than FD_SETSIZE
    //  file descriptors.
    //  TODO since this function is called by a client, we could return errno EINVAL/ENOMEM/... here
    zmq_assert (nitems_ <= FD_SETSIZE);

    zmq_poll_select_fds_t_ fds (nitems_);

    //  Build the fd_sets for passing to select ().
    for (int i = 0; i != nitems_; i++) {
        //  If the poll item is a 0MQ socket we are interested in input on the
        //  notification file descriptor retrieved by the ZMQ_FD socket option.
        if (items_[i].socket) {
            size_t zmq_fd_size = sizeof (zmq::fd_t);
            zmq::fd_t notify_fd;
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
  long timeout_, bool first_pass, now: u64, end: u64, timeval &timeout)
{
    timeval *ptimeout;
    if (first_pass) {
        timeout.tv_sec = 0;
        timeout.tv_usec = 0;
        ptimeout = &timeout;
    } else if (timeout_ < 0)
        ptimeout = null_mut();
    else {
        timeout.tv_sec = static_cast<long> ((end - now) / 1000);
        timeout.tv_usec = static_cast<long> ((end - now) % 1000 * 1000);
        ptimeout = &timeout;
    }
    return ptimeout;
}

timespec *zmq_poll_select_set_timeout_ (
  long timeout_, bool first_pass, now: u64, end: u64, timespec &timeout)
{
    timespec *ptimeout;
    if (first_pass) {
        timeout.tv_sec = 0;
        timeout.tv_nsec = 0;
        ptimeout = &timeout;
    } else if (timeout_ < 0)
        ptimeout = null_mut();
    else {
        timeout.tv_sec = static_cast<long> ((end - now) / 1000);
        timeout.tv_nsec = static_cast<long> ((end - now) % 1000 * 1000000);
        ptimeout = &timeout;
    }
    return ptimeout;
}

int zmq_poll_select_check_events_ (zmq_pollitem_t *items_,
                                   nitems_: i32,
                                   zmq_poll_select_fds_t_ &fds,
                                   int &nevents)
{
    //  Check for the events.
    for (int i = 0; i != nitems_; i++) {
        items_[i].revents = 0;

        //  The poll item is a 0MQ socket. Retrieve pending events
        //  using the ZMQ_EVENTS socket option.
        if (items_[i].socket) {
            size_t zmq_events_size = mem::size_of::<uint32_t>();
            uint32_t zmq_events;
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
        //  the events to zmq_pollitem_t-style format.
        else {
            if (FD_ISSET (items_[i].fd, fds.inset.get ()))
                items_[i].revents |= ZMQ_POLLIN;
            if (FD_ISSET (items_[i].fd, fds.outset.get ()))
                items_[i].revents |= ZMQ_POLLOUT;
            if (FD_ISSET (items_[i].fd, fds.errset.get ()))
                items_[i].revents |= ZMQ_POLLERR;
        }

        if (items_[i].revents)
            nevents++;
    }

    return 0;
}

bool zmq_poll_must_break_loop_ (long timeout_,
                                nevents: i32,
                                bool &first_pass,
                                zmq::clock_t &clock,
                                u64 &now,
                                u64 &end)
{
    //  If timeout is zero, exit immediately whether there are events or not.
    if (timeout_ == 0)
        return true;

    //  If there are events to return, we can exit immediately.
    if (nevents)
        return true;

    //  At this point we are meant to wait for events but there are none.
    //  If timeout is infinite we can just loop until we get some events.
    if (timeout_ < 0) {
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
        end = now + timeout_;
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
int zmq_ppoll (zmq_pollitem_t *items_,
               nitems_: i32,
               long timeout_,
               const sigset_t *sigmask_)
// #else
// Windows has no sigset_t
int zmq_ppoll (zmq_pollitem_t *items_,
               nitems_: i32,
               long timeout_,
               const sigmask_: *mut c_void)
// #endif
{
// #ifdef ZMQ_HAVE_PPOLL
    int rc = zmq_poll_check_items_ (items_, nitems_, timeout_);
    if (rc <= 0) {
        return rc;
    }

    zmq::clock_t clock;
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
        timespec *ptimeout = zmq_poll_select_set_timeout_ (timeout_, first_pass,
                                                           now, end, timeout);

        //  Wait for events. Ignore interrupts if there's infinite timeout.
        while (true) {
            memcpy (fds.inset.get (), fds.pollset_in.get (),
                    zmq::valid_pollset_bytes (*fds.pollset_in.get ()));
            memcpy (fds.outset.get (), fds.pollset_out.get (),
                    zmq::valid_pollset_bytes (*fds.pollset_out.get ()));
            memcpy (fds.errset.get (), fds.pollset_err.get (),
                    zmq::valid_pollset_bytes (*fds.pollset_err.get ()));
            int rc =
              pselect (fds.maxfd + 1, fds.inset.get (), fds.outset.get (),
                       fds.errset.get (), ptimeout, sigmask_);
            if (unlikely (rc == -1)) {
                errno_assert (errno == EINTR || errno == EBADF);
                return -1;
            }
            break;
        }

        rc = zmq_poll_select_check_events_ (items_, nitems_, fds, nevents);
        if (rc < 0) {
            return rc;
        }

        if (zmq_poll_must_break_loop_ (timeout_, nevents, first_pass, clock,
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
    zmq::socket_poller_t *poller = new (std::nothrow) zmq::socket_poller_t;
    if (!poller) {
        errno = ENOMEM;
    }
    return poller;
}

int zmq_poller_destroy (void **poller_p_)
{
    if (poller_p_) {
        const zmq::socket_poller_t *const poller =
          static_cast<const zmq::socket_poller_t *> (*poller_p_);
        if (poller && poller->check_tag ()) {
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
        || !(static_cast<zmq::socket_poller_t *> (poller_))->check_tag ()) {
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

    if (!s_ || !(static_cast<zmq::ZmqSocketBase *> (s_))->check_tag ()) {
        errno = ENOTSOCK;
        return -1;
    }

    return 0;
}

static int check_poller_fd_registration_args (poller_: *const c_void,
                                              const zmq::fd_t fd_)
{
    if (-1 == check_poller (poller_))
        return -1;

    if (fd_ == zmq::retired_fd) {
        errno = EBADF;
        return -1;
    }

    return 0;
}

int zmq_poller_size (poller_: *mut c_void)
{
    if (-1 == check_poller (poller_))
        return -1;

    return (static_cast<zmq::socket_poller_t *> (poller_))->size ();
}

int zmq_poller_add (poller_: *mut c_void, s_: *mut c_void, user_data_: *mut c_void, short events_)
{
    if (-1 == check_poller_registration_args (poller_, s_)
        || -1 == check_events (events_))
        return -1;

    let mut socket: *mut ZmqSocketBase =  static_cast<zmq::ZmqSocketBase *> (s_);

    return (static_cast<zmq::socket_poller_t *> (poller_))
      ->add (socket, user_data_, events_);
}

int zmq_poller_add_fd (poller_: *mut c_void,
                       zmq::fd_t fd_,
                       user_data_: *mut c_void,
                       short events_)
{
    if (-1 == check_poller_fd_registration_args (poller_, fd_)
        || -1 == check_events (events_))
        return -1;

    return (static_cast<zmq::socket_poller_t *> (poller_))
      ->add_fd (fd_, user_data_, events_);
}


int zmq_poller_modify (poller_: *mut c_void, s_: *mut c_void, short events_)
{
    if (-1 == check_poller_registration_args (poller_, s_)
        || -1 == check_events (events_))
        return -1;

    const zmq::ZmqSocketBase *const socket =
      static_cast<const zmq::ZmqSocketBase *> (s_);

    return (static_cast<zmq::socket_poller_t *> (poller_))
      ->modify (socket, events_);
}

int zmq_poller_modify_fd (poller_: *mut c_void, zmq::fd_t fd_, short events_)
{
    if (-1 == check_poller_fd_registration_args (poller_, fd_)
        || -1 == check_events (events_))
        return -1;

    return (static_cast<zmq::socket_poller_t *> (poller_))
      ->modify_fd (fd_, events_);
}

int zmq_poller_remove (poller_: *mut c_void, s_: *mut c_void)
{
    if (-1 == check_poller_registration_args (poller_, s_))
        return -1;

    let mut socket: *mut ZmqSocketBase =  static_cast<zmq::ZmqSocketBase *> (s_);

    return (static_cast<zmq::socket_poller_t *> (poller_))->remove (socket);
}

int zmq_poller_remove_fd (poller_: *mut c_void, zmq::fd_t fd_)
{
    if (-1 == check_poller_fd_registration_args (poller_, fd_))
        return -1;

    return (static_cast<zmq::socket_poller_t *> (poller_))->remove_fd (fd_);
}

int zmq_poller_wait (poller_: *mut c_void, zmq_poller_event_t *event_, long timeout_)
{
    const int rc = zmq_poller_wait_all (poller_, event_, 1, timeout_);

    if (rc < 0 && event_) {
        event_->socket = null_mut();
        event_->fd = zmq::retired_fd;
        event_->user_data = null_mut();
        event_->events = 0;
    }
    // wait_all returns number of events, but we return 0 for any success
    return rc >= 0 ? 0 : rc;
}

int zmq_poller_wait_all (poller_: *mut c_void,
                         zmq_poller_event_t *events_,
                         n_events_: i32,
                         long timeout_)
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

    const int rc =
      (static_cast<zmq::socket_poller_t *> (poller_))
        ->wait (reinterpret_cast<zmq::socket_poller_t::event_t *> (events_),
                n_events_, timeout_);

    return rc;
}

int zmq_poller_fd (poller_: *mut c_void, zmq_fd_t *fd_)
{
    if (!poller_
        || !(static_cast<zmq::socket_poller_t *> (poller_)->check_tag ())) {
        errno = EFAULT;
        return -1;
    }
    return static_cast<zmq::socket_poller_t *> (poller_)->signaler_fd (fd_);
}

//  Peer-specific state

int zmq_socket_get_peer_state (s_: *mut c_void,
                               const routing_id_: *mut c_void,
                               routing_id_size_: usize)
{
    const zmq::ZmqSocketBase *const s = as_socket_base_t (s_);
    if (!s)
        return -1;

    return s.get_peer_state (routing_id_, routing_id_size_);
}

//  Timers

void *zmq_timers_new (void)
{
    zmq::timers_t *timers = new (std::nothrow) zmq::timers_t;
    alloc_assert (timers);
    return timers;
}

int zmq_timers_destroy (void **timers_p_)
{
    void *timers = *timers_p_;
    if (!timers || !(static_cast<zmq::timers_t *> (timers))->check_tag ()) {
        errno = EFAULT;
        return -1;
    }
    delete (static_cast<zmq::timers_t *> (timers));
    *timers_p_ = null_mut();
    return 0;
}

int zmq_timers_add (timers_: *mut c_void,
                    interval_: usize,
                    zmq_timer_fn handler_,
                    arg_: *mut c_void)
{
    if (!timers_ || !(static_cast<zmq::timers_t *> (timers_))->check_tag ()) {
        errno = EFAULT;
        return -1;
    }

    return (static_cast<zmq::timers_t *> (timers_))
      ->add (interval_, handler_, arg_);
}

int zmq_timers_cancel (timers_: *mut c_void, timer_id_: i32)
{
    if (!timers_ || !(static_cast<zmq::timers_t *> (timers_))->check_tag ()) {
        errno = EFAULT;
        return -1;
    }

    return (static_cast<zmq::timers_t *> (timers_))->cancel (timer_id_);
}

int zmq_timers_set_interval (timers_: *mut c_void, timer_id_: i32, interval_: usize)
{
    if (!timers_ || !(static_cast<zmq::timers_t *> (timers_))->check_tag ()) {
        errno = EFAULT;
        return -1;
    }

    return (static_cast<zmq::timers_t *> (timers_))
      ->set_interval (timer_id_, interval_);
}

int zmq_timers_reset (timers_: *mut c_void, timer_id_: i32)
{
    if (!timers_ || !(static_cast<zmq::timers_t *> (timers_))->check_tag ()) {
        errno = EFAULT;
        return -1;
    }

    return (static_cast<zmq::timers_t *> (timers_))->reset (timer_id_);
}

long zmq_timers_timeout (timers_: *mut c_void)
{
    if (!timers_ || !(static_cast<zmq::timers_t *> (timers_))->check_tag ()) {
        errno = EFAULT;
        return -1;
    }

    return (static_cast<zmq::timers_t *> (timers_))->timeout ();
}

int zmq_timers_execute (timers_: *mut c_void)
{
    if (!timers_ || !(static_cast<zmq::timers_t *> (timers_))->check_tag ()) {
        errno = EFAULT;
        return -1;
    }

    return (static_cast<zmq::timers_t *> (timers_))->execute ();
}

//  The proxy functionality

int zmq_proxy (frontend_: *mut c_void, backend_: *mut c_void, capture_: *mut c_void)
{
    if (!frontend_ || !backend_) {
        errno = EFAULT;
        return -1;
    }
    return zmq::proxy (static_cast<zmq::ZmqSocketBase *> (frontend_),
                       static_cast<zmq::ZmqSocketBase *> (backend_),
                       static_cast<zmq::ZmqSocketBase *> (capture_));
}

int zmq_proxy_steerable (frontend_: *mut c_void,
                         backend_: *mut c_void,
                         capture_: *mut c_void,
                         control_: *mut c_void)
{
    if (!frontend_ || !backend_) {
        errno = EFAULT;
        return -1;
    }
    return zmq::proxy (static_cast<zmq::ZmqSocketBase *> (frontend_),
                       static_cast<zmq::ZmqSocketBase *> (backend_),
                       static_cast<zmq::ZmqSocketBase *> (capture_),
                       static_cast<zmq::ZmqSocketBase *> (control_));
}

//  The deprecated device functionality

int zmq_device (int /* type */, frontend_: *mut c_void, backend_: *mut c_void)
{
    return zmq::proxy (static_cast<zmq::ZmqSocketBase *> (frontend_),
                       static_cast<zmq::ZmqSocketBase *> (backend_), null_mut());
}

//  Probe library capabilities; for now, reports on transport and security

int zmq_has (capability_: *const c_char)
{
// #if defined(ZMQ_HAVE_IPC)
    if (strcmp (capability_, zmq::protocol_name::ipc) == 0)
        return true;
// #endif
// #if defined(ZMQ_HAVE_OPENPGM)
    if (strcmp (capability_, zmq::protocol_name::pgm) == 0)
        return true;
// #endif
// #if defined(ZMQ_HAVE_TIPC)
    if (strcmp (capability_, zmq::protocol_name::tipc) == 0)
        return true;
// #endif
// #if defined(ZMQ_HAVE_NORM)
    if (strcmp (capability_, zmq::protocol_name::norm) == 0)
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
    if (strcmp (capability_, zmq::protocol_name::vmci) == 0)
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