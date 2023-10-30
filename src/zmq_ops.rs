use std::intrinsics::size_of;
use std::os::raw::{c_long, c_uint, c_void};

#[cfg(not(target_os = "windows"))]
use libc::iovec;
use libc::{pselect, sigset_t, time_t, timespec, timeval, usleep, EINTR};
use windows::Win32::Networking::WinSock::{POLLIN, POLLOUT, POLLPRI};

use crate::clock::ZmqClock;
use crate::ctx::ZmqContext;
use crate::defines::{
    ZmqFd, ZmqPollFd, MSG_MORE, MSG_SHARED, RETIRED_FD, ZMQ_EVENTS, ZMQ_FD, ZMQ_IO_THREADS,
    ZMQ_MORE, ZMQ_PAIR, ZMQ_PEER, ZMQ_POLLERR, ZMQ_POLLIN, ZMQ_POLLOUT, ZMQ_POLLPRI, ZMQ_SNDMORE,
    ZMQ_TYPE, ZMQ_VERSION_MAJOR, ZMQ_VERSION_MINOR, ZMQ_VERSION_PATCH,
};
use crate::err::ZmqError;
use crate::err::ZmqError::{InvalidContext, PollerError, SocketError, TimerError};
use crate::ip::{initialize_network, shutdown_network};
use crate::msg::{MsgFreeFn, ZmqMsg};
use crate::options::ZmqOptions;
use crate::poll::zmq_poll_int;
use crate::poller_event::ZmqPollerEvent;
use crate::polling_util::{compute_timeout, valid_pollset_bytes, OptimizedFdSet};
use crate::pollitem::ZmqPollitem;
use crate::proxy::proxy;
use crate::select::{FD_SET, FD_ZERO};
use crate::socket::ZmqSocket;
use crate::socket_poller::ZmqSocketPoller;
use crate::timers::{Timers, TimersTimerFn};
use crate::utils::{get_errno, FD_ISSET};
use crate::zmq_draft::zmq_fd_t;

/*  Socket types.                                                             */

/*  Deprecated aliases                                                        */

/*  Socket options.                                                           */

/*  Message options                                                           */

/*  Send/recv options.                                                        */

/*  Security mechanisms                                                       */

/*  RADIO-DISH protocol                                                       */

/*  Deprecated options and aliases                                            */

/*  Deprecated Message options                                                */

/******************************************************************************/
/*  GSSAPI definitions                                                        */
/******************************************************************************/

/*  GSSAPI principal name types                                               */
/*  Unspecified system errors during handshake. Event value is an errno.      */
/*  Handshake complete successfully with successful authentication (if        *
 *  enabled). Event value is unused.                                          */
/*  Protocol errors between ZMTP peers or between server and ZAP handler.     *
 *  Event value is one of ZMQ_PROTOCOL_ERROR_*                                */
/*  Failed authentication requests. Event value is the numeric ZAP status     *
 *  code, i.e. 300, 400 or 500.                                               */

// #if defined _WIN32
// // Windows uses a pointer-sized unsigned integer to store the socket fd.
// #if defined _WIN64
// typedef unsigned __int64 zmq_fd_t;
// #else
// typedef unsigned int zmq_fd_t;
// #endif
// #else
// typedef int zmq_fd_t;
// #endif

/*  Deprecated aliases */

/*  DRAFT Socket types.                                                       */

/*  DRAFT Socket options.                                                     */

/*  DRAFT ZMQ_NORM_MODE options                                               */

/*  DRAFT ZMQ_RECONNECT_STOP options                                          */

/*  DRAFT Context options                                                     */

/*  DRAFT Msg property names.                                                 */

/*  Router notify options                                                     */

/*  DRAFT Socket monitoring events                                            */

//  Compile time check whether msg_t fits into zmq_msg_t.
// typedef char
//   check_msg_t_size[sizeof (zmq::msg_t) == sizeof (zmq_msg_t) ? 1 : -1];

pub fn zmq_version() -> (u32, u32, u32) {
    (ZMQ_VERSION_MAJOR, ZMQ_VERSION_MINOR, ZMQ_VERSION_PATCH)
}

// const char *zmq_strerror (int errnum_)
pub fn zmq_strerror(errnum_: i32) -> &'static str {
    // return zmq::errno_to_string (errnum_);
    todo!()
}

// int zmq_errno (void)
pub fn zmq_errno() -> i32 {
    // return errno;
    todo!()
}

// void *zmq_ctx_new (void)
pub fn zmq_ctx_new() -> Result<ZmqContext, ZmqError> {
    //  We do this before the ctx constructor since its embedded mailbox_t
    //  object needs the network to be up and running (at least on Windows).
    initialize_network()?;

    //  Create 0MQ context.
    let mut ctx = ZmqContext::new();
    if !ctx.valid() {
        return Err(InvalidContext("invalid context"));
    }
    return Ok(ctx);
}

// pub zmq_ctx_term (void *ctx_)
pub fn zmq_ctx_term(ctx: &mut ZmqContext) -> Result<(), ZmqError> {
    // if (!ctx_ || !(static_cast<zmq::ctx_t *> (ctx_))->check_tag ()) {
    if ctx.check_tag() == false {
        // errno = EFAULT;
        return Err(InvalidContext("tag check failed"));
    }

    // const int rc = (static_cast<zmq::ctx_t *> (ctx_))->terminate ();
    ctx.terminate()?;
    // const int en = errno;
    // let en = get_errno();

    //  Shut down only if termination was not interrupted by a signal.
    if get_errno() != EINTR {
        shutdown_network();
    }

    // errno = en;
    return Ok(());
}

// int zmq_ctx_shutdown (void *ctx_)
pub fn zmq_ctx_shutdown(ctx_: &mut ZmqContext) -> Result<(), ZmqError> {
    // if (!ctx_ || !(static_cast<zmq::ctx_t *> (ctx_))->check_tag ())
    if ctx_.check_tag() == false {
        // errno = EFAULT;
        return Err(InvalidContext("tag check failed"));
    }
    // return (static_cast<zmq::ctx_t *> (ctx_))->shutdown ();
    return ctx_.shutdown();
}

// int zmq_ctx_set (void *ctx_, int option_, int optval_)
pub fn zmq_ctx_set(ctx_: &mut ZmqContext, option_: i32, optval_: i32) -> i32 {
    zmq_ctx_set_ext(ctx_, option_, &optval_.to_le_bytes(), 4)
}

pub fn zmq_ctx_set_ext(
    ctx_: &mut ZmqContext,
    option_: i32,
    optval_: &[u8],
    optvallen_: usize,
) -> i32 {
    // if (!ctx_ || !(static_cast<zmq::ctx_t *> (ctx_))->check_tag ()) {
    //         errno = EFAULT;
    //         return -1;
    //     }
    //     return (static_cast<zmq::ctx_t *> (ctx_))
    //       ->set (option_, optval_, optvallen_);
    if ctx_.check_tag() == false {
        // errno = EFAULT;
        return -1;
    }
    ctx_.set(option_, optval_, optvallen_)
}

// int zmq_ctx_get (void *ctx_, int option_)
pub fn zmq_ctx_get(ctx_: &mut ZmqContext, option_: i32) -> i32 {
    // if (!ctx_ || !(static_cast<zmq::ctx_t *> (ctx_))->check_tag ()) {
    //     errno = EFAULT;
    //     return -1;
    // }
    // return (static_cast<zmq::ctx_t *> (ctx_))->get (option_);
    if ctx_.check_tag() == false {
        // errno = EFAULT;
        return -1;
    }
    ctx_.get(option_)
}

// int zmq_ctx_get_ext (void *ctx_, int option_, void *optval_, size_t *optvallen_)
pub fn zmq_ctx_get_ext(
    ctx_: &mut ZmqContext,
    option_: i32,
    optval_: &[u8],
    optvallen_: usize,
) -> i32 {
    // if (!ctx_ || !(static_cast<zmq::ctx_t *> (ctx_))->check_tag ()) {
    //     errno = EFAULT;
    //     return -1;
    // }
    // return (static_cast<zmq::ctx_t *> (ctx_))
    //   ->get (option_, optval_, optvallen_);
    if ctx_.check_tag() == false {
        // errno = EFAULT;
        return -1;
    }
    ctx_.get(option_, optval_, optvallen_)
}

// void *zmq_init (int io_threads_)
pub fn zmq_init(io_threads_: i32) -> Option<ZmqContext> {
    if (io_threads_ >= 0) {
        // void *ctx = zmq_ctx_new ();
        let mut ctx = zmq_ctx_new().unwrap();
        zmq_ctx_set(&mut ctx, ZMQ_IO_THREADS as i32, io_threads_);
        return Some(ctx);
    }
    // errno = EINVAL;
    return None;
}

// int zmq_term (void *ctx_)
pub fn zmq_term(ctx_: &mut ZmqContext) -> Result<(), ZmqError> {
    return zmq_ctx_term(ctx_);
}

// int zmq_ctx_destroy (void *ctx_)
pub fn zmq_ctx_destroy(ctx_: &mut ZmqContext) -> Result<(), ZmqError> {
    return zmq_ctx_term(ctx_);
}

// static zmq::socket_base_t *as_socket_base_t (void *s_)
pub fn as_socket_base_t<'a>(s_: &mut ZmqSocket) -> Option<&'a mut ZmqSocket> {
    // zmq::socket_base_t *s = static_cast<zmq::socket_base_t *> (s_);
    // if (!s_ || !s->check_tag ()) {
    //     errno = ENOTSOCK;
    //     return NULL;
    // }
    // return s;
    if s_.check_tag() == false {
        // errno = ENOTSOCK;
        return None;
    }
    return Some(s_);
}

pub unsafe fn zmq_socket<'a>(ctx_: &mut ZmqContext, type_: i32) -> Result<ZmqSocket, ZmqError> {
    // if (!ctx_ || !(static_cast<zmq::ctx_t *> (ctx_))->check_tag ())
    if ctx_.check_tag() == false {
        // errno = EFAULT;
        return Err(InvalidContext("tag check failed"));
    }
    // zmq::ctx_t *ctx = static_cast<zmq::ctx_t *> (ctx_);
    // zmq::socket_base_t *s = ctx->create_socket (type_);
    // return static_cast<void *> (s);
    return ctx_.create_socket(type_);
}

// int zmq_close (void *s_)
pub unsafe fn zmq_close(s_: &mut ZmqSocket) -> i32 {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    s_.close();
    return 0;
}

pub unsafe fn zmq_setsockopt(
    options: &mut ZmqOptions,
    sock: &mut ZmqSocket,
    option_: i32,
    optval_: &[u8],
    optvallen_: usize,
) -> Result<(), ZmqError> {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    // return s->setsockopt (option_, optval_, optvallen_);
    sock.setsockopt(options, option_, optval_, optvallen_)
}

// int zmq_getsockopt (void *s_, int option_, void *optval_, size_t *optvallen_)
pub unsafe fn zmq_getsockopt(sock: &mut ZmqSocket, option_: u32) -> Result<[u8], ZmqError> {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    // return s->getsockopt (option_, optval_, optvallen_);
    sock.getsockopt(option_)
}

// int zmq_socket_monitor_versioned (
//   void *s_, const char *addr_, uint64_t events_, int event_version_, int type_)
pub unsafe fn zmq_socket_monitor_versioned(
    sock: &mut ZmqSocket,
    addr: &str,
    events: u64,
    event_version: i32,
    type_: i32,
) -> i32 {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    // return s->monitor (addr_, events_, event_version_, type_);
    sock.monitor(addr, events, event_version, type_)
}

// int zmq_socket_monitor (void *s_, const char *addr_, int events_)
pub unsafe fn zmq_socket_monitor(sock: &mut ZmqSocket, addr_: &str, events_: i32) -> i32 {
    return zmq_socket_monitor_versioned(sock, addr_, events_ as u64, 1, ZMQ_PAIR as i32);
}

// int zmq_join (void *s_, const char *group_)
pub unsafe fn zmq_join(s_: &mut ZmqSocket, group_: &str) -> i32 {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    // return s->join (group_);
    s_.join(group_)
}

// int zmq_leave (void *s_, const char *group_)
pub unsafe fn zmq_leave(s_: &mut ZmqSocket, group_: &str) -> i32 {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    // return s->leave (group_);
    s_.leave(group_)
}

// int zmq_bind (void *s_, const char *addr_)
pub unsafe fn zmq_bind(s_: &mut ZmqSocket, addr_: &str) -> Result<(), ZmqError> {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    // return s->Bind (addr_);
    s_.bind(addr_)
}

// int zmq_connect (void *s_, const char *addr_)
pub unsafe fn zmq_connect(s_: &mut ZmqSocket, addr_: &str) -> i32 {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    // return s->connect (addr_);
    s_.connect(addr_)
}

// uint32_t zmq_connect_peer (void *s_, const char *addr_)
pub unsafe fn zmq_connect_peer(sock: &mut ZmqSocket, addr_: &str) -> Result<(), ZmqError> {
    // zmq::peer_t *s = static_cast<zmq::peer_t *> (s_);
    // if (!s_ || !s->check_tag ()) {
    //     errno = ENOTSOCK;
    //     return 0;
    // }
    if sock.check_tag() == false {
        return Err(SocketError("tag check failed"));
    }

    // int socket_type;
    // let mut socket_type = 0i32;
    // size_t socket_type_size = sizeof (socket_type);
    // let mut socket_type_size = 4;
    // if (s_.getsockopt(ZMQ_TYPE, &socket_type, &socket_type_size) != 0) {
    //     return 0;
    // }
    let socket_type_bytes = sock.getsockopt(ZMQ_TYPE)?;
    let socket_type = u32::from(socket_type_bytes);

    if socket_type != ZMQ_PEER {
        // errno = ENOTSUP;
        return Err(SocketError("socket type not supported"));
    }

    return sock.connect_peer(addr_);
}

// int zmq_unbind (void *s_, const char *addr_)
pub unsafe fn unbind(s_: &mut ZmqSocket, addr_: &str) -> i32 {
    // zmq::socket_base_t *s = as_socket_base_t (s_);

    // if (!s)
    //     return -1;
    return s_.term_endpoint(addr_);
}

// int zmq_disconnect (void *s_, const char *addr_)
pub unsafe fn zmq_disconnect(s_: &mut ZmqSocket, addr_: &str) -> i32 {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    return s_.term_endpoint(addr_);
}

// static inline int
// s_sendmsg (zmq::socket_base_t *s_, zmq_msg_t *msg, int flags_)
pub unsafe fn s_sendmsg(
    s_: &mut ZmqSocket,
    msg: &mut ZmqMsg,
    flags_: i32,
) -> Result<usize, ZmqError> {
    let sz = zmq_msg_size(msg);
    s_.send((msg), flags_)?;
    // if ((rc < 0)) {
    //     return -1;
    // }

    //  This is what I'd like to do, my C++ fu is too weak -- PH 2016/02/09
    //  int max_msgsz = s_->parent->get (ZMQ_MAX_MSGSZ);
    let max_msgsz = usize::MAX;

    //  Truncate returned size to INT_MAX to avoid overflow to negative values
    return Ok(if sz < max_msgsz { sz } else { max_msgsz });
}

// int zmq_sendmsg (void *s_, zmq_msg_t *msg, int flags_)
pub unsafe fn zmq_sendmsq(
    s_: &mut ZmqSocket,
    msg: &mut ZmqMsg,
    flags_: i32,
) -> Result<usize, ZmqError> {
    return zmq_msg_send(msg, s_, flags_);
}

// int zmq_send (void *s_, const void *buf_, size_t len_, int flags_)
pub unsafe fn zmq_send(sock: &mut ZmqSocket, buf_: &[u8], len_: usize, flags_: i32) -> i32 {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    // zmq_msg_t msg;
    let mut msg = ZmqMsg::default();
    msg = zmq_msg_init_buffer(buf_, len_)?;
    // if ((rc < 0)) {
    //     return -1;
    // }

    s_sendmsg(sock, &mut msg, flags_)?;
    // if ((rc < 0)) {
    //     // const int err = errno;
    //     let rc2 = zmq_msg_close(&msg);
    //     // errno_assert (rc2 == 0);
    //     // errno = err;
    //     return -1;
    // }
    //  Note the optimisation here. We don't close the msg object as it is
    //  empty anyway. This may change when implementation of zmq_msg_t changes.
    // return rc;
}

// int zmq_send_const (void *s_, const void *buf_, size_t len_, int flags_)
pub unsafe fn zmq_send_const(
    sock: &mut ZmqSocket,
    in_buf: &[u8],
    in_buf_len: usize,
    flags: i32,
) -> Result<usize, ZmqError> {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    // zmq_msg_t msg;
    let mut msg = ZmqMsg::default();
    zmq_msg_init_data(&mut msg, in_buf, in_buf_len, None, None)?;
    // if (rc != 0) {
    //     return -1;
    // }

    return s_sendmsg(sock, &mut msg, flags);
    // if ((rc < 0)) {
    //     // const int err = errno;
    //     let rc2 = zmq_msg_close(&msg);
    //     // errno_assert (rc2 == 0);
    //     // errno = err;
    //     return -1;
    // }
    //  Note the optimisation here. We don't close the msg object as it is
    //  empty anyway. This may change when implementation of zmq_msg_t changes.
    // return rc;
}

// int zmq_sendiov (void *s_, iovec *a_, size_t count_, int flags_)
#[cfg(not(target_os = "windows"))]
pub unsafe fn zmq_sendiov(
    sock: &mut ZmqSocket,
    in_iovecs: &[iovec],
    count: usize,
    mut flags: i32,
) -> Result<usize, ZmqError> {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    if count <= 0 {
        // errno = EINVAL;
        return Err(SocketError("invalid argument"));
    }

    let mut rc = 0;
    // zmq_msg_t msg;
    let mut msg = ZmqMsg::default();

    // for (size_t i = 0; i < count_; ++i)
    for i in 0..count {
        zmq_msg_init_size(&mut msg, in_iovecs[i].iov_len)?;
        // if (rc != 0) {
        //     rc = -1;
        //     break;
        // }
        // libc::memcpy(zmq_msg_data(&msg), in_iovecs[i].iov_base, in_iovecs[i].iov_len);
        let iov_data: Vec<u8> = Vec::from_raw_parts(
            in_iovecs[i].iov_base as *mut u8,
            in_iovecs[i].iov_len,
            in_iovecs[i].iov_len,
        );
        zmq_msg_data(&mut msg).copy_from_slice(iov_data.as_slice());

        if i == count - 1 {
            flags = flags & !ZMQ_SNDMORE;
        }
        rc = s_sendmsg(sock, &mut msg, flags)?;
        // if rc < 0 {
        //     // const int err = errno;
        //     let rc2 = zmq_msg_close(&msg);
        //     // errno_assert (rc2 == 0);
        //     // errno = err;
        //     rc = -1;
        //     break;
        // }
    }
    Ok(rc)
}

// static int s_recvmsg (zmq::socket_base_t *s_, zmq_msg_t *msg, int flags_)
pub unsafe fn s_recvmsg(
    s_: &mut ZmqSocket,
    msg: &mut ZmqMsg,
    flags_: i32,
) -> Result<usize, ZmqError> {
    s_.recv((msg), flags_)?;

    //  Truncate returned size to INT_MAX to avoid overflow to negative values
    let sz = zmq_msg_size(msg);
    return Ok(if sz < i32::MAX as usize {
        sz
    } else {
        i32::MAX as usize
    });
}

// int zmq_recvmsg (void *s_, zmq_msg_t *msg, int flags_)
pub unsafe fn zmq_recvmsg(
    s_: &mut ZmqSocket,
    msg: &mut ZmqMsg,
    flags_: i32,
) -> Result<usize, ZmqError> {
    return zmq_msg_recv(msg, s_, flags_);
}

// int zmq_recv (void *s_, void *buf_, size_t len_, int flags_)
pub unsafe fn zmq_recv(
    s_: &mut ZmqSocket,
    buf_: &mut [u8],
    len_: usize,
    flags_: i32,
) -> Result<usize, ZmqError> {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    // zmq_msg_t msg;
    let mut msg = ZmqMsg::default();
    zmq_msg_init(&mut msg)?;
    // errno_assert (rc == 0);

    let nbytes = s_recvmsg(s_, &mut msg, flags_)?;
    if nbytes < 0 {
        // let err = errno;
        zmq_msg_close(&mut msg)?;
        // errno_assert (rc == 0);
        // errno = err;
        return Err(SocketError("recvmsg failed"));
    }

    //  An oversized message is silently truncated.
    let to_copy = if (nbytes) < len_ { nbytes } else { len_ };

    //  We explicitly allow a null buffer argument if len is zero
    if to_copy {
        // assert (buf_);
        // libc::memcpy(buf_, zmq_msg_data(&msg), to_copy);
        buf_.copy_from_slice(zmq_msg_data(&mut msg));
    }
    zmq_msg_close(&mut msg)?;
    // errno_assert (rc == 0);

    return Ok(nbytes);
}

// int zmq_recviov (void *s_, iovec *a_, size_t *count_, int flags_)
#[cfg(not(target_os = "windows"))]
pub unsafe fn zmq_recviov(
    sock: &mut ZmqSocket,
    out_iovecs: &mut Vec<iovec>,
    iovec_count: &mut usize,
    flags_: i32,
) -> Result<usize, ZmqError> {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //    return -1;
    // if ((!count_ || *count_ <= 0 || !a_)) {
    //     // errno = EINVAL;
    //     return -1;
    // }

    let count = *iovec_count;
    let mut nread = 0;
    let mut recvmore = true;

    *iovec_count = 0;

    // for (size_t i = 0; recvmore && i < count; ++i)
    for i in 0..count {
        // zmq_msg_t msg;
        let mut msg = ZmqMsg::default();
        zmq_msg_init(&mut msg)?;
        // errno_assert (rc == 0);

        let nbytes = s_recvmsg(sock, &mut msg, flags_)?;
        if (nbytes < 0) {
            // const int err = errno;
            zmq_msg_close(&mut msg)?;
            // errno_assert (rc == 0);
            // errno = err;
            nread = -1;
            break;
        }

        let mut out_iovec = iovec::default();
        out_iovec.iov_len = zmq_msg_size(&mut msg);
        unsafe {
            out_iovec.iov_base = (libc::malloc(out_iovec.iov_len));
            if (!out_iovec.iov_base) {
                // errno = ENOMEM;
                return Err(SocketError("malloc failed"));
            }
            libc::memcpy(
                out_iovec.iov_base,
                zmq_msg_data(&mut msg).as_ptr() as *const libc::c_void,
                out_iovec.iov_len,
            );
        }

        // out_iovecs[i].iov_base = (libc::malloc(out_iovecs[i].iov_len));
        // if ((!out_iovecs[i].iov_base)) {
        //     // errno = ENOMEM;
        //     return -1;
        // }
        // libc::memcpy(out_iovecs[i].iov_base, (zmq_msg_data(&msg)),
        //              out_iovecs[i].iov_len);
        // Assume zmq_socket ZMQ_RVCMORE is properly set.
        let p_msg = (&msg);
        recvmore = p_msg.flag_set(MSG_MORE);
        zmq_msg_close(&mut msg)?;
        // errno_assert (rc == 0);
        // ++*count_;
        *iovec_count += 1;
        // ++nread;
        nread += 1;
    }
    return Ok(nread);
}

// int zmq_msg_init (zmq_msg_t *msg)
pub unsafe fn zmq_msg_init(msg: &mut ZmqMsg) -> Result<(), ZmqError> {
    return msg.init2();
}

// int zmq_msg_init_size (zmq_msg_t *msg, size_t size_)
pub unsafe fn zmq_msg_init_size(msg: &mut ZmqMsg, size_: usize) -> i32 {
    return msg.init_size(size_);
}

// int zmq_msg_init_buffer (zmq_msg_t *msg, const void *buf_, size_t size_)
pub unsafe fn zmq_msg_init_buffer(in_buf: &[u8], size_: usize) -> Result<ZmqMsg, ZmqError> {
    let mut msg = ZmqMsg::default();
    msg.init_buffer(in_buf, size_)?;
    return Ok(msg);
}

// int zmq_msg_init_data (
//   zmq_msg_t *msg, void *data_, size_t size_, zmq_free_fn *ffn_, void *hint_)
pub unsafe fn zmq_msg_init_data(
    msg: &mut ZmqMsg,
    data_: &[u8],
    size_: usize,
    ffn_: Option<MsgFreeFn>,
    hint_: Option<&[u8]>,
) -> Result<(), ZmqError> {
    return msg.init_data(data_, size_, ffn_, hint_.unwrap());
}

// int zmq_msg_send (zmq_msg_t *msg, void *s_, int flags_)
pub unsafe fn zmq_msg_send(
    msg: &mut ZmqMsg,
    s_: &mut ZmqSocket,
    flags_: u32,
) -> Result<usize, ZmqError> {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    return s_sendmsg(s_, msg, flags_);
}

// int zmq_msg_recv (zmq_msg_t *msg, void *s_, int flags_)
pub unsafe fn zmq_msg_recv(
    msg: &mut ZmqMsg,
    s_: &mut ZmqSocket,
    flags_: i32,
) -> Result<usize, ZmqError> {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    return s_recvmsg(s_, msg, flags_);
}

// int zmq_msg_close (zmq_msg_t *msg)
pub unsafe fn zmq_msg_close(msg: &mut ZmqMsg) -> i32 {
    return (msg).close();
}

// int zmq_msg_move (zmq_msg_t *dest_, zmq_msg_t *src_)
pub unsafe fn zmq_msg_move(dest: &mut ZmqMsg, src: &mut ZmqMsg) -> i32 {
    // return (dest.move(src));
    todo!()
}

// int zmq_msg_copy (zmq_msg_t *dest_, zmq_msg_t *src_)
pub unsafe fn zmq_msg_copy(dest_: &mut ZmqMsg, src_: &mut ZmqMsg) -> i32 {
    return (dest_).copy(src_);
}

pub unsafe fn zmq_msg_data<'a>(msg: &mut ZmqMsg) -> &'a mut [u8] {
    return (msg).data();
}

pub unsafe fn zmq_msg_data_mut<'a>(msg: &mut ZmqMsg) -> &'a mut [u8] {
    return msg.data_mut();
}

pub unsafe fn zmq_msg_size(msg: &mut ZmqMsg) -> usize {
    return (msg).size();
}

pub unsafe fn zmq_msg_more(msg: &mut ZmqMsg) -> Result<i32, ZmqError> {
    return zmq_msg_get(msg, ZMQ_MORE);
}

// int zmq_msg_get (const zmq_msg_t *msg, int property_)
pub unsafe fn zmq_msg_get(msg: &mut ZmqMsg, property_: u32) -> Result<i32, ZmqError> {
    match property_ {
        ZMQ_MORE => {
            // return (((zmq::msg_t *)
            // msg) -> flags() & zmq::msg_t::more) ? 1: 0;
            return if msg.flags() & MSG_MORE != 0 {
                Ok(1)
            } else {
                Ok(0)
            };
        }
        ZMQ_SRCFD => {
            let fd_string = zmq_msg_gets(msg, "__fd");
            // if (fd_string == NULL)
            // return -1;
            // return atoi(fd_string);
            match i32::from_str_radix(fd_string, 10) {
                Ok(fd) => return Ok(fd),
                Err(e) => return Err(ZmqError::ParseIntError(e)),
            };
        }
        ZMQ_SHARED => {
            // return (((zmq::msg_t *)
            // msg) -> is_cmsg())
            // || (((zmq::msg_t *)
            // msg) -> flags() & zmq::msg_t::shared) ? 1: 0;
            return if msg.is_cmsg() || msg.flags() & MSG_SHARED != 0 {
                Ok(1)
            } else {
                Ok(0)
            };
        }
        _ => {
            // errno = EINVAL;
            return Err(ZmqError::InvalidProperty("invalid property"));
        }
    }
}

// int zmq_msg_set (zmq_msg_t *, int, int)
pub unsafe fn zmq_msg_set(msg: &mut ZmqMsg, a: i32, b: i32) -> i32 {
    //  No properties supported at present
    // errno = EINVAL;
    return -1;
}

pub unsafe fn zmq_msg_set_routing_id(msg: &mut ZmqMsg, routing_id_: u32) {
    msg.set_routing_id(routing_id_ as i32);
}

pub unsafe fn zmq_msg_routing_id(msg: &mut ZmqMsg) -> u32 {
    msg.get_routing_id() as u32
}

pub unsafe fn zmq_msg_set_group(msg: &mut ZmqMsg, group_: &str) -> i32 {
    msg.set_group(group_)
}

pub unsafe fn zmq_msg_group<'a>(msg: &mut ZmqMsg) -> &'a str {
    msg.group().as_str()
}

pub unsafe fn zmq_msg_gets(msg: &mut ZmqMsg, property_: &str) -> &'static str {
    let metadata = msg.metadata();
    let value = metadata.get(property_);
    return value;
}

pub unsafe fn zmq_poller_poll(
    items_: &[ZmqPollitem],
    nitems_: i32,
    timeout_: i32,
) -> Result<usize, ZmqError> {
    // implement zmq_poll on top of zmq_poller
    // int rc;
    let mut rc = 0;
    // zmq_poller_event_t *events;
    let mut events: [ZmqPollerEvent; 1024] = [ZmqPollerEvent::default(); 1024];
    // zmq::socket_poller_t poller;
    let mut poller = ZmqSocketPoller::default();
    // events = new (std::nothrow) zmq_poller_event_t[nitems_];

    // alloc_assert (events);

    let mut repeat_items = false;
    //  Register sockets with poller
    // for (int i = 0; i < nitems_; i++)
    for i in 0..nitems_ {
        items_[i].revents = 0;

        let mut modify = false;
        let e = items_[i].events;
        if (items_[i].socket) {
            //  Poll item is a 0MQ socket.
            // for (int j = 0; j < i; ++j)
            for j in 0..i {
                // Check for repeat entries
                if (items_[j].socket == items_[i].socket) {
                    repeat_items = true;
                    modify = true;
                    e |= items_[j].events;
                }
            }
            if (modify) {
                zmq_poller_modify(&poller, items_[i].socket, e)?;
            } else {
                zmq_poller_add(&poller, items_[i].socket, &mut [0u8; 1], e)?;
            }
            // if rc < 0 {
            //     // delete[] events;
            //     return rc;
            // }
        } else {
            //  Poll item is a raw file descriptor.
            // for (int j = 0; j < i; ++j)
            for j in 0..i {
                // Check for repeat entries
                if (!items_[j].socket && items_[j].fd == items_[i].fd) {
                    repeat_items = true;
                    modify = true;
                    e |= items_[j].events;
                }
            }
            if (modify) {
                zmq_poller_modify_fd(&poller, items_[i].fd, e)?;
            } else {
                zmq_poller_add_fd(&poller, items_[i].fd, &mut [0u8; 1], e)?;
            }
            // if (rc < 0) {
            //     // delete[] events;
            //     return rc;
            // }
        }
    }

    //  Wait for events
    rc = zmq_poller_wait_all(&poller, &mut events, nitems_, timeout_)?;
    // if (rc < 0) {
    //     delete
    //     []
    //     events;
    //     if (zmq_errno() == EAGAIN) {
    //         return 0;
    //     }
    //     return rc;
    // }

    //  Transform poller events into zmq_pollitem events.
    //  items_ contains all items, while events only contains fired events.
    //  If no sockets are repeated (likely), the two are still co-ordered, so step through the items
    //  checking for matches only on the first event.
    //  If there are repeat items, they cannot be assumed to be co-ordered,
    //  so each pollitem must check fired events from the beginning.
    // int j_start = 0, found_events = rc;
    let mut j_start = 0;
    let mut found_events = rc;

    // for (int i = 0; i < nitems_; i++)
    for i in 0..nitems_ {
        // for (int j = j_start; j < found_events; ++j)
        for j in j_start..found_events {
            if ((items_[i].socket && items_[i].socket == events[j].socket)
                || (!(items_[i].socket || events[j].socket) && items_[i].fd == events[j].fd))
            {
                items_[i].revents = events[j].events & items_[i].events;
                if (!repeat_items) {
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
    return Ok(rc);
}

// int zmq_poll (zmq_pollitem_t *items_, int nitems_, long timeout_)
pub unsafe fn zmq_poll(
    items_: &[ZmqPollitem],
    nitems_: i32,
    timeout_: i32,
) -> Result<usize, ZmqError> {
    // #if defined ZMQ_HAVE_POLLER
    // if poller is present, use that if there is at least 1 thread-safe socket,
    // otherwise fall back to the previous implementation as it's faster.
    // for (int i = 0; i != nitems_; i++)
    for i in 0..nitems_ {
        if (items_[i].socket) {
            let sock = as_socket_base_t(items_[i].socket);
            if (sock.is_some()) {
                if (sock.unwrap().is_thread_safe()) {
                    return zmq_poller_poll(items_, nitems_, timeout_);
                }
            } else {
                //as_socket_base_t returned NULL : socket is invalid
                return Err(SocketError("invalid socket"));
            }
        }
    }
    // #endif // ZMQ_HAVE_POLLER
    // #if defined ZMQ_POLL_BASED_ON_POLL || defined ZMQ_POLL_BASED_ON_SELECT
    if (nitems_ < 0) {
        // errno = EINVAL;
        return Err(SocketError("EINVAL"));
    }
    if (nitems_ == 0) {
        if (timeout_ == 0) {
            return Ok(0);
        }
        // #if defined ZMQ_HAVE_WINDOWS
        #[cfg(target_os = "windows")]
        {
            Sleep(if timeout_ > 0 { timeout_ } else { INFINITE } as u32);
            return Ok(0);
        }
        // #elif defined ZMQ_HAVE_VXWORKS
        //         struct timespec ns_;
        //         ns_.tv_sec = timeout_ / 1000;
        //         ns_.tv_nsec = timeout_ % 1000 * 1000000;
        //         return nanosleep (&ns_, 0);
        // #else
        #[cfg(not(target_os = "windows"))]
        {
            return Ok(usleep((timeout_ * 1000) as c_uint) as usize);
        }
        // #endif
    }
    if !items_ {
        // errno = EFAULT;
        return Err(SocketError("EFAULT"));
    }

    // zmq::clock_t clock;
    let mut clock = ZmqClock::default();
    let mut now = 0u64;
    let mut end = 0u64;
    // #if defined ZMQ_POLL_BASED_ON_POLL
    //     zmq::fast_vector_t<pollfd, ZMQ_POLLITEMS_DFLT> pollfds (nitems_);
    let pollfds: Vec<ZmqPollFd> = vec![];
    #[cfg(feature = "poll")]
    {
        //  Build pollset for poll () system call.
        // for (int i = 0; i != nitems_; i++)
        for i in 0..nitems_ {
            //  If the poll item is a 0MQ socket, we poll on the file descriptor
            //  retrieved by the ZMQ_FD socket option.
            if (items_[i].socket) {
                let zmq_fd_size = size_of::<ZmqFd>();
                let fd_bytes = zmq_getsockopt(items_[i].socket, ZMQ_FD)?;
                let fd = u32::from_le_bytes([fd_bytes[0], fd_bytes[1], fd_bytes[2], fd_bytes[3]]);
                pollfds[i].fd = fd;
                // if zmq_getsockopt(items_[i].socket, ZMQ_FD, &pollfds[i].fd,
                //                   &zmq_fd_size) == -1 {
                //     return -1;
                // }
                pollfds[i].events = if items_[i].events { POLLIN } else { 0 };
            }
            //  Else, the poll item is a raw file descriptor. Just convert the
            //  events to normal POLLIN/POLLOUT for poll ().
            else {
                pollfds[i].fd = items_[i].fd;
                pollfds[i].events = (if items_[i].events & ZMQ_POLLIN {
                    POLLIN
                } else {
                    0
                }) | (if items_[i].events & ZMQ_POLLOUT {
                    POLLOUT
                } else {
                    0
                }) | (if items_[i].events & ZMQ_POLLPRI {
                    POLLPRI
                } else {
                    0
                });
            }
        }
    }
    // #else # [cfg(not(feature = "poll"))
    {
        //  Ensure we do not attempt to select () on more than FD_SETSIZE
        //  file descriptors.
        //  TODO since this function is called by a client, we could return errno EINVAL/ENOMEM/... here
        // zmq_assert (nitems_ <= FD_SETSIZE);

        // zmq::optimized_fd_set_t pollset_in (nitems_);
        let pollset_in = OptimizedFdSet::new(nitems_);
        // FD_ZERO (pollset_in.get ());
        FD_ZERO(pollset_in.get());
        let pollset_out = OptimizedFdSet::new(nitems_);
        FD_ZERO(pollset_out.get());
        let pollset_err = OptimizedFdSet::new(nitems_);
        FD_ZERO(pollset_err.get());

        let mut maxfd: ZmqFd = RETIRED_FD as ZmqFd;

        //  Build the fd_sets for passing to select ().
        // for (int i = 0; i != nitems_; i++)
        for i in 0..nitems_ {
            //  If the poll item is a 0MQ socket we are interested in input on the
            //  notification file descriptor retrieved by the ZMQ_FD socket option.
            if items_[i].socket {
                let zmq_fd_size = size_of::<ZmqFd>();
                let mut notify_fd: ZmqFd = RETIRED_FD as ZmqFd;

                // if (zmq_getsockopt(items_[i].socket, ZMQ_FD, &notify_fd,
                //                    &zmq_fd_size) == -1) {
                //     return -1;
                // }
                let fd_bytes = zmq_getsockopt(items_[i].socket, ZMQ_FD)?;
                notify_fd = u32::from_le_bytes([fd_bytes[0], fd_bytes[1], fd_bytes[2], fd_bytes[3]])
                    as ZmqFd;
                if items_[i].events {
                    FD_SET(notify_fd, pollset_in.get());
                    if maxfd < notify_fd {
                        maxfd = notify_fd;
                    }
                }
            }
            //  Else, the poll item is a raw file descriptor. Convert the poll item
            //  events to the appropriate fd_sets.
            else {
                if items_[i].events & ZMQ_POLLIN {
                    FD_SET(items_[i].fd, pollset_in.get());
                }
                if items_[i].events & ZMQ_POLLOUT {
                    FD_SET(items_[i].fd, pollset_out.get());
                }
                if items_[i].events & ZMQ_POLLERR {
                    FD_SET(items_[i].fd, pollset_err.get());
                }
                if maxfd < items_[i].fd {
                    maxfd = items_[i].fd;
                }
            }
        }

        let mut inset = OptimizedFdSet::new(nitems_);
        let mut outset = OptimizedFdSet::new(nitems_);
        let mut errset = OptimizedFdSet::new(nitems_);
    }
    // #endif

    let mut first_pass = true;
    let mut nevents = 0i32;

    loop {
        #[cfg(feature = "poll")]
        {
            // #if defined ZMQ_POLL_BASED_ON_POLL

            //  Compute the timeout for the subsequent poll.
            let timeout = compute_timeout(first_pass, timeout_, now, end);

            //  Wait for events.
            {
                zmq_poll_int(pollfds[0], nitems_ as u32, timeout as u32)?;
                // if (rc == -1 && errno == EINTR) {
                //     return -1;
                // }
                // errno_assert (rc >= 0);
            }
            //  Check for the events.
            // for (int i = 0; i != nitems_; i++)
            for i in 0..nitems_ {
                items_[i].revents = 0;

                //  The poll item is a 0MQ socket. Retrieve pending events
                //  using the ZMQ_EVENTS socket option.
                if (items_[i].socket) {
                    let mut zmq_events_size = 4;
                    let mut zmq_events: u32 = 0;
                    // if (zmq_getsockopt(items_[i].socket, ZMQ_EVENTS, &zmq_events,
                    //                    &zmq_events_size) == -1) {
                    //     return -1;
                    // }
                    let zmq_events_bytes = zmq_getsockopt(items_[i].socket, ZMQ_EVENTS)?;
                    zmq_events = u32::from_le_bytes([
                        zmq_events_bytes[0],
                        zmq_events_bytes[1],
                        zmq_events_bytes[2],
                        zmq_events_bytes[3],
                    ]);
                    if (items_[i].events & ZMQ_POLLOUT) && (zmq_events & ZMQ_POLLOUT) {
                        items_[i].revents |= ZMQ_POLLOUT;
                    }
                    if (items_[i].events & ZMQ_POLLIN) && (zmq_events & ZMQ_POLLIN) {
                        items_[i].revents |= ZMQ_POLLIN;
                    }
                }
                //  Else, the poll item is a raw file descriptor, simply convert
                //  the events to zmq_pollitem_t-style format.
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

                if (items_[i].revents) {
                    nevents += 1;
                }
            }
        }
        // #else
        #[cfg(not(feature = "poll"))]
        {
            //  Compute the timeout for the subsequent poll.
            let mut timeout = timeval {
                tv_sec: 0,
                tv_usec: 0,
            };
            let mut ptimeout: &timeval = &timeout;
            if (first_pass) {
                timeout.tv_sec = 0;
                timeout.tv_usec = 0;
                ptimeout = &timeout;
            } else if (timeout_ < 0) {
                ptimeout = null_mut();
            } else {
                timeout.tv_sec = ((end - now) / 1000);
                timeout.tv_usec = ((end - now) % 1000 * 1000);
                ptimeout = &timeout;
            }

            //  Wait for events. Ignore interrupts if there's infinite timeout.
            loop {
                libc::memcpy(
                    inset.get(),
                    pollset_in.get(),
                    valid_pollset_bytes(*pollset_in.get()),
                );
                libc::memcpy(
                    outset.get(),
                    pollset_out.get(),
                    valid_pollset_bytes(*pollset_out.get()),
                );
                libc::memcpy(
                    errset.get(),
                    pollset_err.get(),
                    valid_pollset_bytes(*pollset_err.get()),
                );
                // #if defined ZMQ_HAVE_WINDOWS
                #[cfg(target_os = "windows")]
                {
                    let rc = select(0, inset.get(), outset.get(), errset.get(), ptimeout);
                    if (unlikely(rc == SOCKET_ERROR)) {
                        errno = zmq::wsa_error_to_errno(WSAGetLastError());
                        wsa_assert(errno == ENOTSOCK);
                        return -1;
                    }
                }
                // #else
                #[cfg(not(target_os = "windows"))]
                {
                    let rc = select(maxfd + 1, inset.get(), outset.get(), errset.get(), ptimeout);
                    if (unlikely(rc == -1)) {
                        errno_assert(errno == EINTR || errno == EBADF);
                        return -1;
                    }
                }
                // #endif
                break;
            }

            //  Check for the events.
            // for (int i = 0; i != nitems_; i++)
            for i in 0..nitems_ {
                items_[i].revents = 0;

                //  The poll item is a 0MQ socket. Retrieve pending events
                //  using the ZMQ_EVENTS socket option.
                if (items_[i].socket) {
                    let zmq_events_size = 4;
                    let mut zmq_events = 0u32;
                    if (zmq_getsockopt(items_[i].socket, ZMQ_EVENTS, &zmq_events, &zmq_events_size)
                        == -1)
                    {
                        return -1;
                    }
                    if ((items_[i].events & ZMQ_POLLOUT) && (zmq_events & ZMQ_POLLOUT)) {
                        items_[i].revents |= ZMQ_POLLOUT;
                    }
                    if ((items_[i].events & ZMQ_POLLIN) && (zmq_events & ZMQ_POLLIN)) {
                        items_[i].revents |= ZMQ_POLLIN;
                    }
                }
                //  Else, the poll item is a raw file descriptor, simply convert
                //  the events to zmq_pollitem_t-style format.
                else {
                    if (FD_ISSET(items_[i].fd, inset.get())) {
                        items_[i].revents |= ZMQ_POLLIN;
                    }
                    if (FD_ISSET(items_[i].fd, outset.get())) {
                        items_[i].revents |= ZMQ_POLLOUT;
                    }
                    if (FD_ISSET(items_[i].fd, errset.get())) {
                        items_[i].revents |= ZMQ_POLLERR;
                    }
                }

                if (items_[i].revents) {
                    nevents += 1;
                }
            }
        }
        // #endif

        //  If timeout is zero, exit immediately whether there are events or not.
        if (timeout_ == 0) {
            break;
        }

        //  If there are events to return, we can exit immediately.
        if (nevents) {
            break;
        }

        //  At this point we are meant to wait for events but there are none.
        //  If timeout is infinite we can just loop until we get some events.
        if (timeout_ < 0) {
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
            end = now + timeout_;
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

    return Ok(nevents as usize);
    // #else
    //  Exotic platforms that support neither poll() nor select().
    // errno = ENOTSUP;
    // return -1;
    // #endif
}

pub unsafe fn zmq_poll_check_items_(
    poll_items: &mut [ZmqPollitem],
    num_items: i32,
    timeout_: i32,
) -> i32 {
    if num_items < 0 {
        // errno = EINVAL;
        return -1;
    }
    if num_items == 0 {
        if timeout_ == 0 {
            return 0;
        }
        // #if defined ZMQ_HAVE_WINDOWS
        #[cfg(target_os = "windows")]
        {
            Sleep(if timeout_ > 0 { timeout_ } else { INFINITE } as u32);
            return 0;
        }
        // #elif defined ZMQ_HAVE_VXWORKS
        //         struct timespec ns_;
        //         ns_.tv_sec = timeout_ / 1000;
        //         ns_.tv_nsec = timeout_ % 1000 * 1000000;
        //         return nanosleep (&ns_, 0);
        // #else
        #[cfg(not(target_os = "windows"))]
        {
            return usleep((timeout_ * 1000) as c_uint);
        }
        // #endif
    }
    if !poll_items {
        // errno = EFAULT;
        return -1;
    }
    return 1;
}

pub struct zmq_poll_select_fds_t_ {
    // explicit zmq_poll_select_fds_t_ (int nitems_) :
    //     pollset_in (nitems_),
    //     pollset_out (nitems_),
    //     pollset_err (nitems_),
    //     inset (nitems_),
    //     outset (nitems_),
    //     errset (nitems_),
    //     maxfd (0)
    // {
    //     FD_ZERO (pollset_in.get ());
    //     FD_ZERO (pollset_out.get ());
    //     FD_ZERO (pollset_err.get ());
    // }

    // zmq::optimized_fd_set_t pollset_in;
    pub pollset_in: OptimizedFdSet,
    // zmq::optimized_fd_set_t pollset_out;
    pub pollset_out: OptimizedFdSet,
    // zmq::optimized_fd_set_t pollset_err;
    pub pollset_err: OptimizedFdSet,
    // zmq::optimized_fd_set_t inset;
    pub inset: OptimizedFdSet,
    // zmq::optimized_fd_set_t outset;
    pub outset: OptimizedFdSet,
    // zmq::optimized_fd_set_t errset;
    pub errset: OptimizedFdSet,
    // zmq::fd_t maxfd;
    pub maxfd: ZmqFd,
}

// zmq_poll_select_fds_t_
// zmq_poll_build_select_fds_ (zmq_pollitem_t *items_, int nitems_, int &rc)
pub unsafe fn zmq_poll_build_select_fds_(
    items_: &[ZmqPollitem],
    nitems_: i32,
    rc: &mut i32,
) -> zmq_poll_select_fds_t_ {
    //  Ensure we do not attempt to select () on more than FD_SETSIZE
    //  file descriptors.
    //  TODO since this function is called by a client, we could return errno EINVAL/ENOMEM/... here
    // zmq_assert (nitems_ <= FD_SETSIZE);

    let fds = zmq_poll_select_fds_t_::new(nitems_);

    //  Build the fd_sets for passing to select ().
    // for (int i = 0; i != nitems_; i++)
    for i in 0..nitems_ {
        //  If the poll item is a 0MQ socket we are interested in input on the
        //  notification file descriptor retrieved by the ZMQ_FD socket option.
        if (items_[i].socket) {
            let zmq_fd_size = size_of::<ZmqFd>();
            // zmq::fd_t notify_fd;
            let mut notify_fd: ZmqFd;
            // if (zmq_getsockopt(items_[i].socket, ZMQ_FD, &notify_fd,
            //                    &zmq_fd_size) == -1) {
            //     rc = -1;
            //     return fds;
            // }
            let fd_bytes = zmq_getsockopt(items_[i].socket, ZMQ_FD)?;
            notify_fd =
                u32::from_le_bytes([fd_bytes[0], fd_bytes[1], fd_bytes[2], fd_bytes[3]]) as ZmqFd;
            if (items_[i].events) {
                FD_SET(notify_fd, fds.pollset_in.get());
                if (fds.maxfd < notify_fd) {
                    fds.maxfd = notify_fd;
                }
            }
        }
        //  Else, the poll item is a raw file descriptor. Convert the poll item
        //  events to the appropriate fd_sets.
        else {
            if (items_[i].events & ZMQ_POLLIN) {
                FD_SET(items_[i].fd, fds.pollset_in.get());
            }
            if (items_[i].events & ZMQ_POLLOUT) {
                1
            }
            if (items_[i].events & ZMQ_POLLERR) {
                FD_SET(items_[i].fd, fds.pollset_err.get());
            }
            if (fds.maxfd < items_[i].fd) {
                fds.maxfd = items_[i].fd;
            }
        }
    }

    *rc = 0;
    return fds;
}

// timeval *zmq_poll_select_set_timeout_ (
//   long timeout_, bool first_pass, uint64_t now, uint64_t end, timeval &timeout)
pub unsafe fn zmq_poll_select_set_timeout_(
    timeout_: i32,
    first_pass: bool,
    now: u64,
    end: u64,
    timeout: &mut timeval,
) -> &mut timeval {
    // timeval *ptimeout;
    let ptimeout: &mut timeval;
    if (first_pass) {
        timeout.tv_sec = 0;
        timeout.tv_usec = 0;
        ptimeout = timeout;
    } else if (timeout_ < 0) {
        ptimeout = &mut timeval {
            tv_sec: 0,
            tv_usec: 0,
        };
    } else {
        timeout.tv_sec = ((end - now) / 1000) as c_long;
        timeout.tv_usec = ((end - now) % 1000 * 1000) as c_long;
        ptimeout = timeout;
    }
    return ptimeout;
}

// timespec *zmq_poll_select_set_timeout_2 (
//   long timeout_, bool first_pass, uint64_t now, uint64_t end, timespec &timeout)
pub unsafe fn zmq_poll_select_set_timeout_2(
    timeout_: i32,
    first_pass: bool,
    now: u64,
    end: u64,
    timeout: &mut timespec,
) -> &mut timespec {
    // timespec * ptimeout;
    let mut ptimeout: &mut timespec;
    if (first_pass) {
        timeout.tv_sec = 0;
        timeout.tv_nsec = 0;
        ptimeout = timeout;
    } else if (timeout_ < 0) {
        ptimeout = &mut timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
    } else {
        timeout.tv_sec = ((end - now) / 1000) as time_t;
        timeout.tv_nsec = ((end - now) % 1000 * 1000000) as c_long;
        ptimeout = timeout;
    }
    return ptimeout;
}

// int zmq_poll_select_check_events_ (zmq_pollitem_t *items_,
//                                    int nitems_,
//                                    zmq_poll_select_fds_t_ &fds,
//                                    int &nevents)
pub unsafe fn zmq_poll_select_check_events_(
    items_: &[ZmqPollitem],
    nitems_: i32,
    fds: &mut zmq_poll_select_fds_t_,
    nevents: &mut i32,
) -> i32 {
    //  Check for the events.
    // for (int i = 0; i != nitems_; i++)
    for i in 0..nitems_ {
        items_[i].revents = 0;

        //  The poll item is a 0MQ socket. Retrieve pending events
        //  using the ZMQ_EVENTS socket option.
        if (items_[i].socket) {
            let zmq_events_size = 4;
            let zmq_events: u32;
            // if (zmq_getsockopt(items_[i].socket, ZMQ_EVENTS, &zmq_events,
            //                    &zmq_events_size) == -1) {
            //     return -1;
            // }
            let zmq_events_bytes = zmq_getsockopt(items_[i].socket, ZMQ_EVENTS)?;
            zmq_events = u32::from_le_bytes([
                zmq_events_bytes[0],
                zmq_events_bytes[1],
                zmq_events_bytes[2],
                zmq_events_bytes[3],
            ]);
            if ((items_[i].events & ZMQ_POLLOUT) & &(zmq_events & ZMQ_POLLOUT)) {
                items_[i].revents |= ZMQ_POLLOUT;
            }
            if ((items_[i].events & ZMQ_POLLIN) & &(zmq_events & ZMQ_POLLIN)) {
                items_[i].revents |= ZMQ_POLLIN;
            }
        }
        //  Else, the poll item is a raw file descriptor, simply convert
        //  the events to zmq_pollitem_t-style format.
        else {
            if (FD_ISSET(items_[i].fd, fds.inset.get())) {
                items_[i].revents |= ZMQ_POLLIN;
            }
            if (FD_ISSET(items_[i].fd, fds.outset.get())) {
                items_[i].revents |= ZMQ_POLLOUT;
            }
            if (FD_ISSET(items_[i].fd, fds.errset.get())) {
                items_[i].revents |= ZMQ_POLLERR;
            }
        }

        if (items_[i].revents) {
            *nevents += 1;
        }
    }

    return 0;
}

pub unsafe fn zmq_poll_must_break_loop_(
    timeout_: i32,
    nevents: i32,
    first_pass: &mut bool,
    clock: &mut ZmqClock,
    now: &mut u64,
    end: &mut u64,
) -> bool {
    //  If timeout is zero, exit immediately whether there are events or not.
    if (timeout_ == 0) {
        return true;
    }

    //  If there are events to return, we can exit immediately.
    if (nevents) {
        return true;
    }

    //  At this point we are meant to wait for events but there are none.
    //  If timeout is infinite we can just loop until we get some events.
    if (timeout_ < 0) {
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
        *end = *now + timeout_;
        if now == end {
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

// #if !defined _WIN32
// int zmq_ppoll (zmq_pollitem_t *items_,
//                int nitems_,
//                long timeout_,
//                const sigset_t *sigmask_)
// #else
// // Windows has no sigset_t
// int zmq_ppoll (zmq_pollitem_t *items_,
//                int nitems_,
//                long timeout_,
//                const void *sigmask_)
// #endif
#[cfg(not(target_os = "windows"))]
pub unsafe fn zmq_ppoll(
    items_: &mut [ZmqPollitem],
    nitems_: i32,
    timeout_: i32,
    sigmask_: &sigset_t,
) -> i32 {
    // #ifdef ZMQ_HAVE_PPOLL
    let mut rc = zmq_poll_check_items_(items_, nitems_, timeout_);
    if (rc <= 0) {
        return rc;
    }

    let mut clock: ZmqClock = ZmqClock {
        last_tsc: 0,
        last_time: 0,
    };
    let mut now = 0;
    let mut end = 0;
    let mut fds = zmq_poll_build_select_fds_(items_, nitems_, &mut rc);
    if rc == -1 {
        return -1;
    }

    let mut first_pass = true;
    let mut nevents = 0;

    loop {
        //  Compute the timeout for the subsequent poll.
        // timespec timeout;
        let mut timeout = timeval {
            tv_sec: 0,
            // tv_nsec: 0,
            tv_usec: 0,
        };
        let ptimeout = zmq_poll_select_set_timeout_(timeout_, first_pass, now, end, &mut timeout);
        let mut ptimeout_spec = &mut timespec {
            tv_sec: timeout.tv_sec,
            tv_nsec: timeout.tv_usec * 1000,
        };

        //  Wait for events. Ignore interrupts if there's infinite timeout.
        loop {
            libc::memcpy(
                fds.inset.get(),
                fds.pollset_in.get(),
                valid_pollset_bytes(*fds.pollset_in.get()),
            );
            libc::memcpy(
                fds.outset.get(),
                fds.pollset_out.get(),
                valid_pollset_bytes(*fds.pollset_out.get()),
            );
            libc::memcpy(
                fds.errset.get(),
                fds.pollset_err.get(),
                valid_pollset_bytes(*fds.pollset_err.get()),
            );
            let mut rc = pselect(
                fds.maxfd + 1,
                fds.inset.get(),
                fds.outset.get(),
                fds.errset.get(),
                ptimeout_spec,
                sigmask_,
            );
            if rc == -1 {
                // errno_assert (errno == EINTR || errno == EBADF);
                return -1;
            }
            break;
        }

        rc = zmq_poll_select_check_events_(items_, nitems_, &mut fds, &mut nevents);
        if rc < 0 {
            return rc;
        }

        if zmq_poll_must_break_loop_(
            timeout_,
            nevents,
            &mut first_pass,
            &mut clock,
            &mut now,
            &mut end,
        ) {
            break;
        }
    }

    return nevents;
    // #else
    //     errno = ENOTSUP;
    //     return -1;
    // #endif // ZMQ_HAVE_PPOLL
}

// void *zmq_poller_new (void)
pub unsafe fn zmq_poller_new() -> ZmqSocketPoller {
    // zmq::socket_poller_t *poller = new (std::nothrow) zmq::socket_poller_t;
    // if (!poller) {
    //     errno = ENOMEM;
    // }
    // return poller;
    ZmqSocketPoller::new()
}

// int zmq_poller_destroy (void **poller_p_)
pub unsafe fn zmq_poller_destroy(poller_p_: &mut ZmqSocketPoller) -> i32 {
    // if (poller_p_) {
    //     const zmq::socket_poller_t *const poller =
    //       static_cast<const zmq::socket_poller_t *> (*poller_p_);
    //     if (poller && poller->check_tag ()) {
    //         delete poller;
    //         *poller_p_ = NULL;
    //         return 0;
    //     }
    // }
    // errno = EFAULT;
    // return -1;
    todo!()
}

// static int check_poller (void *const poller_)
pub unsafe fn check_poller(poller_: &mut ZmqSocketPoller) -> Result<(), ZmqError> {
    // if (!poller_
    //     || !(static_cast<zmq::socket_poller_t *> (poller_))->check_tag ()) {
    //     errno = EFAULT;
    //     return -1;
    // }
    //
    // return 0;
    if !poller_.check_tag() {
        return Err(SocketError("poller check_tag failed"));
    } else {
        return Ok(());
    }
}

// static int check_events (const short events_)
pub unsafe fn check_events(events_: i16) -> i32 {
    if (events_ & !(ZMQ_POLLIN | ZMQ_POLLOUT | ZMQ_POLLERR | ZMQ_POLLPRI)) {
        // errno = EINVAL;
        return -1;
    }
    return 0;
}

// static int check_poller_registration_args (void *const poller_, void *const s_)
pub unsafe fn check_poller_registration_args(
    poller: &mut ZmqSocketPoller,
    sock: &mut ZmqSocket,
) -> i32 {
    if -1 == check_poller(poller) {
        return -1;
    }

    // if (!s_ || !(static_cast<zmq::socket_base_t *> (s_))->check_tag ()) {
    //     errno = ENOTSOCK;
    //     return -1;
    // }
    if !sock.check_tag() {
        return -1;
    }

    return 0;
}

// static int check_poller_fd_registration_args (void *const poller_,
//                                               const zmq::fd_t fd_)
pub unsafe fn check_poller_fd_registration_args(poller_: &mut ZmqSocketPoller, fd_: ZmqFd) -> i32 {
    if -1 == check_poller(poller_) {
        return -1;
    }

    if fd_ == RETIRED_FD {
        // errno = EBADF;
        return -1;
    }

    return 0;
}

// int zmq_poller_size (void *poller_)
pub unsafe fn zmq_poller_size(poller_: &mut ZmqSocketPoller) -> i32 {
    if -1 == check_poller(poller_) {
        return -1;
    }

    // return (static_cast<zmq::socket_poller_t *> (poller_))->size ();
    poller_.size()
}

// int zmq_poller_add (void *poller_, void *s_, void *user_data_, short events_)
pub unsafe fn zmq_poller_add(
    poller_: &mut ZmqSocketPoller,
    s_: &mut ZmqSocket,
    user_data_: &mut [u8],
    events_: i16,
) -> i32 {
    if -1 == check_poller_registration_args(poller_, s_) || -1 == check_events(events_) {
        return -1;
    }

    // zmq::socket_base_t *socket = static_cast<zmq::socket_base_t *> (s_);

    // return (static_cast<zmq::socket_poller_t *> (poller_))
    //   ->add (socket, user_data_, events_);
    poller_.add(s_, user_data_, events_)
}

// int zmq_poller_add_fd (void *poller_,
//                        zmq::fd_t fd_,
//                        void *user_data_,
//                        short events_)
pub unsafe fn zmq_poller_add_fd(
    poller_: &mut ZmqSocketPoller,
    fd_: ZmqFd,
    user_data_: &mut [u8],
    events_: i16,
) -> i32 {
    if (-1 == check_poller_fd_registration_args(poller_, fd_) || -1 == check_events(events_)) {
        return -1;
    }

    // return (static_cast<zmq::socket_poller_t *> (poller_))
    //   ->add_fd (fd_, user_data_, events_);
    poller_.add_fd(fd_, user_data_, events_)
}

// int zmq_poller_modify (void *poller_, void *s_, short events_)
pub unsafe fn zmq_poller_modify(
    poller_: &mut ZmqSocketPoller,
    s_: &mut ZmqSocket,
    events_: i16,
) -> i32 {
    if -1 == check_poller_registration_args(poller_, s_) || -1 == check_events(events_) {
        return -1;
    }

    // const zmq::socket_base_t *const socket =
    //   static_cast<const zmq::socket_base_t *> (s_);

    // return (static_cast<zmq::socket_poller_t *> (poller_))
    //   ->modify (socket, events_);
    poller_.modify(s_, events_)
}

// int zmq_poller_modify_fd (void *poller_, zmq::fd_t fd_, short events_)
pub unsafe fn zmq_poller_modify_fd(poller_: &mut ZmqSocketPoller, fd_: ZmqFd, events_: i16) -> i32 {
    if (-1 == check_poller_fd_registration_args(poller_, fd_) || -1 == check_events(events_)) {
        return -1;
    }

    // return (static_cast<zmq::socket_poller_t *> (poller_))
    //   ->modify_fd (fd_, events_);
    poller_.modify_fd(fd_, events_)
}

// int zmq_poller_remove (void *poller_, void *s_)
pub unsafe fn zmq_poller_remove(poller_: &mut ZmqSocketPoller, s_: &mut ZmqSocket) -> i32 {
    if (-1 == check_poller_registration_args(poller_, s_)) {
        return -1;
    }

    // zmq::socket_base_t *socket = static_cast<zmq::socket_base_t *> (s_);

    // return (static_cast<zmq::socket_poller_t *> (poller_))->remove (socket);
    poller_.remove(s_)
}

// int zmq_poller_remove_fd (void *poller_, zmq::fd_t fd_)
pub unsafe fn zmq_poller_remove_fd(poller_: &mut ZmqSocketPoller, fd_: ZmqFd) -> i32 {
    if (-1 == check_poller_fd_registration_args(poller_, fd_)) {
        return -1;
    }

    // return (static_cast<zmq::socket_poller_t *> (poller_))->remove_fd (fd_);
    poller_.remove_fd(fd_)
}

// int zmq_poller_wait (void *poller_, zmq_poller_event_t *event_, long timeout_)
pub unsafe fn zmq_poller_wait(
    poller_: &mut ZmqSocketPoller,
    event_: &mut ZmqPollerEvent,
    timeout_: i32,
) -> Result<usize, ZmqError> {
    zmq_poller_wait_all(poller_, &mut [event_.clone()], 1, timeout_)

    // if (rc < 0) {
    //     // event_->socket = NULL;
    //     // event_->fd = zmq::retired_fd;
    //     // event_->user_data = NULL;
    //     // event_->events = 0;
    // }
    // // wait_all returns number of events, but we return 0 for any success
    // return if rc >= 0 { 0 } else { rc };
}

// int zmq_poller_wait_all (void *poller_,
//                          zmq_poller_event_t *events_,
//                          int n_events_,
//                          long timeout_)
pub unsafe fn zmq_poller_wait_all(
    poller_: &mut ZmqSocketPoller,
    events_: &mut [ZmqPollerEvent],
    n_events_: i32,
    timeout_: i32,
) -> Result<usize, ZmqError> {
    // if -1 == check_poller(poller_) {
    //     return -1;
    // }
    check_poller(poller_)?;

    if !events_ {
        // errno = EFAULT;
        // return -1;
        return Err(PollerError("EFAULT"));
    }
    if n_events_ < 0 {
        // errno = EINVAL;
        // return -1;
        return Err(PollerError("EINVAL"));
    }

    let rc = poller_.wait(events_, n_events_, timeout_);

    return Ok(rc as usize);
}

// int zmq_poller_fd (void *poller_, zmq_fd_t *fd_)
pub unsafe fn zmq_poller_fd(poller_: &mut ZmqSocketPoller, fd_: &mut zmq_fd_t) -> i32 {
    // if (!poller_
    //     || !(static_cast<zmq::socket_poller_t *> (poller_)->check_tag ())) {
    //     errno = EFAULT;
    //     return -1;
    // }
    if !poller_.check_tag() {
        return -1;
    }
    return (poller_).signaler_fd(fd_);
}

// int zmq_socket_get_peer_state (void *s_,
//                                const void *routing_id_,
//                                size_t routing_id_size_)
pub unsafe fn zmq_socket_get_peer_state(
    s_: &mut ZmqSocket,
    routing_id_: &c_void,
    routing_id_size_: usize,
) -> i32 {
    // const zmq::socket_base_t *const s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;

    // return s->get_peer_state (routing_id_, routing_id_size_);
    s_.get_peer_state(routing_id_, routing_id_size_)
}

// void *zmq_timers_new (void)
pub unsafe fn zmq_timers_new() -> Timers {
    // zmq::timers_t *timers = new (std::nothrow) zmq::timers_t;
    // alloc_assert (timers);
    // return timers;
    Timers::new()
}

// int zmq_timers_destroy (void **timers_p_)
pub unsafe fn zmq_timers_destroy(timers_p_: &mut Timers) -> i32 {
    // void *timers = *timers_p_;
    // if (!timers || !(static_cast<zmq::timers_t *> (timers))->check_tag ()) {
    //     errno = EFAULT;
    //     return -1;
    // }
    // delete (static_cast<zmq::timers_t *> (timers));
    // *timers_p_ = NULL;
    // return 0;
    todo!()
}

pub type zmq_timer_fn = fn();

// int zmq_timers_add (void *timers_,
//                     size_t interval_,
//                     zmq_timer_fn handler_,
//                     void *arg_)
pub unsafe fn zmq_timers_add(
    timers_: &mut Timers,
    interval_: usize,
    handler_: TimersTimerFn,
    arg_: &mut [u8],
) -> Result<(), ZmqError> {
    // if (!timers_ || !(static_cast<zmq::timers_t *> (timers_))->check_tag ()) {
    //     errno = EFAULT;
    //     return -1;
    // }
    if !timers_.check_tag() {
        return Err(TimerError("tag check failed"));
    }

    // return (static_cast<zmq::timers_t *> (timers_))
    //   ->add (interval_, handler_, arg_);
    match timers_.add(interval_ as i32, handler_, arg_) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

// int zmq_timers_cancel (void *timers_, int timer_id_)
pub unsafe fn zmq_timers_cancel(timers_: &mut Timers, timer_id_: i32) -> i32 {
    // if (!timers_ || !(static_cast<zmq::timers_t *> (timers_))->check_tag ()) {
    //     errno = EFAULT;
    //     return -1;
    // }
    if !timers_.check_tag() {
        return -1;
    }

    // return (static_cast<zmq::timers_t *> (timers_))->cancel (timer_id_);
    timers_.cancel(timer_id_)
}

// int zmq_timers_set_interval (void *timers_, int timer_id_, size_t interval_)
pub unsafe fn zmq_timers_set_interval(
    timers_: &mut Timers,
    timer_id_: i32,
    interval_: usize,
) -> i32 {
    // if (!timers_ || !(static_cast<zmq::timers_t *> (timers_))->check_tag ()) {
    //     errno = EFAULT;
    //     return -1;
    // }
    if !timers_.check_tag() {
        return -1;
    }

    // return (static_cast<zmq::timers_t *> (timers_))
    //   ->set_interval (timer_id_, interval_);
    timers_.set_interval(timer_id_, interval_)
}

// int zmq_timers_reset (void *timers_, int timer_id_)
pub unsafe fn zmq_timers_reset(timers_: &mut Timers, timer_id_: i32) -> i32 {
    // if (!timers_ || !(static_cast<zmq::timers_t *> (timers_))->check_tag ()) {
    //     errno = EFAULT;
    //     return -1;
    // }
    if !timers_.check_tag() {
        return -1;
    }

    // return (static_cast<zmq::timers_t *> (timers_))->reset (timer_id_);
    timers_.reset(timer_id_)
}

// long zmq_timers_timeout (void *timers_)
pub unsafe fn zmq_timers_timeout(timers_: &mut Timers) -> i32 {
    // if (!timers_ || !(static_cast<zmq::timers_t *> (timers_))->check_tag ()) {
    //     errno = EFAULT;
    //     return -1;
    // }
    if !timers_.check_tag() {
        return -1;
    }

    // return (static_cast<zmq::timers_t *> (timers_))->timeout ();
    timers_.timeout()
}

// int zmq_timers_execute (void *timers_)
pub unsafe fn zmq_timers_execute(timers_: &mut Timers) -> i32 {
    // if (!timers_ || !(static_cast<zmq::timers_t *> (timers_))->check_tag ()) {
    //     errno = EFAULT;
    //     return -1;
    // }
    if !timers_.check_tag() {
        return -1;
    }

    // return (static_cast<zmq::timers_t *> (timers_))->execute ();
    timers_.execute()
}

// int zmq_proxy (void *frontend_, void *backend_, void *capture_)
pub unsafe fn zmq_proxy(
    frontend_: &mut ZmqSocket,
    backend_: &mut ZmqSocket,
    capture_: &mut ZmqSocket,
) -> i32 {
    if (!frontend_ || !backend_) {
        // errno = EFAULT;
        return -1;
    }
    return proxy((frontend_), (backend_), Some(capture_));
}

// int zmq_proxy_steerable (void *frontend_,
//                          void *backend_,
//                          void *capture_,
//                          void *control_)
pub unsafe fn zmq_proxy_steerable(
    frontend_: &mut ZmqSocket,
    backend_: &mut ZmqSocket,
    capture_: &mut ZmqSocket,
    control_: &mut ZmqSocket,
) -> i32 {
    // if (!frontend_ || !backend_ || !control_) {
    //     errno = EFAULT;
    //     return -1;
    // }
    // return proxy_steerable ( (frontend_),
    //                                 (backend_),
    //                                 (capture_),
    //                                 (control_));
    return -1;
}
// {
//     if (!frontend_ || !backend_) {
//         errno = EFAULT;
//         return -1;
//     }
// #ifdef ZMQ_HAVE_WINDOWS
//     errno = WSAEOPNOTSUPP;
// #else
//     errno = EOPNOTSUPP;
// #endif
//   return -1;
// }

// int zmq_device (int /* type */, void *frontend_, void *backend_)
pub unsafe fn zmq_device(type_: i32, frontend_: &mut ZmqSocket, backend_: &mut ZmqSocket) -> i32 {
    return proxy((frontend_), (backend_), None);
}

// int zmq_has (const char *capability_)
pub unsafe fn zmq_has(capability_: &str) -> i32 {
    todo!()
    // #if defined(ZMQ_HAVE_IPC)
    //     if (strcmp (capability_, zmq::protocol_name::ipc) == 0)
    //         return true;
    // #endif
    // #if defined(ZMQ_HAVE_OPENPGM)
    //     if (strcmp (capability_, zmq::protocol_name::pgm) == 0)
    //         return true;
    // #endif
    // #if defined(ZMQ_HAVE_TIPC)
    //     if (strcmp (capability_, zmq::protocol_name::tipc) == 0)
    //         return true;
    // #endif
    // #if defined(ZMQ_HAVE_NORM)
    //     if (strcmp (capability_, zmq::protocol_name::norm) == 0)
    //         return true;
    // #endif
    // #if defined(ZMQ_HAVE_CURVE)
    //     if (strcmp (capability_, "curve") == 0)
    //         return true;
    // #endif
    // #if defined(HAVE_LIBGSSAPI_KRB5)
    //     if (strcmp (capability_, "gssapi") == 0)
    //         return true;
    // #endif
    // #if defined(ZMQ_HAVE_VMCI)
    //     if (strcmp (capability_, zmq::protocol_name::vmci) == 0)
    //         return true;
    // #endif
    // #if defined(ZMQ_BUILD_DRAFT_API)
    //     if (strcmp (capability_, "draft") == 0)
    //         return true;
    // #endif
    // #if defined(ZMQ_HAVE_WS)
    //     if (strcmp (capability_, "WS") == 0)
    //         return true;
    // #endif
    // #if defined(ZMQ_HAVE_WSS)
    //     if (strcmp (capability_, "WSS") == 0)
    //         return true;
    // #endif
    //     //  Whatever the application asked for, we don't have
    //     return false;
}

// int zmq_socket_monitor_pipes_stats (void *s_)
pub unsafe fn zmq_socket_monitor_pipes_stats(s_: &mut ZmqSocket) -> i32 {
    // zmq::socket_base_t *s = as_socket_base_t (s_);
    // if (!s)
    //     return -1;
    // return s->query_pipes_stats ();
    s_.query_pipes_stats()
}
