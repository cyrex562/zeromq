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

use crate::context::ZmqContext;
use crate::ctx_hdr::ZmqContext;
use crate::defines::{
    zmq_free_fn, zmq_timer_fn, ZmqMessage, ZMQ_EVENTS, ZMQ_FD, ZMQ_IO_THREADS, ZMQ_MORE, ZMQ_PAIR,
    ZMQ_PEER, ZMQ_POLLERR, ZMQ_POLLIN, ZMQ_POLLOUT, ZMQ_POLLPRI, ZMQ_SHARED, ZMQ_SNDMORE,
    ZMQ_SRCFD, ZMQ_TYPE, ZMQ_VERSION_MAJOR, ZMQ_VERSION_MINOR, ZMQ_VERSION_PATCH,
};
use crate::err::ZmqError::{
    AddItemToPollerFailed, AddTimerFailed, BindSocketFailed, CancelTimerFailed, CheckTagFailed,
    CloseMessageFailed, CloseSocketFailed, ConnectPeerSocketFailed, ConnectSocketFailed,
    DeserializeZmqPeerFailed, DeserializeZmqSocketBaseFailed, ExecuteTimerFailed,
    GetContextPropertyFailed, GetMessageFailed, GetSocketOptionFailed, GetSocketPeerStateFailed,
    InitializeMessageFailed, InvalidEvent, InvalidFileDescriptor, InvalidInput,
    InvalidMessageProperty, InvalidPeer, JoinGroupFailed, LeaveGroupFailed, MallocFailed,
    ModifyPollerItemFailed, PollFailed, PollerWaitFailed, ProxyFailed, RemoveItemFromPollerFailed,
    ResetTimerFailed, SelectFailed, SendMessageFailed, SerializeZmqSocketBaseFailed,
    SetContextPropertyFailed, SetMessagePropertyFailed, SetTimerIntervalFailed,
    ShutdownContextFailed, TerminateEndpointFailed, UnsupportedSocketType,
};
use crate::err::{errno_to_string, wsa_error_to_errno, ZmqError};
use crate::fd::ZmqFileDesc;
use crate::ip::{initialize_network, shutdown_network};
use crate::message::{ZmqMessage, ZMQ_MSG_MORE, ZMQ_MSG_SHARED};
use crate::options::ZmqOptions;
use crate::peer::ZmqPeer;
use crate::poll_item::ZmqPollItem;
use crate::poller_event::ZmqPollerEvent;
use crate::polling_util::{compute_timeout, OptimizedFdSet};
use crate::proxy::proxy;
use crate::socket_base::ZmqSocketBase;
use crate::socket_poller::ZmqSocketPoller;
use crate::timers::ZmqTimers;
use crate::utils::copy_bytes;
use anyhow::{anyhow, bail};
use bincode::options;
use libc::{
    atoi, c_char, c_long, c_void, clock_t, fd_set, iovec, poll, pollfd, pselect, select, sigset_t,
    suseconds_t, time_t, timespec, timeval, EFAULT, EINTR, EINVAL, ENOMEM, ENOTSOCK, ENOTSUP,
    INT_MAX, POLLIN, POLLOUT, POLLPRI,
};
use serde::Serialize;
use std::error::Error;
use std::ptr::null_mut;
use std::time::Duration;
use std::{mem, thread, time};
#[cfg(windows)]
use windows::Win32::Networking::WinSock::{
    select, WSAGetLastError, FD_SET, POLLIN, POLLOUT, POLLPRI, SOCKET_ERROR, TIMEVAL,
};

pub fn zmq_version(major_: *mut u32, minor_: *mut u32, patch_: *mut u32) {
    unsafe {
        *major_ = ZMQ_VERSION_MAJOR;
        *minor_ = ZMQ_VERSION_MINOR;
        *patch_ = ZMQ_VERSION_PATCH;
    }
}

// pub fn zmq_strerror (errnum_: i32) -> String
// {
//     return errno_to_string (errnum_).to_string();
// }

// pub fn zmq_errno () -> i32
// {
//     return errno;
// }

pub fn zmq_ctx_new() -> Result<Vec<u8>, ZmqError> {
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
    Ok(bincode::serialize(&ctx).expect("failed to serialize context"))
}

pub fn zmq_ctx_term(ctx_raw: &mut [u8]) -> Result<(), ZmqError> {
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

    ctx.terminate.map_err(|e| {
        shutdown_network();
        e
    })
}

pub fn zmq_ctx_shutdown(ctx_raw: &mut [u8]) -> Result<(), ZmqError> {
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
    match ctx.shutdown() {
        Ok(_) => Ok(()),
        Err(e) => ShutdownContextFailed(fmt!("failed to shutdown context: {}", e)),
    }
}

pub fn zmq_ctx_set(ctx_raw: &mut [u8], option_: i32, mut optval_: i32) -> Result<(), ZmqError> {
    return zmq_ctx_set_ext(ctx_raw, option_, optval_.to_le_bytes().as_mut_slice());
}

pub fn zmq_ctx_set_ext(ctx_raw: &mut [u8], option: i32, optval: &mut [u8]) -> Result<(), ZmqError> {
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

    match ctx.set(option, optval, optval.len()) {
        Ok(_) => Ok(()),
        Err(e) => SetContextPropertyFailed(fmt!("failed to set context property: {}", e)),
    }
}

pub fn zmq_ctx_get(ctx_raw: &mut [u8], opt_kind: i32) -> Result<i32, ZmqError> {
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
    match ctx.option_i32(opt_kind) {
        Ok(v) => Ok(v),
        Err(e) => GetContextPropertyFailed(fmt!("failed to get context property: {}", e)),
    }
}

pub fn zmq_ctx_get_ext(ctx_raw: &mut [u8], opt_kind: i32) -> Result<Vec<u8>, ZmqError> {
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
    match ctx.option_bytes(opt_kind) {
        Ok(v) => Ok(v),
        Err(e) => GetContextPropertyFailed(fmt!("failed to get context property: {}", e)),
    }
}

//  Stable/legacy context API

pub fn zmq_init(io_threads: i32) -> Result<Vec<u8>, ZmqError> {
    if io_threads >= 0 {
        let mut ctx_raw = zmq_ctx_new()?;
        zmq_ctx_set(ctx_raw.as_mut_slice(), ZMQ_IO_THREADS, io_threads)?;
        Ok(ctx_raw)
    }
    bail!("invalid io_threads {}", io_threads)
}

pub fn zmq_term(ctx: &mut [u8]) -> Result<(), ZmqError> {
    zmq_ctx_term(ctx)
}

pub fn zmq_ctx_destroy(ctx: &mut [u8]) -> Result<(), ZmqError> {
    zmq_ctx_term(ctx)
}

// Sockets

pub fn as_socket_base(in_bytes: &[u8]) -> Result<ZmqSocketBase, ZmqError> {
    // ZmqSocketBase *s = static_cast<ZmqSocketBase *> (s_);
    // let mut s: *mut ZmqSocketBase = s_ as *mut ZmqSocketBase;
    // if s_.is_null() || !s.check_tag () {
    //     errno = ENOTSOCK;
    //     return null_mut();
    // }
    // return s;
    let mut out: ZmqSocketBase = bincode::deserialize(in_bytes)?;
    if out.check_tag() == false {
        return Err(DeserializeZmqSocketBaseFailed(fmt!("ENOTSOCK")));
    }
    Ok(out)
}

pub fn as_zmq_peer(in_bytes: &[u8]) -> Result<ZmqPeer, ZmqError> {
    let mut out: ZmqPeer = bincode::deserialize(in_bytes)?;
    if out.check_tag() == false {
        return Err(DeserializeZmqPeerFailed(fmt!("ENOTSOCK")));
    }
    Ok(out)
}

pub fn zmq_socket(ctx: &mut [u8], type_: i32) -> Result<Vec<u8>, ZmqError> {
    let mut ctx: ZmqContext = bincode::deserialize(ctx)?;
    if ctx.check_tag() == false {
        return Err(CheckTagFailed("check tag failed".to_string()));
    }
    // if !ctx || !(ctx as *mut ZmqContext).check_tag() {
    //     errno = EFAULT;
    //     return null_mut();
    // }
    // let mut ctx: *mut ZmqContext = ctx as *mut ZmqContext;
    // let mut s: *mut ZmqSocketBase = ctx.create_socket(type_);
    let s: ZmqSocketBase = ctx.create_socket(type_).unwrap();
    match bincode::serialize(&s) {
        Ok(v) => Ok(v),
        Err(e) => Err(SerializeZmqSocketBaseFailed(fmt!(
            "failed to serialize socket: {}",
            e
        ))),
    }
}

pub fn zmq_close(s_: &mut [u8]) -> Result<(), ZmqError> {
    let mut s: ZmqSocketBase = as_socket_base(s_)?;
    match s.close() {
        Ok(_) => Ok(()),
        Err(e) => Err(CloseSocketFailed(fmt!("failed to close socket: {}", e))),
    }
}

pub fn zmq_setsockopt(
    options: &mut ZmqOptions,
    in_bytes: &[u8],
    opt_kind: i32,
    opt_val: &[u8],
    opt_val_len: usize,
) -> anyhow::Result<()> {
    let mut s: ZmqSocketBase = as_socket_base(in_bytes)?;
    s.setsockopt(options, opt_kind, opt_val, opt_val_len)
}

pub fn zmq_getsockopt(
    options: &mut ZmqOptions,
    in_bytes: &[u8],
    opt_kind: u8,
    opt_val: &mut [u8],
    opt_val_len: *mut usize,
) -> Result<Vec<u8>, ZmqError> {
    let mut s: ZmqSocketBase = as_socket_base(in_bytes)?;
    match s.getsockopt(options, opt_kind as i32) {
        Ok(v) => Ok(v),
        Err(e) => Err(GetSocketOptionFailed(fmt!(
            "failed to get socket option: {}",
            e
        ))),
    }
}

pub fn zmq_socket_monitor_versioned(
    options: &mut ZmqOptions,
    s_: &mut [u8],
    addr_: &str,
    events_: u64,
    event_version_: i32,
    type_: i32,
) -> Result<(), ZmqError> {
    let mut s: ZmqSocketBase = as_socket_base(s_)?;

    if s.monitor(options, addr_, events_, event_version_, type_)
        .is_ok()
    {
        Ok(())
    }
    bail!("monitor failed")
}

pub fn zmq_socket_monitor(
    options: &mut ZmqOptions,
    s_: &mut [u8],
    addr_: &str,
    events_: u64,
) -> Result<(), ZmqError> {
    zmq_socket_monitor_versioned(options, s_, addr_, events_, 1, ZMQ_PAIR)
}

pub fn zmq_join(s_: &mut [u8], group_: &str) -> Result<(), ZmqError> {
    let mut s: ZmqSocketBase = as_socket_base(s_)?;

    match s.join(group_) {
        Ok(_) => Ok(()),
        Err(e) => Err(JoinGroupFailed(fmt!("failed to join group: {}", e))),
    }
}

pub fn zmq_leave(s_: &mut [u8], group_: &str) -> Result<(), ZmqError> {
    let mut s: ZmqSocketBase = as_socket_base(s_)?;

    match s.leave(group_) {
        Ok(_) => Ok(()),
        Err(e) => Err(LeaveGroupFailed(fmt!("failed to leave group: {}", e))),
    }
}

pub fn zmq_bind(
    ctx: &mut ZmqContext,
    options: &mut ZmqOptions,
    s_: &mut [u8],
    addr_: &str,
) -> Result<(), ZmqError> {
    let mut s: ZmqSocketBase = as_socket_base(s_)?;

    match s.bind(ctx, options, addr_) {
        Ok(_) => Ok(()),
        Err(e) => Err(BindSocketFailed(fmt!("failed to bind socket: {}", e))),
    }
}

pub fn zmq_connect(options: &mut ZmqOptions, s_: &mut [u8], addr_: &str) -> Result<(), ZmqError> {
    let mut s: ZmqSocketBase = as_socket_base(s_)?;
    match s.connect(options, addr_) {
        Ok(_) => Ok(()),
        Err(e) => Err(ConnectSocketFailed(fmt!("failed to connect socket: {}", e))),
    }
}

pub fn zmq_connect_peer(s_: &mut [u8], addr_: &str) -> Result<(), ZmqError> {
    let mut s: ZmqPeer = as_zmq_peer(s_)?;

    let mut socket_type: i32 = 0i32;
    let mut socket_type_size = mem::sizeof::<socket_type>();
    if s.getsockopt(ZMQ_TYPE, &socket_type, &socket_type_size)
        .is_err()
    {
        return Err(GetSocketOptionFailed(
            "failed to get socket option".to_string(),
        ));
    }
    if socket_type != ZMQ_PEER {
        return Err(UnsupportedSocketType("unsupported socket type".to_string()));
    }

    match s.connect_peer(addr_) {
        Ok(_) => Ok(()),
        Err(e) => Err(ConnectPeerSocketFailed(fmt!(
            "failed to connect peer socket: {}",
            e
        ))),
    }
}

pub fn zmq_unbind(options: &mut ZmqOptions, s_: &mut [u8], addr_: &str) -> Result<(), ZmqError> {
    let mut s: ZmqSocketBase = as_socket_base(s_)?;
    match s.term_endpoint(options, addr_) {
        Ok(_) => Ok(()),
        Err(e) => Err(TerminateEndpointFailed(fmt!(
            "failed to terminate socket endpoint: {}",
            e
        ))),
    }
}

pub fn zmq_disconnect(
    options: &mut ZmqOptions,
    s_: &mut [u8],
    addr_: &str,
) -> Result<(), ZmqError> {
    let mut s: ZmqSocketBase = as_socket_base(s_)?;
    match s.term_endpoint(options, addr_) {
        Ok(_) => Ok(()),
        Err(e) => Err(TerminateEndpointFailed(fmt!(
            "failed to terminate socket endpoint: {}",
            e
        ))),
    }
}

// Sending functions.

pub fn s_sendmsg(
    options: &mut ZmqOptions,
    s_: &mut ZmqSocketBase,
    msg: &mut ZmqMessage,
    flags: i32,
) -> Result<i32, ZmqError> {
    let mut sz: usize = zmq_msg_size(msg);
    s_.send(msg, options, flags)?;

    //  This is what I'd like to do, my C+= 1 fu is too weak -- PH 2016/02/09
    //  int max_msgsz = s_->parent->get (ZMQ_MAX_MSGSZ);
    let max_msgsz: usize = i32::MAX as usize;

    //  Truncate returned size to INT_MAX to avoid overflow to negative values
    Ok(if sz < max_msgsz { sz } else { max_msgsz } as i32)
}

//   To be deprecated once zmq_msg_send() is stable
pub fn zmq_sendmsg(
    options: &mut ZmqOptions,
    s_: &mut [u8],
    msg: &mut ZmqMessage,
    flags: i32,
) -> Result<i32, ZmqError> {
    zmq_msg_send(options, msg, s_, flags)
}

pub fn zmq_send(
    options: &mut ZmqOptions,
    s_: &mut [u8],
    buf: &mut [u8],
    len_: usize,
    flags: i32,
) -> Result<(), ZmqError> {
    let mut s: ZmqSocketBase = as_socket_base(s_)?;
    let mut msg: ZmqMessage = ZmqMessage::default();
    zmq_msg_init_buffer(&mut msg, buf, len_)?;
    match s_sendmsg(options, &mut s, &mut msg, flags) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn zmq_send_const(
    options: &mut ZmqOptions,
    s_: &mut [u8],
    buf: &mut [u8],
    len_: usize,
    flags: i32,
) -> Result<(), ZmqError> {
    let mut s = as_socket_base(s_)?;
    let mut msg: ZmqMessage = ZmqMessage::default();
    zmq_msg_init_data(&mut msg, buf as &mut [u8], len_, None)?;
    match s_sendmsg(options, &mut s, &mut msg, flags) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

// Send multiple messages.
// TODO: this function has no man page
//
// If flag bit ZMQ_SNDMORE is set the vector is treated as
// a single multi-part message, i.e. the last message has
// ZMQ_SNDMORE bit switched off.
//
pub fn zmq_sendiov(
    options: &mut ZmqOptions,
    s_: &mut [u8],
    a_: &mut iovec,
    count: usize,
    mut flags: i32,
) -> Result<(), ZmqError> {
    let mut s: ZmqSocketBase = as_socket_base(s_)?;

    // let mut rc = 0;
    let mut msg = ZmqMessage::default();

    // for (size_t i = 0; i < count; += 1i)
    for i in 0..count {
        zmq_msg_init_size(&mut msg, a_[i].iov_len)?;
        copy_bytes(msg.data_mut(), 0, a_[i].iov_base, 0, a_[i].iov_len);

        if i == count - 1 {
            flags = flags & !ZMQ_SNDMORE;
        }
        rc = s_sendmsg(options, &mut s, &mut msg, flags).map_err(|e| {
            zmq_msg_close(&mut msg)?;
            e
        })?;
    }
    Ok(())
}

// Receiving functions.

pub fn s_recvmsg(
    options: &mut ZmqOptions,
    s_: &mut ZmqSocketBase,
    msg: &mut ZmqMessage,
    flags: i32,
) -> Result<usize, ZmqError> {
    match s_.recv(msg, options, flags) {
        Ok(_) => (),
        Err(e) => {
            return Err(ReceiveMessageFailed(fmt!(
                "failed to receive message: {}",
                e
            )));
        }
    }

    //  Truncate returned size to INT_MAX to avoid overflow to negative values
    let sz = zmq_msg_size(msg);
    return if sz < usize::MAX {
        Ok(sz)
    } else {
        Ok(usize::MAX)
    };
}

//   To be deprecated once zmq_msg_recv() is stable
pub fn zmq_recvmsg(
    options: &mut ZmqOptions,
    s_: &mut [u8],
    msg: &mut ZmqMessage,
    flags: i32,
) -> Result<usize, ZmqError> {
    return zmq_msg_recv(options, msg, s_, flags);
}

pub fn zmq_recv(s_: &mut [u8], buf: &mut [u8], len: usize, flags: i32) -> Result<usize, ZmqError> {
    let mut s: ZmqSocketBase = as_socket_base(s_)?;

    let mut msg: ZmqMessage = ZmqMessage::default();
    zmq_msg_init(&mut msg)?;

    let nbytes = s_recvmsg(options: &mut ZmqOptions, &mut s, &mut msg, flags).map_err(|e| {
        zmq_msg_close(&mut msg)?;
        e
    })?;

    //  An oversized message is silently truncated.
    let to_copy: usize = if (nbytes as usize) < len { nbytes } else { len };

    //  We explicitly allow a null buffer argument if len is zero
    if to_copy {
        // assert (buf);
        copy_bytes(buf, 0, zmq_msg_data(&msg), 0, to_copy);
    }
    zmq_msg_close(&mut msg)?;
    Ok(nbytes)
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
pub fn zmq_recviov(
    options: &mut ZmqOptions,
    s_: &mut [u8],
    a_: &mut [iovec],
    count: &mut usize,
    flags: i32,
) -> Result<i32, ZmqError> {
    let mut s: ZmqSocketBase = as_socket_base(s_)?;
    let mut nread = 0;
    let mut recvmore = true;

    *count = 0;

    // for (size_t i = 0; recvmore && i < count; += 1i)
    for i in 0..recvmore {
        let mut msg = ZmqMessage::default();
        let mut rc = zmq_msg_init(&mut msg);
        // errno_assert (rc == 0);

        let nbytes = s_recvmsg(options, &mut s, &mut msg, flags).map_err(|e| {
            zmq_msg_close(&mut msg)?;
            e
        })?;

        a_[i].iov_len = zmq_msg_size(&msg);
        unsafe {
            a_[i].iov_base = libc::malloc(a_[i].iov_len);
        }
        if a_[i].iov_base == null_mut() {
            return Err(MallocFailed(
                "failed to allocate memory for iov_base".to_string(),
            ));
        }

        unsafe {
            libc::memcpy(
                a_[i].iov_base,
                zmq_msg_data(&msg) as *const c_void,
                a_[i].iov_len,
            );
        }
        // Assume zmq_socket ZMQ_RVCMORE is properly set.
        let p_msg = &mut msg;
        recvmore = p_msg.flag_set(ZMQ_MSG_MORE);
        zmq_msg_close(&mut msg)?;
        // errno_assert (rc == 0);
        *count += 1;
        nread += 1;
    }
    Ok(nread)
}

// Message manipulators.

pub fn zmq_msg_init(msg: &mut ZmqMessage) -> Result<(), ZmqError> {
    match msg.init2() {
        Ok(_) => Ok(()),
        Err(e) => Err(InitializeMessageFailed(fmt!("zmq_msg_init failed: {}", e))),
    }
}

pub fn zmq_msg_init_size(msg: &mut ZmqMessage, size: usize) -> Result<(), ZmqError> {
    match msg.init_size(size) {
        Ok(_) => Ok(()),
        Err(e) => Err(InitializeMessageFailed(fmt!(
            "zmq_msg_init_size failed: {}",
            e
        ))),
    }
}

pub fn zmq_msg_init_buffer(
    msg: &mut ZmqMessage,
    buf: &mut [u8],
    size: usize,
) -> Result<(), ZmqError> {
    match msg.init_buffer(buf, size) {
        Ok(_) => Ok(()),
        Err(e) => Err(InitializeMessageFailed(fmt!(
            "zmq_msg_init_buffer failed: {}",
            e
        ))),
    }
}

pub fn zmq_msg_init_data(
    msg: &mut ZmqMessage,
    data: &mut [u8],
    size: usize,
    hint: Option<&mut [u8]>,
) -> Result<(), ZmqError> {
    match msg.init_data(data, size, hint) {
        Ok(_) => Ok(()),
        Err(e) => Err(InitializeMessageFailed(fmt!(
            "zmq_msg_init_data failed: {}",
            e
        ))),
    }
}

pub fn zmq_msg_send(
    options: &mut ZmqOptions,
    msg: &mut ZmqMessage,
    s_: &mut [u8],
    flags: i32,
) -> Result<i32, ZmqError> {
    let mut s = as_socket_base(s_)?;
    s_sendmsg(options, &mut s, msg, flags)
}

pub fn zmq_msg_recv(
    options: &mut ZmqOptions,
    msg: &mut ZmqMessage,
    s_: &mut [u8],
    flags: i32,
) -> Result<usize, ZmqError> {
    let mut s: ZmqSocketBase = as_socket_base(s_)?;
    return s_recvmsg(options, &mut s, msg, flags);
}

pub fn zmq_msg_close(msg: &mut ZmqMessage) -> Result<(), ZmqError> {
    match msg.close() {
        Ok(_) => Ok(()),
        Err(e) => Err(CloseMessageFailed(fmt!("zmq_msg_close failed: {}", e))),
    }
}

// pub fn zmq_msg_move (dest_: *mut ZmqMessage, src_: *mut ZmqMessage)
// {
//     todo!()
//     // TODO: convert raw message to ZmqMessage and move from source to dest
//     // return (dest_ as *mut ZmqMessage).move(src_ as *mut ZmqMessage);
// }

// pub fn zmq_msg_copy (dest_: &mut ZmqMessage, src_: &mut ZmqMessage) -> i32
// {
//     // return (reinterpret_cast<ZmqMessage *> (dest_))
//     //   ->copy (*reinterpret_cast<ZmqMessage *> (src_));
//     dest_ = src_;
//     return 0;
// }

// pub fn zmq_msg_data (msg: &mut ZmqMessage) -> Vec<u8>
// {
//     // return (msg as *mut zmq_ZmqMessage).data ();
//     msg.data()
// }

// pub fn zmq_msg_size (msg: &ZmqMessage) -> usize
// {
//     msg.size()
// }

pub fn zmq_msg_more(msg: &mut ZmqMessage) -> Result<i32, ZmqError> {
    zmq_msg_get(msg, ZMQ_MORE as i32)
}

pub fn zmq_msg_get(msg: &mut ZmqMessage, property_: i32) -> Result<i32, ZmqError> {
    let mut fd_string = String::new();

    return match property_ {
        ZMQ_MORE => {
            if msg.flags() & ZMQ_MSG_MORE != 0 {
                Ok(1)
            } else {
                Ok(0)
            }
        }
        ZMQ_SRCFD => {
            fd_string = zmq_msg_gets(msg, "__fd")?;
            if fd_string == null_mut() {
                return Err(GetMessageFailed("failed to get message".to_string()));
            }
            Ok(i32::from_str_radix(&fd_string, 10).unwrap())
        }
        ZMQ_SHARED => {
            if msg.is_cmsg() || msg.flag_set(ZMQ_MSG_SHARED) {
                Ok(1)
            } else {
                Ok(0)
            }
        }
        _ => Err(InvalidMessageProperty(
            "invalid message property".to_string(),
        )),
    };
}

pub fn zmq_msg_set(msg: &mut ZmqMessage, a: i32, b: i32) -> Result<(), ZmqError> {
    //  No properties supported at present
    unimplemented!()
}

pub fn zmq_msg_set_routing_id(msg: &mut ZmqMessage, routing_id: u32) -> Result<(), ZmqError> {
    match msg.set_routing_id(routing_id) {
        Ok(_) => Ok(()),
        Err(e) => Err(SetMessagePropertyFailed(fmt!(
            "zmq_msg_set_routing_id failed: {}",
            e
        ))),
    }
}

pub fn zmq_msg_routing_id(msg: &mut ZmqMessage) -> u32 {
    return msg.routing_id;
}

pub fn zmq_msg_set_group(msg: &mut ZmqMessage, group_: &str) -> Result<(), ZmqError> {
    match msg.set_group(group_) {
        Ok(_) => Ok(()),
        Err(e) => Err(SetMessagePropertyFailed(fmt!(
            "zmq_msg_set_group failed: {}",
            e
        ))),
    }
}

pub fn zmq_msg_group(msg: &mut ZmqMessage) -> String {
    return msg.group.clone();
}

//  Get message metadata string

pub fn zmq_msg_gets(msg: &mut ZmqMessage, property_: &str) -> Result<String, ZmqError> {
    let metadata = msg.metadata.clone();
    if metadata.is_none() {
        return Err(GetMessageFailed(
            "failed to get message metadata".to_string(),
        ));
    }

    let value = metadata.unwrap().get(property_);
    if value.is_none() {
        return Err(GetMessageFailed(
            "failed to get message metadata".to_string(),
        ));
    }
    return Ok(value.unwrap().clone());
}

// Polling.

// #if defined ZMQ_HAVE_POLLER
pub fn zmq_poller_poll(items: &mut [ZmqPollItem], timeout: i32) -> Result<i32, ZmqError> {
    // implement zmq_poll on top of zmq_poller
    let mut events: ZmqPollerEvent = ZmqPollerEvent::new(items.len());
    let mut poller: ZmqSocketPoller = ZmqSocketPoller::default();
    let mut repeat_items = false;
    //  Register sockets with poller
    for i in 0..items.len() {
        items[i].revents = 0;

        let mut modify = false;
        let mut e = items[i].events;
        if items[i].socket {
            //  Poll item is a 0MQ socket.
            for j in 0..i {
                // Check for repeat entries
                if items_[j].socket == items_[i].socket {
                    repeat_items = true;
                    modify = true;
                    e |= items[j].events;
                }
            }
            if modify {
                zmq_poller_modify(&mut poller, items[i].socket, e)?;
            } else {
                zmq_poller_add(&mut poller, items[i].socket, None, e)?;
            }
        } else {
            //  Poll item is a raw file descriptor.
            for j in 0..i {
                // Check for repeat entries
                if !items[j].socket && items[j].fd == items[i].fd {
                    repeat_items = true;
                    modify = true;
                    e |= items[j].events;
                }
            }
            if (modify) {
                zmq_poller_modify_fd(&mut poller, items[i].fd, e)?;
            } else {
                zmq_poller_add_fd(&mut poller, items[i].fd, None, e)?;
            }
        }
    }

    //  Wait for events
    let found_events = zmq_poller_wait_all(&mut poller, &mut events, items.len(), timeout)?;

    //  Transform poller events into zmq_pollitem events.
    //  items_ contains all items, while events only contains fired events.
    //  If no sockets are repeated (likely), the two are still co-ordered, so step through the items
    //  checking for matches only on the first event.
    //  If there are repeat items, they cannot be assumed to be co-ordered,
    //  so each pollitem must check fired events from the beginning.
    let mut j_start = 0;
    for i in 0..items.len() {
        // for (int j = j_start; j < found_events; += 1j)
        for j in j_start..found_events {
            if (items[i].socket && items[i].socket == events[j].socket)
                || (!(items[i].socket || &events[j].socket) && items[i].fd == events[j].fd)
            {
                items[i].revents = &events[j].events & items[i].events;
                if !repeat_items {
                    // no repeats, we can ignore events we've already seen
                    j_start += 1;
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
    // delete[] events;
    // return rc;
    Ok(0)
}
// #endif // ZMQ_HAVE_POLLER

pub fn zmq_poll(
    options: &mut ZmqOptions,
    items_: &mut [ZmqPollItem],
    timeout: i32,
) -> Result<i32, ZmqError> {
    // #if defined ZMQ_HAVE_POLLER
    // if poller is present, use that if there is at least 1 thread-safe socket,
    // otherwise fall back to the previous implementation as it's faster.
    for i in 0..items_.len() {
        if (items_[i].socket) {
            let mut s: *mut ZmqSocketBase = as_socket_base_t(items_[i].socket);
            if (s) {
                if (s.is_thread_safe()) {
                    return zmq_poller_poll(items_, timeout);
                }
            } else {
                //as_socket_base_t returned null_mut() : socket is invalid
                return Err(PollFailed("invalid socket".to_string()));
            }
        }
    }
    // #endif // ZMQ_HAVE_POLLER
    // #if defined ZMQ_POLL_BASED_ON_POLL || defined ZMQ_POLL_BASED_ON_SELECT
    if (items_.len() < 0) {
        return Err(PollFailed("invalid number of items".to_string()));
    }
    if (items_.len() == 0) {
        if (timeout == 0) {
            return Ok(0);
        }
        thread::sleep(time::Duration::from_millis(timeout as u64));
    }

    // clock_t clock;
    // u64 now = 0;
    // u64 end = 0;
    // #if defined ZMQ_POLL_BASED_ON_POLL
    //     fast_vector_t<pollfd, ZMQ_POLLITEMS_DFLT> pollfds (nitems_);
    let mut pollfds: Vec<pollfd> = Vec::with_capacity(items_.len());

    //  Build pollset for poll () system call.
    for i in 0..items_.len() {
        //  If the poll item is a 0MQ socket, we poll on the file descriptor
        //  retrieved by the ZMQ_FD socket option.
        if (items_[i].socket) {
            let mut zmq_fd_size = sizeof::<ZmqFileDesc>();
            zmq_getsockopt(
                options,
                items_[i].socket,
                ZMQ_FD,
                &mut pollfds[i].fd as &mut [u8],
                &mut zmq_fd_size,
            )?;

            pollfds[i].events = if items_[i].events { POLLIN } else { 0 };
        }
        //  Else, the poll item is a raw file descriptor. Just convert the
        //  events to normal POLLIN/POLLOUT for poll ().
        else {
            pollfds[i].fd = items_[i].fd;
            pollfds[i].events = (if items_[i].events & ZMQ_POLLIN == 1 {
                POLLIN
            } else {
                0
            }) | (if items_[i].events & ZMQ_POLLOUT == 1 {
                POLLOUT
            } else {
                0
            }) | (if items_[i].events & ZMQ_POLLPRI == 1 {
                POLLPRI
            } else {
                0
            });
        }
    }
    // #else
    //  Ensure we do not attempt to select () on more than FD_SETSIZE
    //  file descriptors.
    //  TODO since this function is called by a client, we could return errno EINVAL/ENOMEM/... here
    // zmq_assert (nitems_ <= FD_SETSIZE);

    let mut pollset_in: Vec<fd_set> = Vec::with_capacity(items_.len());
    let mut pollset_out: Vec<fd_set> = Vec::with_capacity(items_.len());
    let mut pollset_err: Vec<fd_set> = Vec::with_capacity(items_.len());

    let mut maxfd: ZmqFileDesc = 0;

    //  Build the fd_sets for passing to select ().
    for i in 0..items_.len() {
        //  If the poll item is a 0MQ socket we are interested in input on the
        //  notification file descriptor retrieved by the ZMQ_FD socket option.
        if (items_[i].socket) {
            let zmq_fd_size = sizeof::<ZmqFileDesc>();
            let mut notify_fd: ZmqFileDesc = 0;
            zmq_getsockopt(
                options,
                items_[i].socket,
                ZMQ_FD,
                &mut notify_fd as &mut [u8],
                &mut zmq_fd_size,
            )?;
            if (items_[i].events) {
                // FD_SET (notify_fd, pollset_in.get ());
                pollset_in.push(notify_fd as fd_set);
                if (maxfd < notify_fd) {
                    maxfd = notify_fd;
                }
            }
        }
        //  Else, the poll item is a raw file descriptor. Convert the poll item
        //  events to the appropriate fd_sets.
        else {
            if (items_[i].events & ZMQ_POLLIN) {
                // FD_SET(items_[i].fd, pollset_in.get());
                pollset_in.push(items_[i].fd as fd_set);
            }
            if (items_[i].events & ZMQ_POLLOUT) {
                // FD_SET(items_[i].fd, pollset_out.get());
                pollset_out.push(items_[i].fd as fd_set);
            }
            if (items_[i].events & ZMQ_POLLERR) {
                // FD_SET(items_[i].fd, pollset_err.get());
                pollset_err.push(items_[i].fd as fd_set);
            }
            if (maxfd < items_[i].fd) {
                maxfd = items_[i].fd;
            }
        }
    }

    // OptimizedFdSet inset (nitems_);
    let mut inset: Vec<fd_set> = Vec::with_capacity(items_.len());
    // OptimizedFdSet outset (nitems_);
    let mut outset: Vec<fd_set> = Vec::with_capacity(items_.len());
    // OptimizedFdSet errset (nitems_);
    let mut errset: Vec<fd_set> = Vec::with_capacity(items_.len());
    // #endif

    let mut first_pass = true;
    let mut nevents = 0;

    loop {
        // #if defined ZMQ_POLL_BASED_ON_POLL

        //  Compute the timeout for the subsequent poll.
        let timeout = compute_timeout(first_pass, timeout, now, end);

        //  Wait for events.
        unsafe {
            {
                let rc: i32 = poll(&mut pollfds[0], nitems_, timeout);
                if rc == -1 && errno == EINTR {
                    return Err(PollFailed("call to poll failed".to_string()));
                }
                // errno_assert (rc >= 0);
            }
        }
        //  Check for the events.
        // for (int i = 0; i != nitems_; i+= 1)
        for i in 0..items_.len() {
            items_[i].revents = 0;

            //  The poll item is a 0MQ socket. Retrieve pending events
            //  using the ZMQ_EVENTS socket option.
            if items_[i].socket {
                let mut zmq_events_size = mem::size_of::<u32>();
                let mut zmq_events = 0;
                zmq_getsockopt(
                    options,
                    items_[i].socket,
                    ZMQ_EVENTS,
                    &mut zmq_events as &mut [u8],
                    &mut zmq_events_size,
                )?;
                if (items_[i].events & ZMQ_POLLOUT == 1) && (zmq_events & ZMQ_POLLOUT == 1) {
                    items_[i].revents |= ZMQ_POLLOUT;
                }
                if (items_[i].events & ZMQ_POLLIN == 1) && (zmq_events & ZMQ_POLLIN == 1) {
                    items_[i].revents |= ZMQ_POLLIN;
                }
            }
            //  Else, the poll item is a raw file descriptor, simply convert
            //  the events to ZmqPollItem-style format.
            else {
                if pollfds[i].revents & POLLIN {
                    items_[i].revents |= ZMQ_POLLIN;
                }
                if pollfds[i].revents & POLLOUT {
                    items_[i].revents |= ZMQ_POLLOUT;
                }
                if pollfds[i].revents & POLLPRI {
                    items_[i].revents |= ZMQ_POLLPRI;
                }
                if pollfds[i].revents & !(POLLIN | POLLOUT | POLLPRI) {
                    items_[i].revents |= ZMQ_POLLERR;
                }
            }

            if items_[i].revents {
                nevents += 1;
            }
        }

        // #else

        //  Compute the timeout for the subsequent poll.
        let mut timeout: timeval = timeval {
            tv_sec: 0,
            tv_usec: 0,
        };
        let mut ptimeout: *mut timeval = &mut timeval;
        if (first_pass) {
            timeout.tv_sec = 0;
            timeout.tv_usec = 0;
            ptimeout = &mut timeout;
        } else if (timeout < 0) {
            ptimeout = null_mut();
        } else {
            timeout.tv_sec = ((end - now) / 1000);
            timeout.tv_usec = ((end - now) % 1000 * 1000);
            ptimeout = &mut timeout;
        }

        //  Wait for events. Ignore interrupts if there's infinite timeout.
        unsafe {
            loop {
                // TODO
                // memcpy (inset.get (), pollset_in.get (),
                //         valid_pollset_bytes (*pollset_in.get ()));

                // TODO
                // memcpy (outset.get (), pollset_out.get (),
                //         valid_pollset_bytes (*pollset_out.get ()));

                // TODO
                // memcpy (errset.get (), pollset_err.get (),
                //         valid_pollset_bytes (*pollset_err.get ()));
                // #if defined ZMQ_HAVE_WINDOWS
                #[cfg(target_os = "windows")]
                {
                    let rc = select(
                        0,
                        Some(&mut inset[0]),
                        Some(&mut outset[0]),
                        Some(&mut errset[0]),
                        Some(ptimeout as *const TIMEVAL),
                    );
                    if (rc == SOCKET_ERROR) {
                        errno = wsa_error_to_errno(WSAGetLastError());
                        wsa_assert(errno == ENOTSOCK);
                        return Err(PollFailed("call to select failed".to_string()));
                    }
                }
                // #else
                // TODO
                #[cfg(target_os = "linux")]
                {
                    let mut rc = 0i32;
                    unsafe {
                        rc = select(
                            maxfd + 1,
                            inset.unwrap().pop(),
                            outset.unwrap().pop(),
                            errset.unwrap().pop(),
                            ptimeout,
                        )
                    };
                    if rc == -1 {
                        // errno_assert (errno == EINTR || errno == EBADF);
                        return Err(PollFailed("call to select failed".to_string()));
                    }
                }
                // #endi
                break;
            }
        }

        //  Check for the events.

        for i in 0..items_.len() {
            items_[i].revents = 0;

            //  The poll item is a 0MQ socket. Retrieve pending events
            //  using the ZMQ_EVENTS socket option.
            if items_[i].socket {
                let mut zmq_events_size = mem::size_of::<u32>();
                let mut zmq_events = 0u32;
                zmq_getsockopt(
                    options,
                    items_[i].socket,
                    ZMQ_EVENTS,
                    &mut zmq_events as &mut [u8],
                    &mut zmq_events_size,
                )?;
                if (items_[i].events & ZMQ_POLLOUT == 1) && (zmq_events & ZMQ_POLLOUT == 1) {
                    items_[i].revents |= ZMQ_POLLOUT;
                }
                if (items_[i].events & ZMQ_POLLIN == 1) && (zmq_events & ZMQ_POLLIN == 1) {
                    items_[i].revents |= ZMQ_POLLIN;
                }
            }
            //  Else, the poll item is a raw file descriptor, simply convert
            //  the events to ZmqPollItem-style format.
            else {
                if FD_ISSET(items_[i].fd, inset.pop()) {
                    items_[i].revents |= ZMQ_POLLIN;
                }
                if FD_ISSET(items_[i].fd, outset.pop()) {
                    items_[i].revents |= ZMQ_POLLOUT;
                }
                if FD_ISSET(items_[i].fd, errset.pop()) {
                    items_[i].revents |= ZMQ_POLLERR;
                }
            }

            if (items_[i].revents) {
                nevents += 1;
            }
        }
        // #endif

        //  If timeout is zero, exit immediately whether there are events or not.
        if (timeout == 0) {
            break;
        }

        //  If there are events to return, we can exit immediately.
        if (nevents) {
            break;
        }

        //  At this point we are meant to wait for events but there are none.
        //  If timeout is infinite we can just loop until we get some events.
        if (timeout < 0) {
            if (first_pass) {
                first_pass = false;
            }
            continue;
        }

        //  The timeout is finite and there are no events. In the first pass
        //  we get a timestamp of when the polling have begun. (We assume that
        //  first pass have taken negligible time). We also compute the time
        //  when the polling should time out.
        if (first_pass) {
            now = clock.now_ms();
            end = now + timeout;
            if (now == end) {
                break;
            }
            first_pass = false;
            continue;
        }

        //  Find out whether timeout have expired.
        now = clock.now_ms();
        if (now >= end) {
            break;
        }
    }

    return Ok(nevents);
    // #else
    //  Exotic platforms that support neither poll() nor select().
    // errno = ENOTSUP;
    // return -1;
    // #endif
}

// #ifdef ZMQ_HAVE_PPOLL
// return values of 0 or -1 should be returned from zmq_poll; return value 1 means items passed checks
pub fn zmq_poll_check_items_(items_: &mut [ZmqPollItem], timeout: i32) -> Result<i32, ZmqError> {
    if items_.len() == 0 {
        if timeout == 0 {
            return Ok(0);
        }
        thread::sleep(Duration::from_millis(timeout as u64));
        return Ok(0);
    }
    return Ok(1);
}

struct zmq_poll_select_fds_t_ {
    pub pollset_in: Vec<fd_set>,
    pub pollset_out: Vec<fd_set>,
    pub pollset_err: Vec<fd_set>,
    pub inset: Vec<fd_set>,
    pub outset: Vec<fd_set>,
    pub errset: Vec<fd_set>,
    pub maxfd: ZmqFileDesc,
}

impl zmq_poll_select_fds_t_ {
    pub fn new(nitems: usize) -> Self {
        Self {
            pollset_in: Vec::with_capacity(nitems),
            pollset_out: Vec::with_capacity(nitems),
            pollset_err: Vec::with_capacity(nitems),
            inset: Vec::with_capacity(nitems),
            outset: Vec::with_capacity(nitems),
            errset: Vec::with_capacity(nitems),
            maxfd: 0,
        }
    }
}

pub fn zmq_poll_build_select_fds_(
    options: &mut ZmqOptions,
    items: &mut [ZmqPollItem],
    rc: &mut i32,
) -> zmq_poll_select_fds_t_ {
    //  Ensure we do not attempt to select () on more than FD_SETSIZE
    //  file descriptors.
    //  TODO since this function is called by a client, we could return errno EINVAL/ENOMEM/... here
    // zmq_assert (nitems_ <= FD_SETSIZE);

    // zmq_poll_select_fds_t_ fds (nitems_);
    let mut fds = zmq_poll_select_fds_t_::new(items.len());

    //  Build the fd_sets for passing to select ().
    // for (int i = 0; i != nitems_; i+= 1)
    for i in 0..items_.len() {
        //  If the poll item is a 0MQ socket we are interested in input on the
        //  notification file descriptor retrieved by the ZMQ_FD socket option.
        if (items_[i].socket) {
            let mut zmq_fd_size = sizeof::<ZmqFileDesc>();
            let mut notify_fd: ZmqFileDesc = 0;
            zmq_getsockopt(
                options,
                items_[i].socket,
                ZMQ_FD,
                &mut notify_fd as &mut [u8],
                &mut zmq_fd_size,
            )?;
            if items_[i].events {
                FD_SET(notify_fd, fds.pollset_in.pop());
                if (fds.maxfd < notify_fd) {
                    fds.maxfd = notify_fd;
                }
            }
        }
        //  Else, the poll item is a raw file descriptor. Convert the poll item
        //  events to the appropriate fd_sets.
        else {
            if (items_[i].events & ZMQ_POLLIN) {
                FD_SET(items_[i].fd, fds.pollset_in.pop());
            }
            if (items_[i].events & ZMQ_POLLOUT) {
                FD_SET(items_[i].fd, fds.pollset_out.pop());
            }
            if (items_[i].events & ZMQ_POLLERR) {
                FD_SET(items_[i].fd, fds.pollset_err.pop());
            }
            if (fds.maxfd < items_[i].fd) {
                fds.maxfd = items_[i].fd;
            }
        }
    }

    *rc = 0;
    return fds;
}

pub fn zmq_poll_select_set_timeout_(
    timeout: &mut timeval,
    first_pass: bool,
    now: u64,
    end: u64,
) -> timeval {
    timeval * ptimeout;
    if (first_pass) {
        timeout.tv_sec = 0;
        timeout.tv_usec = 0;
        ptimeout = &timeout;
    }
    // TODO
    // else if timeout < 0 {
    //     ptimeout = null_mut();
    // }
    else {
        timeout.tv_sec = ((end - now) / 1000) as c_long;
        timeout.tv_usec = ((end - now) % 1000 * 1000) as c_long;
        ptimeout = &timeout;
    }
    return timeval.clone();
}

// timespec *zmq_poll_select_set_timeout_ (
//   long timeout, first_pass: bool, now: u64, end: u64, timespec &timeout)
// {
//     timespec *ptimeout;
//     if (first_pass) {
//         timeout.tv_sec = 0;
//         timeout.tv_nsec = 0;
//         ptimeout = &timeout;
//     } else if (timeout < 0)
//         ptimeout = null_mut();
//     else {
//         timeout.tv_sec = static_cast<long> ((end - now) / 1000);
//         timeout.tv_nsec = static_cast<long> ((end - now) % 1000 * 1000000);
//         ptimeout = &timeout;
//     }
//     return ptimeout;
// }

pub fn zmq_poll_select_check_events_(
    options: &mut ZmqOptions,
    items_: &mut [ZmqPollItem],
    fds: &mut zmq_poll_select_fds_t_,
    nevents: &mut i32,
) -> i32 {
    //  Check for the events.
    // for (int i = 0; i != nitems_; i+= 1)
    for i in 0..items_.len() {
        items_[i].revents = 0;

        //  The poll item is a 0MQ socket. Retrieve pending events
        //  using the ZMQ_EVENTS socket option.
        if (items_[i].socket) {
            let mut zmq_events_size = mem::size_of::<u32>();
            let mut zmq_events = 0u32;
            zmq_getsockopt(
                options,
                items_[i].socket,
                ZMQ_EVENTS,
                &mut zmq_events as &mut [u8],
                &mut zmq_events_size,
            )?;

            if ((items_[i].events & ZMQ_POLLOUT == 1) && (zmq_events & ZMQ_POLLOUT == 1)) {
                items_[i].revents |= ZMQ_POLLOUT;
            }
            if ((items_[i].events & ZMQ_POLLIN == 1) && (zmq_events & ZMQ_POLLIN == 1)) {
                items_[i].revents |= ZMQ_POLLIN;
            }
        }
        //  Else, the poll item is a raw file descriptor, simply convert
        //  the events to ZmqPollItem-style format.
        else {
            if (FD_ISSET(items_[i].fd, fds.inset.pop())) {
                items_[i].revents |= ZMQ_POLLIN;
            }
            if (FD_ISSET(items_[i].fd, fds.outset.pop())) {
                items_[i].revents |= ZMQ_POLLOUT;
            }
            if (FD_ISSET(items_[i].fd, fds.errset.pop())) {
                items_[i].revents |= ZMQ_POLLERR;
            }
        }

        if (items_[i].revents) {
            *nevents += 1;
        }
    }

    return 0;
}

pub fn zmq_poll_must_break_loop_(
    timeout: i32,
    nevents: i32,
    first_pass: &mut bool,
    clock: &mut clock_t,
    now: &mut u64,
    end: &mut u64,
) -> bool {
    //  If timeout is zero, exit immediately whether there are events or not.
    if (timeout == 0) {
        return true;
    }

    //  If there are events to return, we can exit immediately.
    if (nevents) {
        return true;
    }

    //  At this point we are meant to wait for events but there are none.
    //  If timeout is infinite we can just loop until we get some events.
    if (timeout < 0) {
        if (first_pass) {
            *first_pass = false;
        }
        return false;
    }

    //  The timeout is finite and there are no events. In the first pass
    //  we get a timestamp of when the polling have begun. (We assume that
    //  first pass have taken negligible time). We also compute the time
    //  when the polling should time out.
    if (first_pass) {
        *now = clock.now_ms();
        *end = now + timeout;
        if (now == end) {
            return true;
        }
        *first_pass = false;
        return false;
    }

    //  Find out whether timeout have expired.
    *now = clock.now_ms();
    if (now >= end) {
        return true;
    }

    // finally, in all other cases, we just continue
    return false;
}
// #endif // ZMQ_HAVE_PPOLL

// #if !defined _WIN32
// TODO
// int zmq_ppoll (ZmqPollItem *items_,
//                nitems_: i32,
//                long timeout,
//                 sigset_t *sigmask_)
// #else
// Windows has no sigset_t
pub fn zmq_ppoll(
    options: &mut ZmqOptions,
    items_: &mut [ZmqPollItem],
    nitems_: i32,
    timeout: i32,
    sigmask_: *mut c_void,
) -> Result<i32, ZmqError>
// #endif
{
    // #ifdef ZMQ_HAVE_PPOLL
    zmq_poll_check_items_(items_, timeout)?;

    // clock_t clock;
    let mut clock: clock_t = clock_t::new();
    // u64 now = 0;
    let mut now = 0u64;
    // u64 end = 0;
    let mut end = 0u64;
    let fds = zmq_poll_build_select_fds_(options, items_, rc)?;

    let mut first_pass = true;
    let mut nevents = 0;

    loop {
        //  Compute the timeout for the subsequent poll.
        // timespec timeout;
        let mut timeout: timespec = timespec::new();
        let mut ptimeout =
            zmq_poll_select_set_timeout_(&mut timeout as &mut timeval, first_pass, now, end);

        //  Wait for events. Ignore interrupts if there's infinite timeout.
        loop {
            // TODO
            // memcpy (fds.inset.get (), fds.pollset_in.get (),
            //         valid_pollset_bytes (*fds.pollset_in.get ()));
            // memcpy (fds.outset.get (), fds.pollset_out.get (),
            //         valid_pollset_bytes (*fds.pollset_out.get ()));
            // memcpy (fds.errset.get (), fds.pollset_err.get (),
            //         valid_pollset_bytes (*fds.pollset_err.get ()));
            unsafe {
                let rc = pselect(
                    fds.maxfd + 1,
                    fds.inset.get(),
                    fds.outset.get(),
                    fds.errset.get(),
                    &ptimeout as *const timespec,
                    sigmask_ as *const sigset_t,
                );
                if (rc == -1) {
                    // errno_assert (errno == EINTR || errno == EBADF);
                    return Err(SelectFailed("zmq_poll: pselect".to_string()));
                }
            }
            break;
        }

        zmq_poll_select_check_events_(options, items_, fds, &mut nevents)?;

        if zmq_poll_must_break_loop_(
            timeout.tv_sec as i32,
            nevents,
            &mut first_pass,
            &mut clock,
            &mut now,
            &mut end,
        ) {
            break;
        }
    }

    return Ok(nevents);
    // #else
    //     errno = ENOTSUP;
    //     return -1;
    // #endif // ZMQ_HAVE_PPOLL
}

//  The poller functionality

pub fn zmq_poller_new() -> ZmqSocketPoller {
    ZmqSocketPoller::new()
}

pub fn zmq_poller_destroy(poller: &mut ZmqSocketPoller) -> i32 {
    // if (poller_p_) {
    //      ZmqSocketPoller * poller =
    //       static_cast< ZmqSocketPoller *> (*poller_p_);
    //     if (poller && poller.check_tag ()) {
    //         delete poller;
    //         *poller_p_ = null_mut();
    //         return 0;
    //     }
    // }
    // errno = EFAULT;
    // return -1;
    return 0;
}

pub fn check_poller(poller_: &mut ZmqSocketPoller) -> Result<(), ZmqError> {
    // if (!poller_
    //     || !(static_cast<ZmqSocketPoller *> (poller_))->check_tag ()) {
    //     errno = EFAULT;
    //     return -1;
    // }
    //
    // return 0;
    if poller_.check_tag() {
        return Ok(());
    }
    return Err(CheckTagFailed(String::from("poller_")));
}

pub fn check_events(events_: i16) -> Result<(), ZmqError> {
    if (events_ & !(ZMQ_POLLIN | ZMQ_POLLOUT | ZMQ_POLLERR | ZMQ_POLLPRI)) {
        // errno = EINVAL;
        return Err(InvalidEvent(fmt!("invalid events value: {}", events_)));
    }
    Ok(())
}

pub fn check_poller_registration_args(
    poller_: &mut ZmqSocketPoller,
    s_: &mut ZmqSocketBase,
) -> Result<(), ZmqError> {
    check_poller(poller_)?;

    if s_.check_tag() {
        return Ok(());
    } else {
        return Err(CheckTagFailed(String::from("s_")));
    }
}

pub fn check_poller_fd_registration_args(
    poller_: &mut ZmqSocketPoller,
    fd: ZmqFileDesc,
) -> Result<(), ZmqError> {
    check_poller(poller_)?;

    if (fd == retired_fd) {
        return Err(InvalidFileDescriptor(String::from("retired_fd")));
    }

    Ok(())
}

pub fn zmq_poller_size(poller_: &mut ZmqSocketPoller) -> Result<usize, ZmqError> {
    check_poller(poller_)?;
    Ok(poller_._pollset_size as usize)
}

pub fn zmq_poller_add(
    poller_: &mut ZmqSocketPoller,
    s_: &mut ZmqSocketBase,
    user_data_: Option<&mut [u8]>,
    events_: i16,
) -> Result<(), ZmqError> {
    check_poller_registration_args(poller_, s_)?;
    check_events(events_)?;

    match poller_.add(s_, user_data_, events_) {
        Ok(_) => Ok(()),
        Err(e) => Err(AddItemToPollerFailed(e.to_string())),
    }
}

pub fn zmq_poller_add_fd(
    poller_: &mut ZmqSocketPoller,
    fd: ZmqFileDesc,
    user_data_: Option<&mut [u8]>,
    events_: i16,
) -> Result<(), ZmqError> {
    check_poller_fd_registration_args(poller_, fd)?;
    check_events(events_)?;
    match poller_.add_fd(fd, user_data_, events_) {
        Ok(_) => Ok(()),
        Err(e) => Err(AddItemToPollerFailed(e.to_string())),
    }
}

pub fn zmq_poller_modify(
    poller_: &mut ZmqSocketPoller,
    s_: &mut ZmqSocketBase,
    events_: i16,
) -> Result<(), ZmqError> {
    check_poller_registration_args(poller_, s_)?;
    check_events(events_)?;
    match poller_.modify(s_, events_) {
        Ok(_) => Ok(()),
        Err(e) => Err(ModifyPollerItemFailed(e.to_string())),
    }
}

pub fn zmq_poller_modify_fd(
    poller_: &mut ZmqSocketPoller,
    fd: ZmqFileDesc,
    events_: i16,
) -> Result<(), ZmqError> {
    check_poller_fd_registration_args(poller_, fd)?;
    check_events(events_)?;
    match poller_.modify_fd(fd, events_) {
        Ok(_) => Ok(()),
        Err(e) => Err(ModifyPollerItemFailed(e.to_string())),
    }
}

pub fn zmq_poller_remove(
    poller_: &mut ZmqSocketPoller,
    s_: &mut ZmqSocketBase,
) -> Result<(), ZmqError> {
    check_poller_registration_args(poller_, s_)?;
    match poller_.remove(s_) {
        Ok(_) => Ok(()),
        Err(e) => Err(RemoveItemFromPollerFailed(e.to_string())),
    }
}

pub fn zmq_poller_remove_fd(
    poller_: &mut ZmqSocketPoller,
    fd: ZmqFileDesc,
) -> Result<(), ZmqError> {
    check_poller_fd_registration_args(poller_, fd)?;

    match poller_.remove_fd(fd) {
        Ok(_) => Ok(()),
        Err(e) => Err(RemoveItemFromPollerFailed(e.to_string())),
    }
}

pub fn zmq_poller_wait(
    poller_: &mut ZmqSocketPoller,
    event_: &mut ZmqPollerEvent,
    timeout: timeout,
) -> Result<(), ZmqError> {
    zmq_poller_wait_all(poller_, event_, 1, timeout)?;

    event_.socket = None;
    event_.fd = retired_fd;
    event_.user_data = None;
    event_.events = 0;

    Ok(())
}

pub fn zmq_poller_wait_all(
    poller_: &mut ZmqSocketPoller,
    events_: &mut ZmqPollerEvent,
    n_events_: usize,
    timeout: i32,
) -> Result<(), ZmqError> {
    check_poller(poller_)?;
    if !events_ {
        return Err(InvalidPollerEventArray(String::from("events_")));
    }
    if n_events_ < 0 {
        return Err(InvalidPollerEventArraySize(n_events_));
    }

    match poller_.wait(events_, n_events_, timeout) {
        Ok(_) => Ok(()),
        Err(e) => Err(PollerWaitFailed(e.to_string())),
    }
    if rc < 0 {
        return Err(PollerWaitFailed(rc));
    }

    Ok(())
}

pub fn zmq_poller_fd(poller_: &mut ZmqSocketPoller) -> Result<ZmqFileDesc, ZmqError> {
    poller_.check_tag()?;
    match poller_.signaler_fd() {
        Ok(fd) => Ok(fd),
        Err(e) => Err(GetPollerSignalerFdFailed(e.to_string())),
    }
}

//  Peer-specific state

pub fn zmq_socket_get_peer_state(
    s_: &mut ZmqSocketBase,
    routing_id_: &mut [u8],
    routing_id_size_: usize,
) -> Result<i32, ZmqError> {
    match s_.get_peer_state(routing_id_, routing_id_size_) {
        Ok(rc) => Ok(rc),
        Err(e) => Err(GetSocketPeerStateFailed(e.to_string())),
    }
}

//  Timers

pub fn zmq_timers_new() -> ZmqTimers {
    ZmqTimers::new()
}

pub fn zmq_timers_destroy(timers: &mut ZmqTimers) -> Result<(), ZmqError> {
    // void *timers = *timers_p_;
    // if (!timers || !(static_cast<ZmqTimers *> (timers))->check_tag ()) {
    //     errno = EFAULT;
    //     return -1;
    // }
    // delete (static_cast<ZmqTimers *> (timers));
    // *timers_p_ = null_mut();
    // return 0;
    timers.check_tag()?;
    Ok(())
}

pub fn zmq_timers_add(
    timers_: &mut ZmqTimers,
    interval_: usize,
    handler_: zmq_timer_fn,
    arg_: &mut [u8],
) -> Result<(), ZmqError> {
    timers.check_tag()?;
    match timers.add(interval_, handler_, arg_) {
        Ok(_) => Ok(()),
        Err(e) => Err(AddTimerFailed(e.to_string())),
    }
}

pub fn zmq_timers_cancel(timers_: &mut ZmqTimers, timer_id_: i32) -> Result<(), ZmqError> {
    timers.check_tag()?;
    match timers.cancel(timer_id_) {
        Ok(_) => Ok(()),
        Err(e) => Err(CancelTimerFailed(e.to_string())),
    }
}

pub fn zmq_timers_set_interval(
    timers_: &mut ZmqTimers,
    timer_id_: i32,
    interval_: usize,
) -> Result<(), ZmqError> {
    timers.check_tag()?;
    match timers.set_interval(timer_id_, interval_) {
        Ok(_) => Ok(()),
        Err(e) => Err(SetTimerIntervalFailed(e.to_string())),
    }
}

pub fn zmq_timers_reset(timers_: &mut ZmqTimers, timer_id_: i32) -> Result<(), ZmqError> {
    timers.check_tag()?;

    match timers.reset(timer_id_) {
        Ok(_) => Ok(()),
        Err(e) => Err(ResetTimerFailed(e.to_string())),
    }
}

pub fn zmq_timers_timeout(timers_: &mut ZmqTimers) -> Result<i32, ZmqError> {
    timers.check_tag()?;

    match timers.timeout() {
        Ok(rc) => Ok(rc),
        Err(e) => Err(GetTimerTimeoutFailed(e.to_string())),
    }
}

pub fn zmq_timers_execute(timers_: &mut ZmqTimers) -> Result<(), ZmqError> {
    timers.check_tag()?;
    match timers.execute() {
        Ok(_) => Ok(()),
        Err(e) => Err(ExecuteTimerFailed(e.to_string())),
    }
}

//  The proxy functionality
pub fn zmq_proxy(
    options: &mut ZmqOptions,
    frontend_: &mut ZmqSocketBase,
    backend_: &mut ZmqSocketBase,
    capture_: &mut ZmqSocketBase,
) -> Result<(), ZmqError> {
    match proxy(options, frontend_, backend_, capture_, None) {
        Ok(_) => Ok(()),
        Err(e) => Err(ProxyFailed(e.to_string())),
    }
}

pub fn zmq_proxy_steerable(
    options: &mut ZmqOptions,
    frontend_: &mut ZmqSocketBase,
    backend_: &mut ZmqSocketBase,
    capture_: &mut ZmqSocketBase,
    control_: Option<&mut ZmqSocketBase>,
) -> Result<(), ZmqError> {
    match proxy(options, (frontend_), (backend_), (capture_), (control_)) {
        Ok(_) => Ok(()),
        Err(e) => Err(ProxyFailed(e.to_string())),
    }
}

//  The deprecated device functionality

// pub fn zmq_device (frontend_: &mut [u8], backend_: *mut c_void)
// {
//     return proxy (static_cast<ZmqSocketBase *> (frontend_),
//                        static_cast<ZmqSocketBase *> (backend_), null_mut());
// }

//  Probe library capabilities; for now, reports on transport and security

pub fn zmq_has(capability_: &str) -> bool {
    // // #if defined(ZMQ_HAVE_IPC)
    //     if (strcmp (capability_, protocol_name::ipc) == 0)
    //         return true;
    // // #endif
    // // #if defined(ZMQ_HAVE_OPENPGM)
    //     if (strcmp (capability_, protocol_name::pgm) == 0)
    //         return true;
    // // #endif
    // // #if defined(ZMQ_HAVE_TIPC)
    //     if (strcmp (capability_, protocol_name::tipc) == 0)
    //         return true;
    // // #endif
    // // #if defined(ZMQ_HAVE_NORM)
    //     if (strcmp (capability_, protocol_name::norm) == 0)
    //         return true;
    // // #endif
    // // #if defined(ZMQ_HAVE_CURVE)
    //     if (strcmp (capability_, "curve") == 0)
    //         return true;
    // // #endif
    // // #if defined(HAVE_LIBGSSAPI_KRB5)
    //     if (strcmp (capability_, "gssapi") == 0)
    //         return true;
    // // #endif
    // // #if defined(ZMQ_HAVE_VMCI)
    //     if (strcmp (capability_, protocol_name::vmci) == 0)
    //         return true;
    // // #endif
    // // #if defined(ZMQ_BUILD_DRAFT_API)
    //     if (strcmp (capability_, "draft") == 0)
    //         return true;
    // // #endif
    // // #if defined(ZMQ_HAVE_WS)
    //     if (strcmp (capability_, "WS") == 0)
    //         return true;
    // // #endif
    // // #if defined(ZMQ_HAVE_WSS)
    //     if (strcmp (capability_, "WSS") == 0)
    //         return true;
    // // #endif
    //     //  Whatever the application asked for, we don't have
    //     return false;
    unimplemented!()
}

pub fn zmq_socket_monitor_pipes_stats(s_: &mut ZmqSocketBase) -> Result<(), ZmqError> {
    match s_.query_pipes_stats() {
        Ok(_) => Ok(()),
        Err(e) => Err(QueryPipesStatsFailed(e.to_string())),
    }
}
