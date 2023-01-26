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

    *************************************************************************
    NOTE to contributors. This file comprises the principal public contract
    for ZeroMQ API users. Any change to this file supplied in a stable
    release SHOULD not break existing applications.
    In practice this means that the value of constants must not change, and
    that old values may not be reused for new constants.
    *************************************************************************
*/

use libc::c_void;

// #ifndef __ZMQ_H_INCLUDED__
// #define __ZMQ_H_INCLUDED__

/*  Version macros for compile-time API version detection                     */
// #define ZMQ_VERSION_MAJOR 4
pub const ZMQ_VERSION_MAJOR: u32 = 4;
// #define ZMQ_VERSION_MINOR 3
pub const ZMQ_VERSION_MINOR: u32 = 4;
// #define ZMQ_VERSION_PATCH 5
pub const ZMQ_VERSION_PATCH: u32 = 5;

// #define ZMQ_MAKE_VERSION(major, minor, patch)                                  \
//     ((major) *10000 + (minor) *100 + (patch))
// #define ZMQ_VERSION                                                            \
//     ZMQ_MAKE_VERSION (ZMQ_VERSION_MAJOR, ZMQ_VERSION_MINOR, ZMQ_VERSION_PATCH)

// #ifdef __cplusplus
// extern "C" {
// #endif

// #if !defined _WIN32_WCE
// #include <errno.h>
// #endif
// #include <stddef.h>
// #include <stdio.h>

/*  Handle DSO symbol visibility                                             */
// #if defined ZMQ_NO_EXPORT
// #define ZMQ_EXPORT
// #else
// #if defined _WIN32
// #if defined ZMQ_STATIC
// #define ZMQ_EXPORT
// #elif defined DLL_EXPORT
// #define ZMQ_EXPORT __declspec(dllexport)
// #else
// #define ZMQ_EXPORT __declspec(dllimport)
// #endif
// #else
// #if defined __SUNPRO_C || defined __SUNPRO_CC
// #define ZMQ_EXPORT __global
// #elif (defined __GNUC__ && __GNUC__ >= 4) || defined __INTEL_COMPILER
// #define ZMQ_EXPORT __attribute__ ((visibility ("default")))
// #else
// #define ZMQ_EXPORT
// #endif
// #endif
// #endif

/*  Define integer types needed for event interface                          */
// #define ZMQ_DEFINED_STDINT 1
// #if defined ZMQ_HAVE_SOLARIS || defined ZMQ_HAVE_OPENVMS
// #include <inttypes.h>
// #elif defined _MSC_VER && _MSC_VER < 1600
// #ifndef uint64_t
// typedef unsigned __int64 uint64_t;
// #endif
// #ifndef int32_t
// typedef __int32 int32_t;
// #endif
// #ifndef uint32_t
// typedef unsigned __int32 uint32_t;
// #endif
// #ifndef uint16_t
// typedef unsigned __int16 uint16_t;
// #endif
// #ifndef uint8_t
// typedef unsigned __int8 uint8_t;
// #endif
// #else
// #include <stdint.h>
// #endif

// #if !defined _WIN32
// needed for sigset_t definition in zmq_ppoll
// #include <signal.h>
// #endif

//  32-bit AIX's pollfd struct members are called reqevents and rtnevents so it
//  defines compatibility macros for them. Need to include that header first to
//  stop build failures since zmq_pollset_t defines them as events and revents.
// #ifdef ZMQ_HAVE_AIX
// #include <poll.h>
// #endif


/******************************************************************************/
/*  0MQ errors.                                                               */
/******************************************************************************/

/*  A number random enough not to collide with different errno ranges on      */
/*  different OSes. The assumption is that error_t is at least 32-bit type.   */
// #define ZMQ_HAUSNUMERO 156384712

/*  On Windows platform some of the standard POSIX errnos are not defined.    */
// #ifndef ENOTSUP
// #define ENOTSUP (ZMQ_HAUSNUMERO + 1)
// #endif
// #ifndef EPROTONOSUPPORT
// #define EPROTONOSUPPORT (ZMQ_HAUSNUMERO + 2)
// #endif
// #ifndef ENOBUFS
// #define ENOBUFS (ZMQ_HAUSNUMERO + 3)
// #endif
// #ifndef ENETDOWN
// #define ENETDOWN (ZMQ_HAUSNUMERO + 4)
// #endif
// #ifndef EADDRINUSE
// #define EADDRINUSE (ZMQ_HAUSNUMERO + 5)
// #endif
// #ifndef EADDRNOTAVAIL
// #define EADDRNOTAVAIL (ZMQ_HAUSNUMERO + 6)
// #endif
// #ifndef ECONNREFUSED
// #define ECONNREFUSED (ZMQ_HAUSNUMERO + 7)
// #endif
// #ifndef EINPROGRESS
// #define EINPROGRESS (ZMQ_HAUSNUMERO + 8)
// #endif
// #ifndef ENOTSOCK
// #define ENOTSOCK (ZMQ_HAUSNUMERO + 9)
// #endif
// #ifndef EMSGSIZE
// #define EMSGSIZE (ZMQ_HAUSNUMERO + 10)
// #endif
// #ifndef EAFNOSUPPORT
// #define EAFNOSUPPORT (ZMQ_HAUSNUMERO + 11)
// #endif
// #ifndef ENETUNREACH
// #define ENETUNREACH (ZMQ_HAUSNUMERO + 12)
// #endif
// #ifndef ECONNABORTED
// #define ECONNABORTED (ZMQ_HAUSNUMERO + 13)
// #endif
// #ifndef ECONNRESET
// #define ECONNRESET (ZMQ_HAUSNUMERO + 14)
// #endif
// #ifndef ENOTCONN
// #define ENOTCONN (ZMQ_HAUSNUMERO + 15)
// #endif
// #ifndef ETIMEDOUT
// #define ETIMEDOUT (ZMQ_HAUSNUMERO + 16)
// #endif
// #ifndef EHOSTUNREACH
// #define EHOSTUNREACH (ZMQ_HAUSNUMERO + 17)
// #endif
// #ifndef ENETRESET
// #define ENETRESET (ZMQ_HAUSNUMERO + 18)
// #endif

/*  Native 0MQ error codes.                                                   */
// #define EFSM (ZMQ_HAUSNUMERO + 51)
// #define ENOCOMPATPROTO (ZMQ_HAUSNUMERO + 52)
// #define ETERM (ZMQ_HAUSNUMERO + 53)
// #define EMTHREAD (ZMQ_HAUSNUMERO + 54)

/*  This function retrieves the errno as it is known to 0MQ library. The goal */
/*  of this function is to make the code 100% portable, including where 0MQ   */
/*  compiled with certain CRT library (on Windows) is linked to an            */
/*  application that uses different CRT library.                              */
// ZMQ_EXPORT int zmq_errno (void);

/*  Resolves system errors and 0MQ errors to human-readable string.           */
// ZMQ_EXPORT const char *zmq_strerror (errnum_: i32);

/*  Run-time API version detection                                            */
// ZMQ_EXPORT void zmq_version (major_: *mut i32, minor_: *mut i32, patch_: *mut i32);

/******************************************************************************/
/*  0MQ infrastructure (a.k.a. context) initialisation & termination.         */
/******************************************************************************/

/*  Context options                                                           */
// #define ZMQ_IO_THREADS 1
pub const ZMQ_IO_THREADS: u8 = 1;
// #define ZMQ_MAX_SOCKETS 2
pub const ZMQ_MAX_SOCKETS: u8 = 2;
// #define ZMQ_SOCKET_LIMIT 3
pub const ZMQ_SOCKET_LIMIT: u8 = 3;
// #define ZMQ_THREAD_PRIORITY 3
pub const ZMQ_THREAD_PRIORITY: u8 = 3;
// #define ZMQ_THREAD_SCHED_POLICY 4
pub const ZMQ_THREAD_SCHED_POLICY: u8 = 4;
// #define ZMQ_MAX_MSGSZ 5
pub const ZMQ_MAX_MSGSZ: u8 = 5;
// #define ZMQ_MSG_T_SIZE 6
pub const ZMQ_MSG_T_SIZE: u8 = 6;
// #define ZMQ_THREAD_AFFINITY_CPU_ADD 7
pub const ZMQ_THREAD_AFFINITY_CPU_ADD: u8 = 7;
// #define ZMQ_THREAD_AFFINITY_CPU_REMOVE 8
pub const ZMQ_THREAD_AFFINITY_CPU_REMOVE: u8 = 8;
// #define ZMQ_THREAD_NAME_PREFIX 9
pub const ZMQ_THREAD_NAME_PREFIX: u8 = 9;

/*  Default for new contexts                                                  */
// #define ZMQ_IO_THREADS_DFLT 1
// #define ZMQ_MAX_SOCKETS_DFLT 1023
// #define ZMQ_THREAD_PRIORITY_DFLT -1
// #define ZMQ_THREAD_SCHED_POLICY_DFLT -1

// ZMQ_EXPORT void *zmq_ctx_new (void);
// ZMQ_EXPORT int zmq_ctx_term (context_: *mut c_void);
// ZMQ_EXPORT int zmq_ctx_shutdown (context_: *mut c_void);
// ZMQ_EXPORT int zmq_ctx_set (void *context_, int option_, optval_: i32);
// ZMQ_EXPORT int zmq_ctx_get (void *context_, option_: i32);

/*  Old (legacy) API                                                          */
// ZMQ_EXPORT void *zmq_init (io_threads_: i32);
// ZMQ_EXPORT int zmq_term (context_: *mut c_void);
// ZMQ_EXPORT int zmq_ctx_destroy (context_: *mut c_void);


/******************************************************************************/
/*  0MQ message definition.                                                   */
/******************************************************************************/

/* Some architectures, like sparc64 and some variants of aarch64, enforce pointer
 * alignment and raise sigbus on violations. Make sure applications allocate
 * zmq_msg_t on addresses aligned on a pointer-size boundary to avoid this issue.
 */
// typedef struct zmq_msg_t
pub struct zmq_msg_t {
    // #if defined(_MSC_VER) && (defined(_M_X64) || defined(_M_ARM64))
//     __declspec(align (8)) unsigned char _[64];
// #elif defined(_MSC_VER)                                                        \
//   && (defined(_M_IX86) || defined(_M_ARM_ARMV7VE) || defined(_M_ARM))
//     __declspec(align (4)) unsigned char _[64];
// #elif defined(__GNUC__) || defined(__INTEL_COMPILER)                           \
//   || (defined(__SUNPRO_C) && __SUNPRO_C >= 0x590)                              \
//   || (defined(__SUNPRO_CC) && __SUNPRO_CC >= 0x590)
//     unsigned char _[64] __attribute__ ((aligned (sizeof (void *))));
// #else
//     unsigned char _[64];
// #endif
    pub _x: [u8; 64],
}
// zmq_msg_t;

// typedef void (zmq_free_fn) (void *data_, hint_: *mut c_void);
pub type zmq_free_fn = fn(data_: *mut c_void, hint_: *mut c_void);


/******************************************************************************/
/*  0MQ socket definition.                                                    */
/******************************************************************************/

/*  Socket types.                                                             */
// #define ZMQ_PAIR 0
pub const ZMQ_PAIR: u8 = 0;
// #define ZMQ_PUB 1
pub const ZMQ_PUB: u8 = 1;
// #define ZMQ_SUB 2
pub const ZMQ_SUB: u8 = 2;
// #define ZMQ_REQ 3
pub const ZMQ_REQ: u8 = 3;
// #define ZMQ_REP 4
pub const ZMQ_REP: u8 = 4;
// #define ZMQ_DEALER 5
pub const ZMQ_DEALER: u8 = 5;
// #define ZMQ_ROUTER 6
pub const ZMQ_ROUTER: u8 = 6;
// #define ZMQ_PULL 7
pub const ZMQ_PULL: u8 = 7;
// #define ZMQ_PUSH 8
pub const ZMQ_PUSH: u8 = 8;
// #define ZMQ_XPUB 9
pub const ZMQ_XPUB: u8 = 9;
// #define ZMQ_XSUB 10
pub const ZMQ_XSUB: u8 = 10;
// #define ZMQ_STREAM 11
pub const ZMQ_STREAM: u8 = 11;

/*  Deprecated aliases                                                        */
// #define ZMQ_XREQ ZMQ_DEALER
pub const ZMQ_XREQ: u8 = ZMQ_DEALER;
// #define ZMQ_XREP ZMQ_ROUTER
pub const ZMQ_XREP: u8 = ZMQ_ROUTER;

/*  Socket options.                                                           */
// #define ZMQ_AFFINITY 4
pub const ZMQ_AFFINITY: u8 = 4;
// #define ZMQ_ROUTING_ID 5
pub const ZMQ_ROUTING_ID: u8 = 5;
// #define ZMQ_SUBSCRIBE 6
pub const ZQM_SUBSCRIBE: u8 = 6;
// #define ZMQ_UNSUBSCRIBE 7
pub const ZMQ_UNSUBSCRIBE: u8 = 7;
// #define ZMQ_RATE 8
pub const ZMQ_RATE: u8 = 8;
// #define ZMQ_RECOVERY_IVL 9
pub const ZMQ_RECOVERY_IVL: u8 = 9;
// #define ZMQ_SNDBUF 11
pub const ZMQ_SNDBUF: u8 = 11;
// #define ZMQ_RCVBUF 12
pub const ZMQ_RCVBUF: u8 = 12;
// #define ZMQ_RCVMORE 13
pub const ZMQ_RCVMORE: u8 = 13;
// #define ZMQ_FD 14
pub const ZMQ_FD: u8 = 14;
// #define ZMQ_EVENTS 15
pub const ZMQ_EVENTS: u8 = 15;
// #define ZMQ_TYPE 16
pub const ZMQ_TYPE: u8 = 16;
// #define ZMQ_LINGER 17
pub const ZMQ_LINGER: u8 = 17;
// #define ZMQ_RECONNECT_IVL 18
pub const ZMQ_RECONNECT_IVL: u8 = 18;
// #define ZMQ_BACKLOG 19
pub const ZMQ_BACKLOG: u8 = 19;
// #define ZMQ_RECONNECT_IVL_MAX 21
pub const ZMQ_RECONNECT_IVL_MAX: u8 = 21;
// #define ZMQ_MAXMSGSIZE 22
pub const ZMQ_MAXMSGZIE: u8 = 22;
// #define ZMQ_SNDHWM 23
pub const ZMQ_SNDHWM: u8 = 23;
// #define ZMQ_RCVHWM 24
pub const ZMQ_RCVHWM: u8 = 24;
pub const ZMQ_MULTICAST_HOPS: u8 = 25;
pub const ZMQ_RCVTIMEO: u8 = 27;
pub const ZMQ_SNDTIMEO: u8 = 28;
pub const ZMQ_LAST_ENDPOINT: u8 = 32;
pub const ZMQ_ROUTER_MANDATORY: u8 = 33;
pub const ZMQ_TCP_KEEPALIVE: u8 = 34;
pub const ZMQ_TCP_KEEPALIVE_CNT: u8 = 35;
pub const ZMQ_TCP_KEEPALIVE_IDLE: u8 = 36;
pub const ZMQ_TCP_KEEPALIVE_INTVL: u8 = 37;
pub const ZMQ_IMMEDIATE: u8 = 39;
pub const ZMQ_XPUB_VERBOSE: u8 = 40;
pub const ZMQ_ROUTER_RAW: u8 = 41;
pub const ZMQ_IPV6: u8 = 42;
pub const ZMQ_MECHANISM: u8 = 43;
pub const ZMQ_PLAIN_SERVER: u8 = 44;
pub const ZMQ_PLAIN_USERNAME: u8 = 45;
pub const ZMQ_PLAIN_PASSWORD: u8 = 46;
pub const ZMQ_CURVE_SERVER: u8 = 47;
pub const ZMQ_CURVE_PUBLICKEY: u8 = 48;
pub const ZMQ_CURVE_SECRETKEY: u8 = 49;
pub const ZMQ_CURVE_SERVERKEY: u8 = 50;
pub const ZMQ_PROBE_ROUTER: u8 = 51;
pub const ZMQ_REQ_CORRELATE: u8 = 52;
pub const ZMQ_REQ_RELAXED: u8 = 53;
pub const ZMQ_CONFLATE: u8 = 54;
pub const ZMQ_ZAP_DOMAIN: u8 = 55;
pub const ZMQ_ROUTER_HANDOVER: u8 = 56;
pub const ZMQ_TOS: u8 = 57;
pub const ZMQ_CONNECT_ROUTING_ID: u8 = 61;
pub const ZMQ_GSSAPI_SERVER: u8 = 62;
pub const ZMQ_GSSAPI_PRINCIPAL: u8 = 63;
pub const ZMQ_GSSAPI_SERVICE_PRINCIPAL: u8 = 64;
pub const ZMQ_GSSAPI_PLAINTEXT: u8 = 65;
pub const ZMQ_HANDSHAKE_IVL: u8 = 66;
pub const ZMQ_SOCKS_PROXY: u8 = 68;
pub const ZMQ_XPUB_NODROP: u8 = 69;
pub const ZMQ_BLOCKY: u8 = 70;
pub const ZMQ_XPUB_MANUAL: u8 = 71;
pub const ZMQ_XPUB_WELCOME_MSG: u8 = 72;
pub const ZMQ_STREAM_NOTIFY: u8 = 73;
pub const ZMQ_INVERT_MATCHING: u8 = 74;
pub const ZMQ_HEARTBEAT_IVL: u8 = 75;
pub const ZMQ_HEARTBEAT_TTL: u8 = 76;
pub const ZMQ_HEARTBEAT_TIMEOUT: u8 = 77;
pub const ZMQ_XPUB_VERBOSER: u8 = 78;
pub const ZMQ_CONNECT_TIMEOUT: u8 = 79;
pub const ZMQ_TCP_MAXRT: u8 = 80;
pub const ZMQ_THREAD_SAFE: u8 = 81;
pub const ZMQ_MULTICAST_MAXTPDU: u8 = 84;
pub const ZMQ_VMCI_BUFFER_SIZE: u8 = 85;
pub const ZMQ_VMCI_BUFFER_MIN_SIZE: u8 = 86;
pub const ZMQ_VMCI_BUFFER_MAX_SIZE: u8 = 87;
pub const ZMQ_VMCI_CONNECT_TIMEOUT: u8 = 88;
pub const ZMQ_USE_FD: u8 = 89;
pub const ZMQ_GSSAPI_PRINCIPAL_NAMETYPE: u8 = 90;
pub const ZMQ_GSSAPI_SERVICE_PRINCIPAL_NAMETYPE: u8 = 91;
pub const ZMQ_BINDTODEVICE: u8 = 92;

/*  Message options                                                           */
pub const ZMQ_MORE: u8 = 1;
pub const ZMQ_SHARED: u8 = 3;

/*  Send/recv options.                                                        */
pub const ZMQ_DONTWAIT: u8 = 1;
pub const ZMQ_SNDMORE: u8 = 2;

/*  Security mechanisms                                                       */
pub const ZMQ_NULL: u8 = 0;
pub const ZMQ_PLAIN: u8 = 1;
pub const ZMQ_CURVE: u8 = 2;
pub const ZMQ_GSSAPI: u8 = 3;

/*  RADIO-DISH protocol                                                       */
pub const ZMQ_GROUP_MAX_LENGTH: u8 = 255;

/*  Deprecated options and aliases                                            */
// #define ZMQ_IDENTITY ZMQ_ROUTING_ID
pub const ZMQ_IDENTITY: u8 = ZMQ_ROUTING_ID;
// #define ZMQ_CONNECT_RID ZMQ_CONNECT_ROUTING_ID
pub const ZMQ_CONNECT_RID: u8 = ZMQ_CONNECT_ROUTING_ID;
pub const ZMQ_TCP_ACCEPT_FILTER: u8 = 38;
pub const ZMQ_IPC_FILTER_PID: u8 = 58;
pub const ZMQ_IPC_FILTER_UID: u8 = 59;
pub const ZMQ_IPC_FILTER_GID: u8 = 60;
pub const ZMQ_IPV4ONLY: u8 = 31;
// #define ZMQ_DELAY_ATTACH_ON_CONNECT ZMQ_IMMEDIATE
pub const ZMQ_DELAY_ATTACH_ON_CONNECT: u8 = ZMQ_IMMEDIATE;
// #define ZMQ_NOBLOCK ZMQ_DONTWAIT
pub const ZMQ_NONBLOCK: u8 = ZMQ_DONTWAIT;
// #define ZMQ_FAIL_UNROUTABLE ZMQ_ROUTER_MANDATORY
pub const ZMQ_FAIL_UNROUTABLE: u8 = ZMQ_ROUTER_MANDATORY;
// #define ZMQ_ROUTER_BEHAVIOR ZMQ_ROUTER_MANDATORY
pub const ZMQ_ROUTER_BEHAVIOR: u8 = ZMQ_ROUTER_MANDATORY;

/*  Deprecated Message options                                                */
pub const ZMQ_SRCFD: u8 = 2;

/******************************************************************************/
/*  GSSAPI definitions                                                        */
/******************************************************************************/

/*  GSSAPI principal name types                                               */
pub const ZMQ_GSSAPI_NT_HOSTBASED: u8 = 0;
pub const ZMQ_GSSAPI_NT_USER_NAME: u8 = 1;
pub const ZMQ_GSSAPI_NT_KRB5_PRINCIPAL: u8 = 2;

/******************************************************************************/
/*  0MQ socket events and monitoring                                          */
/******************************************************************************/

/*  Socket transport events (TCP, IPC and TIPC only)                          */

// #define ZMQ_EVENT_CONNECTED 0x0001
pub const ZMQ_EVENT_CONNECTED: u16 = 0x0001;
// #define ZMQ_EVENT_CONNECT_DELAYED 0x0002
pub const ZMQ_EVENT_CONNECT_DELAYED: u16 = 0x0002;
// #define ZMQ_EVENT_CONNECT_RETRIED 0x0004
pub const ZMQ_EVENT_CONNECT_RETRIED: u16 = 0x0004;
// #define ZMQ_EVENT_LISTENING 0x0008
pub const ZMQ_EVENT_LISTENING: u16 = 0x0008;
// #define ZMQ_EVENT_BIND_FAILED 0x0010
pub const ZMQ_EVENT_BIND_FAILED: u16 = 0x0010;
// #define ZMQ_EVENT_ACCEPTED 0x0020
pub const ZMQ_EVENT_ACCEPTED: u16 = 0x0020;
// #define ZMQ_EVENT_ACCEPT_FAILED 0x0040
pub const ZMQ_EVENT_ACCEPT_FAILED: u16 = 0x0040;
// #define ZMQ_EVENT_CLOSED 0x0080
pub const ZMQ_EVENT_CLOSED: u16 = 0x0080;
// #define ZMQ_EVENT_CLOSE_FAILED 0x0100
pub const ZMQ_EVENT_CLOSE_FAILED: u16 = 0x0100;
// #define ZMQ_EVENT_DISCONNECTED 0x0200
pub const ZMQ_EVENT_DISCONNECTED: u16 = 0x0200;
// #define ZMQ_EVENT_MONITOR_STOPPED 0x0400
pub const ZMQ_EVENT_MONITOR_STOPPED: u16 = 0x0400;
// #define ZMQ_EVENT_ALL 0xFFFF
pub const ZMQ_EVENT_ALL: u16 = 0xffff;
/*  Unspecified system errors during handshake. Event value is an errno.      */
// #define ZMQ_EVENT_HANDSHAKE_FAILED_NO_DETAIL 0x0800
pub const ZMQ_EVENT_HANDSHAKE_FAILED_NO_DETAIL: u16 = 0x0800;
/*  Handshake complete successfully with successful authentication (if        *
 *  enabled). Event value is unused.                                          */
// #define ZMQ_EVENT_HANDSHAKE_SUCCEEDED 0x1000
pub const ZMQ_EVENT_HANDSHAKE_SUCCEEDED: u16 = 0x1000;
/*  Protocol errors between ZMTP peers or between server and ZAP handler.     *
 *  Event value is one of ZMQ_PROTOCOL_ERROR_*                                */
// #define ZMQ_EVENT_HANDSHAKE_FAILED_PROTOCOL 0x2000
pub const ZMQ_EVENT_HANDSHAKE_FAILED_PROTOCOL: u16 = 0x2000;
/*  Failed authentication requests. Event value is the numeric ZAP status     *
 *  code, i.e. 300, 400 or 500.                                               */
// #define ZMQ_EVENT_HANDSHAKE_FAILED_AUTH 0x4000
pub const ZMQ_EVENT_HANDSHAKE_FAILED_AUTH: u16 = 0x4000;
// #define ZMQ_PROTOCOL_ERROR_ZMTP_UNSPECIFIED 0x10000000
pub const ZMQ_PROTOCOL_ERROR_ZMTP_UNSPECIFIED: u32 = 0x10000000;
// #define ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND 0x10000001
pub const ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND: u32 = 0x10000001;
// #define ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_SEQUENCE 0x10000002
pub const ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_SEQUENCE: u32 = 0x10000002;
// #define ZMQ_PROTOCOL_ERROR_ZMTP_KEY_EXCHANGE 0x10000003
pub const ZMQ_PROTOCOL_ERROR_ZMTP_KEY_EXCHANGE: u32 = 0x10000003;
// #define ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_UNSPECIFIED 0x10000011
pub const ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_UNSPECIFIED: u32 = 0x10000011;
// #define ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_MESSAGE 0x10000012
pub const ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_MESSAGE: u32 = 0x10000012;
// #define ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_HELLO 0x10000013
pub const ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_HELLO: u32 = 0x10000013;
// #define ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_INITIATE 0x10000014
pub const ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_INITIATE: u32 = 0x10000014;
// #define ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR 0x10000015
pub const ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR: u32 = 0x10000015;
// #define ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_READY 0x10000016
pub const ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_READY: u32 = 0x10000016;
// #define ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_WELCOME 0x10000017
pub const ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_WELCOME: u32 = 0x10000017;
// #define ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_METADATA 0x10000018
pub const ZMQ_PROTOCOL_ERROR_ZMTP_INVALID_METADATA: u32 = 0x10000018;
// the following two may be due to erroneous configuration of a peer
// #define ZMQ_PROTOCOL_ERROR_ZMTP_CRYPTOGRAPHIC 0x11000001
pub const ZMQ_PROTOCOL_ERROR_ZMTP_CRYPTOGRAPHIC: u32 = 0x11000001;
// #define ZMQ_PROTOCOL_ERROR_ZMTP_MECHANISM_MISMATCH 0x11000002
pub const ZMQ_PROTOCOL_ERROR_ZMTP_MECHANISM_MISMATCH: u32 = 0x11000002;
// #define ZMQ_PROTOCOL_ERROR_ZAP_UNSPECIFIED 0x20000000
pub const ZMQ_PROTOCOL_ERROR_ZAP_UNSPECIFIED: u32 = 0x20000000;
// #define ZMQ_PROTOCOL_ERROR_ZAP_MALFORMED_REPLY 0x20000001
pub const ZMQ_PROTOCOL_ERROR_ZAP_MALFORMED_REPLY: u32 = 0x20000001;
// #define ZMQ_PROTOCOL_ERROR_ZAP_BAD_REQUEST_ID 0x20000002
pub const ZMQ_PROTOCOL_ERROR_ZAP_BAD_REQUEST_ID: u32 = 0x20000002;
// #define ZMQ_PROTOCOL_ERROR_ZAP_BAD_VERSION 0x20000003
pub const ZMQ_PROTOCOL_ERROR_ZAP_BAD_VERSION: u32 = 0x20000003;
// #define ZMQ_PROTOCOL_ERROR_ZAP_INVALID_STATUS_CODE 0x20000004
pub const ZMQ_PROTOCOL_ERROR_ZAP_INVALID_STATUS_CODE: u32 = 0x20000004;
// #define ZMQ_PROTOCOL_ERROR_ZAP_INVALID_METADATA 0x20000005
pub const ZMQ_PROTOCOL_ERROR_ZAP_INVALID_METADATA: u32 = 0x20000005;
// #define ZMQ_PROTOCOL_ERROR_WS_UNSPECIFIED 0x30000000
pub const ZMQ_PROTOCOL_ERROR_WS_UNSPECIFIED: u32 = 0x30000000;

// ZMQ_EXPORT void *zmq_socket (void *, type_: i32);
// ZMQ_EXPORT int zmq_close (s_: *mut c_void);
// ZMQ_EXPORT int
// zmq_setsockopt (void *s_, int option_, const void *optval_, size_t optvallen_);
// ZMQ_EXPORT int
// zmq_getsockopt (void *s_, int option_, void *optval_, size_t *optvallen_);
// ZMQ_EXPORT int zmq_bind (void *s_, const char *addr_);
// ZMQ_EXPORT int zmq_connect (void *s_, const char *addr_);
// ZMQ_EXPORT int zmq_unbind (void *s_, const char *addr_);
// ZMQ_EXPORT int zmq_disconnect (void *s_, const char *addr_);
// ZMQ_EXPORT int zmq_send (void *s_, const void *buf_, size_t len_, flags_: i32);
// ZMQ_EXPORT int
// zmq_send_const (void *s_, const void *buf_, size_t len_, flags_: i32);
// ZMQ_EXPORT int zmq_recv (void *s_, void *buf_, size_t len_, flags_: i32);
// ZMQ_EXPORT int zmq_socket_monitor (void *s_, const char *addr_, events_: i32);

/******************************************************************************/
/*  Hide socket fd type; this was before zmq_poller_event_t typedef below     */
/******************************************************************************/

// #if defined _WIN32
// Windows uses a pointer-sized unsigned integer to store the socket fd.
// #if defined _WIN64
// typedef unsigned __int64 zmq_fd_t;
// #else
#[cfg(all(windows, target_pointer_width = "64"))]
type zmq_fd_t = u64;

// typedef unsigned int zmq_fd_t;
#[cfg(all(windows, target_pointer_width = "32"))]
type zmq_fd_t = u32;
// #endif
// #else
// typedef int zmq_fd_t;
#[cfg(not(windows))]
type zmq_fd_t = i32;
// #endif

/******************************************************************************/
/*  Deprecated I/O multiplexing. Prefer using zmq_poller API                  */
/******************************************************************************/

// #define ZMQ_POLLIN 1
pub const ZMQ_POLLIN: u8 = 1;
// #define ZMQ_POLLOUT 2
pub const ZMQ_POLLOUT: u8 = 2;
// #define ZMQ_POLLERR 4
pub const ZMQ_POLLERR: u8 = 4;
// #define ZMQ_POLLPRI 8
pub const ZMQ_POLLPRI: u8 = 8;

#[derive(Default, Debug, Clone)]
pub struct zmq_pollitem_t {
    // socket: *mut c_void;
    pub socket: *mut c_void,
    // zmq_fd_t fd;
    pub fd: zmq_fd_t,
    // short events;
    pub events: i16,
    // short revents;
    pub revents: i16,
}
// zmq_pollitem_t;

// #define ZMQ_POLLITEMS_DFLT 16
pub const ZMQ_POLLITEMS_DFLT: usize = 16;

// ZMQ_EXPORT int zmq_poll (zmq_pollitem_t *items_, int nitems_, long timeout_);

/******************************************************************************/
/*  Message proxying                                                          */
/******************************************************************************/

// ZMQ_EXPORT int zmq_proxy (void *frontend_, void *backend_, capture_: *mut c_void);
// ZMQ_EXPORT int zmq_proxy_steerable (void *frontend_,
//                                     void *backend_,
//                                     void *capture_,
//                                     control_: *mut c_void);

/******************************************************************************/
/*  Probe library capabilities                                                */
/******************************************************************************/

// #define ZMQ_HAS_CAPABILITIES 1
// ZMQ_EXPORT int zmq_has (const char *capability_);

/*  Deprecated aliases */
// #define ZMQ_STREAMER 1
pub const ZMQ_STREAMER: u8 = 1;
// #define ZMQ_FORWARDER 2
pub const ZMQ_FORWARDER: u8 = 2;
// #define ZMQ_QUEUE 3
pub const ZMQ_QUEUE: u8 = 3;

/*  Deprecated methods */
// ZMQ_EXPORT int zmq_device (int type_, void *frontend_, backend_: *mut c_void);
// ZMQ_EXPORT int zmq_sendmsg (void *s_, zmq_msg_t *msg_, flags_: i32);
// ZMQ_EXPORT int zmq_recvmsg (void *s_, zmq_msg_t *msg_, flags_: i32);
// struct iovec;
// ZMQ_EXPORT int
// zmq_sendiov (void *s_, struct iovec *iov_, size_t count_, flags_: i32);
// ZMQ_EXPORT int
// zmq_recviov (void *s_, struct iovec *iov_, size_t *count_, flags_: i32);

/******************************************************************************/
/*  Encryption functions                                                      */
/******************************************************************************/

/*  Encode data with Z85 encoding. Returns encoded data                       */
// ZMQ_EXPORT char *
// zmq_z85_encode (char *dest_, const uint8_t *data_, size_t size_);

/*  Decode data with Z85 encoding. Returns decoded data                       */
// ZMQ_EXPORT uint8_t *zmq_z85_decode (uint8_t *dest_, const char *string_);

/*  Generate z85-encoded public and private keypair with tweetnacl/libsodium. */
/*  Returns 0 on success.                                                     */
// ZMQ_EXPORT int zmq_curve_keypair (char *z85_public_key_, char *z85_secret_key_);

/*  Derive the z85-encoded public key from the z85-encoded secret key.        */
/*  Returns 0 on success.                                                     */
// ZMQ_EXPORT int zmq_curve_public (char *z85_public_key_,
//                                  const char *z85_secret_key_);

/******************************************************************************/
/*  Atomic utility methods                                                    */
/******************************************************************************/

// ZMQ_EXPORT void *zmq_atomic_counter_new (void);
// ZMQ_EXPORT void zmq_atomic_counter_set (void *counter_, value_: i32);
// ZMQ_EXPORT int zmq_atomic_counter_inc (counter_: *mut c_void);
// ZMQ_EXPORT int zmq_atomic_counter_dec (counter_: *mut c_void);
// ZMQ_EXPORT int zmq_atomic_counter_value (counter_: *mut c_void);
// ZMQ_EXPORT void zmq_atomic_counter_destroy (void **counter_p_);

/******************************************************************************/
/*  Scheduling timers                                                         */
/******************************************************************************/

// #define ZMQ_HAVE_TIMERS

// typedef void (zmq_timer_fn) (int timer_id, arg: *mut c_void);

// ZMQ_EXPORT void *zmq_timers_new (void);
// ZMQ_EXPORT int zmq_timers_destroy (void **timers_p);
// ZMQ_EXPORT int
// zmq_timers_add (void *timers, size_t interval, zmq_timer_fn handler, arg: *mut c_void);
// ZMQ_EXPORT int zmq_timers_cancel (void *timers, timer_id: i32);
// ZMQ_EXPORT int
// zmq_timers_set_interval (void *timers, int timer_id, size_t interval);
// ZMQ_EXPORT int zmq_timers_reset (void *timers, timer_id: i32);
// ZMQ_EXPORT long zmq_timers_timeout (timers: *mut c_void);
// ZMQ_EXPORT int zmq_timers_execute (timers: *mut c_void);


/******************************************************************************/
/*  These functions are not documented by man pages -- use at your own risk.  */
/*  If you need these to be part of the formal ZMQ API, then (a) write a man  */
/*  page, and (b) write a test case in tests.                                 */
/******************************************************************************/

/*  Helper functions are used by perf tests so that they don't have to care   */
/*  about minutiae of time-related functions on different OS platforms.       */

/*  Starts the stopwatch. Returns the handle to the watch.                    */
// ZMQ_EXPORT void *zmq_stopwatch_start (void);

/*  Returns the number of microseconds elapsed since the stopwatch was        */
/*  started, but does not stop or deallocate the stopwatch.                   */
// ZMQ_EXPORT unsigned long zmq_stopwatch_intermediate (watch_: *mut c_void);

/*  Stops the stopwatch. Returns the number of microseconds elapsed since     */
/*  the stopwatch was started, and deallocates that watch.                    */
// ZMQ_EXPORT unsigned long zmq_stopwatch_stop (watch_: *mut c_void);

/*  Sleeps for specified number of seconds.                                   */
// ZMQ_EXPORT void zmq_sleep (seconds_: i32);

// typedef void (zmq_thread_fn) (void *);
pub type zmq_thread_fn = fn(*mut c_void);

/* Start a thread. Returns a handle to the thread.                            */
// ZMQ_EXPORT void *zmq_threadstart (zmq_thread_fn *func_, arg_: *mut c_void);

/* Wait for thread to complete then free up resources.                        */
// ZMQ_EXPORT void zmq_threadclose (thread_: *mut c_void);


/******************************************************************************/
/*  These functions are DRAFT and disabled in stable releases, and subject to */
/*  change at ANY time until declared stable.                                 */
/******************************************************************************/

// #ifdef ZMQ_BUILD_DRAFT_API

/*  DRAFT Socket types.                                                       */
// #define ZMQ_SERVER 12
pub const ZMQ_SERVER: u8 = 12;
// #define ZMQ_CLIENT 13
pub const ZMQ_CLIENT: u8 = 13;
// #define ZMQ_RADIO 14
pub const ZMQ_RADIO: u8 = 14;
// #define ZMQ_DISH 15
pub const ZMQ_DISH: u8 = 15;
// #define ZMQ_GATHER 16
pub const ZMQ_GATHER: u8 = 16;
// #define ZMQ_SCATTER 17
pub const ZMQ_SCATTER: u8 = 17;
// #define ZMQ_DGRAM 18
pub const ZQM_DGRAM: u8 = 18;
// #define ZMQ_PEER 19
pub const ZMQ_PEER: u8 = 19;
// #define ZMQ_CHANNEL 20
pub const ZMQ_CHANNEL: u8 = 20;

/*  DRAFT Socket options.                                                     */
// #define ZMQ_ZAP_ENFORCE_DOMAIN 93
pub const ZMQ_ZAP_ENFORCE_DOMAIN: u8 = 93;
// #define ZMQ_LOOPBACK_FASTPATH 94
pub const ZMQ_LOOPBACK_FASTPATH: u8 = 94;
// #define ZMQ_METADATA 95
pub const ZMQ_METADATA: u8 = 95;
// #define ZMQ_MULTICAST_LOOP 96
pub const ZMQ_MULTICAST_LOOP: u8 = 96;
// #define ZMQ_ROUTER_NOTIFY 97
pub const ZMQ_ROUTER_NOTIFY: u8 = 97;
// #define ZMQ_XPUB_MANUAL_LAST_VALUE 98
pub const ZMQ_XPUB_MANUAL_LAST_VALUE: u8 = 98;
// #define ZMQ_SOCKS_USERNAME 99
pub const ZMQ_SOCKS_USSERNAME: u8 = 99;
// #define ZMQ_SOCKS_PASSWORD 100
pub const ZMQ_SOCKS_PASSWORD: u8 = 100;
// #define ZMQ_IN_BATCH_SIZE 101
pub const ZMQ_IN_BATCH_SIZE: u8 = 101;
// #define ZMQ_OUT_BATCH_SIZE 102
pub const ZMQ_OUT_BATCH_SIZE: u8 = 102;
// #define ZMQ_WSS_KEY_PEM 103
pub const ZMQ_WSS_KEY_PEM: u8 = 103;
// #define ZMQ_WSS_CERT_PEM 104
pub const ZMQ_WSS_CERT_PEM: u8 = 104;
// #define ZMQ_WSS_TRUST_PEM 105
pub const ZMQ_WSS_TRUST_PEM: u8 = 105;
// #define ZMQ_WSS_HOSTNAME 106
pub const ZMQ_WSS_HOSTNAME: u8 = 106;
// #define ZMQ_WSS_TRUST_SYSTEM 107
pub const ZMQ_WSS_TRUST_SYSTESM: u8 = 107;
// #define ZMQ_ONLY_FIRST_SUBSCRIBE 108
pub const ZMQ_ONLY_FIRST_SUBSCRIBE: u8 = 108;
// #define ZMQ_RECONNECT_STOP 109
pub const ZMQ_RECONNECT_STOP: u8 = 109;
// #define ZMQ_HELLO_MSG 110
pub const ZMQ_HELLO_MSG: u8 = 110;
// #define ZMQ_DISCONNECT_MSG 111
pub const ZMQ_DISCONNECT_MSG: u8 = 111;
// #define ZMQ_PRIORITY 112
pub const ZMQ_PRIORITY: u8 = 112;
// #define ZMQ_BUSY_POLL 113
pub const ZMQ_BUSY_POLL: u8 = 113;
// #define ZMQ_HICCUP_MSG 114
pub const ZMQ_HICCUP_MSG: u8 = 114;
// #define ZMQ_XSUB_VERBOSE_UNSUBSCRIBE 115
pub const ZMQ_XSUB_VERBOSE_UNSUBSCRIBE: u8 = 115;
// #define ZMQ_TOPICS_COUNT 116
pub const ZMQ_TOPICS_COUNT: u8 = 116;

/*  DRAFT ZMQ_RECONNECT_STOP options                                          */
// #define ZMQ_RECONNECT_STOP_CONN_REFUSED 0x1
pub const ZMQ_RECONNECT_STOP_CONN_REFUSED: u8 = 0x1;
// #define ZMQ_RECONNECT_STOP_HANDSHAKE_FAILED 0x2
pub const ZMQ_RECONNECT_STOP_HANDSHAKE_FAILED: u8 = 0x2;
// #define ZMQ_RECONNECT_STOP_AFTER_DISCONNECT 0x4
pub const ZMQ_RECONNECT_STOP_AFTER_DISCONNECT: u8 = 0x4;

/*  DRAFT Context options                                                     */
// #define ZMQ_ZERO_COPY_RECV 10

/*  DRAFT Context methods.                                                    */
// ZMQ_EXPORT int zmq_ctx_set_ext (void *context_,
//                                 int option_,
//                                 const void *optval_,
//                                 size_t optvallen_);
// ZMQ_EXPORT int zmq_ctx_get_ext (void *context_,
//                                 int option_,
//                                 void *optval_,
//                                 size_t *optvallen_);

/*  DRAFT Socket methods.                                                     */
// ZMQ_EXPORT int zmq_join (void *s, const char *group);
// ZMQ_EXPORT int zmq_leave (void *s, const char *group);
// ZMQ_EXPORT uint32_t zmq_connect_peer (void *s_, const char *addr_);

/*  DRAFT Msg methods.                                                        */
// ZMQ_EXPORT int zmq_msg_set_routing_id (zmq_msg_t *msg, uint32_t routing_id);
// ZMQ_EXPORT uint32_t zmq_msg_routing_id (zmq_msg_t *msg);
// ZMQ_EXPORT int zmq_msg_set_group (zmq_msg_t *msg, const char *group);
// ZMQ_EXPORT const char *zmq_msg_group (zmq_msg_t *msg);
// ZMQ_EXPORT int
// zmq_msg_init_buffer (zmq_msg_t *msg_, const void *buf_, size_t size_);

/*  DRAFT Msg property names.                                                 */
// #define ZMQ_MSG_PROPERTY_ROUTING_ID "Routing-Id"
// #define ZMQ_MSG_PROPERTY_SOCKET_TYPE "Socket-Type"
// #define ZMQ_MSG_PROPERTY_USER_ID "User-Id"
// #define ZMQ_MSG_PROPERTY_PEER_ADDRESS "Peer-Address"

/*  Router notify options                                                     */
// #define ZMQ_NOTIFY_CONNECT 1
// #define ZMQ_NOTIFY_DISCONNECT 2

/******************************************************************************/
/*  Poller polling on sockets,fd and thread-safe sockets                      */
/******************************************************************************/

// #define ZMQ_HAVE_POLLER

#[derive(Default, Debug, Clone)]
pub struct zmq_poller_event_t {
    // socket: *mut c_void;
    pub socket: *mut c_void,
    // zmq_fd_t fd;
    pub fd: zmq_fd_t,
    // user_data: *mut c_void;
    pub user_data: *mut c_void,
    // short events;
    pub events: i16,
}
//zmq_poller_event_t;

// ZMQ_EXPORT void *zmq_poller_new (void);
// ZMQ_EXPORT int zmq_poller_destroy (void **poller_p);
// ZMQ_EXPORT int zmq_poller_size (poller: *mut c_void);
// ZMQ_EXPORT int
// zmq_poller_add (void *poller, void *socket, void *user_data, short events);
// ZMQ_EXPORT int zmq_poller_modify (void *poller, void *socket, short events);
// ZMQ_EXPORT int zmq_poller_remove (void *poller, socket: *mut c_void);
// ZMQ_EXPORT int
// zmq_poller_wait (void *poller, zmq_poller_event_t *event, long timeout);
// ZMQ_EXPORT int zmq_poller_wait_all (void *poller,
//                                     zmq_poller_event_t *events,
//                                     int n_events,
//                                     long timeout);
// ZMQ_EXPORT int zmq_poller_fd (void *poller, zmq_fd_t *fd);
//
// ZMQ_EXPORT int
// zmq_poller_add_fd (void *poller, zmq_fd_t fd, void *user_data, short events);
// ZMQ_EXPORT int zmq_poller_modify_fd (void *poller, zmq_fd_t fd, short events);
// ZMQ_EXPORT int zmq_poller_remove_fd (void *poller, zmq_fd_t fd);
//
// ZMQ_EXPORT int zmq_socket_get_peer_state (void *socket,
//                                           const void *routing_id,
//                                           size_t routing_id_size);

/*  DRAFT Socket monitoring events                                            */
// #define ZMQ_EVENT_PIPES_STATS 0x10000

// #define ZMQ_CURRENT_EVENT_VERSION 1
// #define ZMQ_CURRENT_EVENT_VERSION_DRAFT 2

// #define ZMQ_EVENT_ALL_V1 ZMQ_EVENT_ALL
// #define ZMQ_EVENT_ALL_V2 ZMQ_EVENT_ALL_V1 | ZMQ_EVENT_PIPES_STATS

// ZMQ_EXPORT int zmq_socket_monitor_versioned (
//   void *s_, const char *addr_, uint64_t events_, int event_version_, type_: i32);
// ZMQ_EXPORT int zmq_socket_monitor_pipes_stats (s: *mut c_void);

// #if !defined _WIN32
// ZMQ_EXPORT int zmq_ppoll (zmq_pollitem_t *items_,
//                           int nitems_,
//                           long timeout_,
//                           const sigset_t *sigmask_);
// #else
// Windows has no sigset_t
// ZMQ_EXPORT int zmq_ppoll (zmq_pollitem_t *items_,
//                           int nitems_,
//                           long timeout_,
//                           const sigmask_: *mut c_void);
// #endif

// #endif // ZMQ_BUILD_DRAFT_API


// #undef ZMQ_EXPORT

// #ifdef __cplusplus
// }
// #endif

// #endif
