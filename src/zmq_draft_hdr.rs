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

// #ifndef __ZMQ_DRAFT_H_INCLUDED__
// #define __ZMQ_DRAFT_H_INCLUDED__

/******************************************************************************/
/*  These functions are DRAFT and disabled in stable releases, and subject to */
/*  change at ANY time until declared stable.                                 */
/******************************************************************************/

// #ifndef ZMQ_BUILD_DRAFT_API

/*  DRAFT Socket types.                                                       */
// #define ZMQ_SERVER 12
// #define ZMQ_CLIENT 13
// #define ZMQ_RADIO 14
// #define ZMQ_DISH 15
// #define ZMQ_GATHER 16
// #define ZMQ_SCATTER 17
// #define ZMQ_DGRAM 18
// #define ZMQ_PEER 19
// #define ZMQ_CHANNEL 20

/*  DRAFT Socket options.                                                     */
// #define ZMQ_ZAP_ENFORCE_DOMAIN 93
// #define ZMQ_LOOPBACK_FASTPATH 94
// #define ZMQ_METADATA 95
// #define ZMQ_MULTICAST_LOOP 96
// #define ZMQ_ROUTER_NOTIFY 97
// #define ZMQ_XPUB_MANUAL_LAST_VALUE 98
// #define ZMQ_SOCKS_USERNAME 99
// #define ZMQ_SOCKS_PASSWORD 100
// #define ZMQ_IN_BATCH_SIZE 101
// #define ZMQ_OUT_BATCH_SIZE 102
// #define ZMQ_WSS_KEY_PEM 103
// #define ZMQ_WSS_CERT_PEM 104
// #define ZMQ_WSS_TRUST_PEM 105
// #define ZMQ_WSS_HOSTNAME 106
// #define ZMQ_WSS_TRUST_SYSTEM 107
// #define ZMQ_ONLY_FIRST_SUBSCRIBE 108
// #define ZMQ_RECONNECT_STOP 109
// #define ZMQ_HELLO_MSG 110
// #define ZMQ_DISCONNECT_MSG 111
// #define ZMQ_PRIORITY 112
// #define ZMQ_BUSY_POLL 113
// #define ZMQ_HICCUP_MSG 114
// #define ZMQ_XSUB_VERBOSE_UNSUBSCRIBE 115
// #define ZMQ_TOPICS_COUNT 116

/*  DRAFT ZMQ_RECONNECT_STOP options                                          */
// #define ZMQ_RECONNECT_STOP_CONN_REFUSED 0x1
// #define ZMQ_RECONNECT_STOP_HANDSHAKE_FAILED 0x2
// #define ZMQ_RECONNECT_STOP_AFTER_DISCONNECT 0x4

/*  DRAFT Context options                                                     */
// #define ZMQ_ZERO_COPY_RECV 10

/*  DRAFT Context methods.                                                    */
int zmq_ctx_set_ext (context_: *mut c_void,
                     option_: i32,
                     const optval_: *mut c_void,
                     optvallen_: usize);
int zmq_ctx_get_ext (context_: *mut c_void,
                     option_: i32,
                     optval_: *mut c_void,
                     optvallen_: *mut usize);

/*  DRAFT Socket methods.                                                     */
int zmq_join (s_: *mut c_void, group_: *const c_char);
int zmq_leave (s_: *mut c_void, group_: *const c_char);

/*  DRAFT Msg methods.                                                        */
int zmq_msg_set_routing_id (msg: *mut zmq_ZmqMessage, uint32_t routing_id_);
uint32_t zmq_msg_routing_id (zmq_ZmqMessage *msg);
int zmq_msg_set_group (msg: *mut zmq_ZmqMessage, group_: *const c_char);
const char *zmq_msg_group (zmq_ZmqMessage *msg);
int zmq_msg_init_buffer (msg: *mut zmq_ZmqMessage, const buf: *mut c_void, size: usize);

/*  DRAFT Msg property names.                                                 */
pub const ZMQ_MSG_PROPERTY_ROUTING_ID: &'static str = "Routing-Id";
// #define ZMQ_MSG_PROPERTY_SOCKET_TYPE "Socket-Type"
pub const ZMQ_MSG_PROPERTY_SOCKET_TYPE: &'static str = "Socket-Type";
// #define ZMQ_MSG_PROPERTY_USER_ID "User-Id"
pub const ZMQ_MSG_PROPERTY_USER_ID: &'static str = "User-Id";
// #define ZMQ_MSG_PROPERTY_PEER_ADDRESS "Peer-Address"
pub const ZMQ_MSG_PROPERTY_PEER_ADDRESS: &'static str = "Peer-Address";

/*  Router notify options                                                     */
// #define ZMQ_NOTIFY_CONNECT 1
// #define ZMQ_NOTIFY_DISCONNECT 2

/******************************************************************************/
/*  Poller polling on sockets,fd and thread-safe sockets                      */
/******************************************************************************/

// #if defined _WIN32
// typedef SOCKET zmq_fd_t;
// #else
// typedef int zmq_fd_t;
// #endif

// void *zmq_poller_new (void);
// int zmq_poller_destroy (void **poller_p_);
// int zmq_poller_size (poller_: *mut c_void);
// int zmq_poller_add (poller_: *mut c_void,
//                     socket_: *mut c_void,
//                     user_data_: *mut c_void,
//                     short events_);
// int zmq_poller_modify (poller_: *mut c_void, socket_: *mut c_void, short events_);
// int zmq_poller_remove (poller_: *mut c_void, socket_: *mut c_void);
// int zmq_poller_wait (poller_: *mut c_void, ZmqPollerEvent *event_, long timeout_);
// int zmq_poller_wait_all (poller_: *mut c_void,
//                          ZmqPollerEvent *events_,
//                          n_events_: i32,
//                          long timeout_);
// zmq_fd_t zmq_poller_fd (poller_: *mut c_void);
//
// int zmq_poller_add_fd (poller_: *mut c_void,
//                        zmq_fd_t fd_,
//                        user_data_: *mut c_void,
//                        short events_);
// int zmq_poller_modify_fd (poller_: *mut c_void, zmq_fd_t fd_, short events_);
// int zmq_poller_remove_fd (poller_: *mut c_void, zmq_fd_t fd_);
//
// int zmq_socket_get_peer_state (socket_: *mut c_void,
//                                const routing_id_: *mut c_void,
//                                routing_id_size_: usize);

/*  DRAFT Socket monitoring events                                            */
// #define ZMQ_EVENT_PIPES_STATS 0x10000

// #define ZMQ_CURRENT_EVENT_VERSION 1
// #define ZMQ_CURRENT_EVENT_VERSION_DRAFT 2

// #define ZMQ_EVENT_ALL_V1 ZMQ_EVENT_ALL
// #define ZMQ_EVENT_ALL_V2 ZMQ_EVENT_ALL_V1 | ZMQ_EVENT_PIPES_STATS

// int zmq_socket_monitor_versioned (
//   s_: *mut c_void, addr_: *const c_char, events_: u64, event_version_: i32, type_: i32);
// int zmq_socket_monitor_pipes_stats (s_: *mut c_void);

// #if !defined _WIN32
// int zmq_ppoll (zmq_pollitem_t *items_,
//                nitems_: i32,
//                long timeout_,
//                const sigset_t *sigmask_);
// #else
// Windows has no sigset_t
// int zmq_ppoll (zmq_pollitem_t *items_,
//                nitems_: i32,
//                long timeout_,
//                const sigmask_: *mut c_void);
// #endif

// #endif // ZMQ_BUILD_DRAFT_API

// #endif //ifndef __ZMQ_DRAFT_H_INCLUDED__
