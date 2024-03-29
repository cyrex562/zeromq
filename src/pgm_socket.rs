// /*
//     Copyright (c) 2007-2016 Contributors as noted in the AUTHORS file

//     This file is part of libzmq, the ZeroMQ core engine in C+= 1.

//     libzmq is free software; you can redistribute it and/or modify it under
//     the terms of the GNU Lesser General Public License (LGPL) as published
//     by the Free Software Foundation; either version 3 of the License, or
//     (at your option) any later version.

//     As a special exception, the Contributors give you permission to link
//     this library with independent modules to produce an executable,
//     regardless of the license terms of these independent modules, and to
//     copy and distribute the resulting executable under terms of your choice,
//     provided that you also meet, for each linked independent module, the
//     terms and conditions of the license of that module. An independent
//     module is a module which is not derived from or based on this library.
//     If you modify this library, you must extend this exception to your
//     version of the library.

//     libzmq is distributed in the hope that it will be useful, but WITHOUT
//     ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
//     FITNESS FOR A PARTICULAR PURPOSE. See the GNU Lesser General Public
//     License for more details.

//     You should have received a copy of the GNU Lesser General Public License
//     along with this program.  If not, see <http://www.gnu.org/licenses/>.
// */

// // #include "precompiled.hpp"

// // #ifdef ZMQ_HAVE_OPENPGM

// // #ifdef ZMQ_HAVE_LINUX
// // #include <poll.h>
// // #endif

// // #include <stdlib.h>
// // #include <string.h>
// // #include <string>

// // #include "options.hpp"
// // #include "pgm_socket.hpp"
// // #include "config.hpp"
// // #include "err.hpp"
// // #include "random.hpp"
// // #include "stdint.hpp"

// // #ifndef MSG_ERRQUEUE
// // #define MSG_ERRQUEUE 0x2000
// // #endif

// use crate::address_family::{AF_INET6, AF_UNSPEC};
// use crate::context::ZmqContext;
// use crate::defines::ZmqFileDesc;
// use bincode::options;
// use libc::{c_long, EBUSY, EINVAL, ENOMEM};
// use std::mem;
// use std::ptr::null_mut;
// use windows::Win32::Foundation::FALSE;
// use windows::Win32::Networking::WinSock::{
//     sa_family_t, IPPROTO_PGM, IPPROTO_UDP, SOCK_SEQPACKET, SOL_SOCKET, SO_RCVBUF, SO_SNDBUF,
// };

// use crate::utils::copy_bytes;

// #[derive(Default, Debug, Clone)]
// pub struct PgmSocket {
//     //  OpenPGM transport.
//     // pgm_sock_t *sock;
//     pub sock: pgm_sock_t,
//     pub last_rx_status: i32,
//     pub last_tx_status: i32,
//     //  Associated socket options.
//     // ZmqOptions options;
//     // pub options: &'a ZmqOptions,
//     //  true when pgm_socket should create receiving side.
//     pub receiver: bool,
//     //  Array of pgm_msgv_t structures to store received data
//     //  from the socket (pgm_transport_recvmsgv).
//     // pgm_msgv_t *pgm_msgv;
//     pub pgm_msgv: pgm_msgv_t,
//     //  Size of pgm_msgv array.
//     pub pgm_msgv_len: usize,
//     // How many bytes were read from pgm socket.
//     pub nbytes_rec: usize,
//     //  How many bytes were processed from last pgm socket read.
//     pub nbytes_processed: usize,
//     //  How many messages from pgm_msgv were already sent up.
//     pub pgm_msgv_processed: usize,
// }

// impl PgmSocket {
//     //  If receiver_ is true PGM transport is not generating SPM packets.
//     // PgmSocket (receiver_: bool, options: &ZmqOptions);
//     pub fn new(receiver_: bool, options: &ZmqContext) -> Self {
//         //     sock (null_mut()),
//         //     options (options_),
//         //     receiver (receiver_),
//         //     pgm_msgv (null_mut()),
//         //     pgm_msgv_len (0),
//         //     nbytes_rec (0),
//         //     nbytes_processed (0),
//         //     pgm_msgv_processed (0)
//         Self {
//             sock: (),
//             last_rx_status: 0,
//             last_tx_status: 0,
//             receiver: receiver_,
//             pgm_msgv: (),
//             pgm_msgv_len: 0,
//             nbytes_rec: 0,
//             nbytes_processed: 0,
//             pgm_msgv_processed: 0,
//         }
//     }

//     //  Closes the transport.
//     // ~PgmSocket ();

//     //  Initialize PGM network structures (GSI, GSRs).
//     // int init (udp_encapsulation_: bool, network_: &str);

//     //  Resolve PGM socket address.
//     //  network_ of the form <interface & multicast Group decls>:<IP port>
//     //  e.g. eth0;239.192.0.1:7500
//     //       link-local;224.250.0.1,224.250.0.2;224.250.0.3:8000
//     //       ;[fe80::1%en0]:7500
//     pub fn init_address(
//         &mut self,
//         network_: &str,
//         res: &mut &mut [pgm_addrinfo_t],
//         port_number: &mut u16,
//     ) -> i32 {
//         //  Parse port number, start from end for IPv6
//         // const char *port_delim = strrchr (network_, ':');
//         let port_delim = network_.find(':').expect("EINVAL");
//         // *port_number = atoi (port_delim + 1);
//         *port_number = u16::from_str_radix(&network_[port_delim + 1..], 10)
//             .expect("failed to convert string to port number");

//         let mut network = String::new();
//         // if (port_delim - network_ >=  mem::size_of::<network>() - 1) {
//         //   // errno = EINVAL;
//         //     return -1;
//         // }
//         // memset (network, 0, mem::size_of::<network>());
//         // memcpy (network, network_, port_delim - network_);
//         unsafe {
//             copy_bytes(
//                 network.as_bytes_mut(),
//                 0,
//                 network_.as_bytes(),
//                 0,
//                 port_delim as i32,
//             );
//         }

//         let pgm_error: pgm_error_t = pgm_error_t::default();
//         let hints: pgm_addrinfo_t = pgm_addrinfo_t::default();

//         // // memset (&hints, 0, mem::size_of::<hints>());
//         // hints.ai_family = AF_UNSPEC;
//         // if (!pgm_getaddrinfo (network, null_mut(), res, &pgm_error)) {
//         //     //  Invalid parameters don't set pgm_error_t.
//         //     // zmq_assert (pgm_error != null_mut());
//         //     if (pgm_error.domain == PGM_ERROR_DOMAIN_IF &&
//         //
//         //         //  NB: cannot catch EAI_BADFLAGS.
//         //         (pgm_error.code != PGM_ERROR_SERVICE
//         //             && pgm_error.code != PGM_ERROR_SOCKTNOSUPPORT)) {
//         //         //  User, host, or network configuration or transient error.
//         //         pgm_error_free (pgm_error);
//         //       // errno = EINVAL;
//         //         return -1;
//         //     }
//         //
//         //     //  Fatal OpenPGM internal error.
//         //     // zmq_assert (false);
//         // }
//         todo!();
//         return 0;
//     }

//     //  Create, Bind and connect PGM socket.
//     pub fn init(&mut self, udp_encapsulation_: bool, network_: &str) -> i32 {
//         //  Can not open transport before destroying old one.
//         // zmq_assert (sock == null_mut());
//         // zmq_assert (options.rate > 0);

//         //  Zero counter used in msgrecv.
//         nbytes_rec = 0;
//         nbytes_processed = 0;
//         pgm_msgv_processed = 0;
//         let mut port_number = 0u16;
//         let mut res = pgm_addrinfo_t::default();
//         let mut sa_family = sa_family_t::default();
//         // pgm_error_t *pgm_error = null_mut();
//         let mut pgm_error = pgm_error_t::default();

//         if (init_address(network_, &res, &port_number) < 0) {
//             // goto err_abort;
//         }

//         // zmq_assert (res != null_mut());

//         //  Pick up detected IP family.
//         sa_family = res.ai_send_addrs[0].gsr_group.ss_family;

//         //  Create IP/PGM or UDP/PGM socket.
//         if (udp_encapsulation_) {
//             if (!pgm_socket(&sock, sa_family, SOCK_SEQPACKET, IPPROTO_UDP, &pgm_error)) {
//                 //  Invalid parameters don't set pgm_error_t.
//                 // zmq_assert (pgm_error != null_mut());
//                 if (pgm_error.domain == PGM_ERROR_DOMAIN_SOCKET
//                     && (pgm_error.code != PGM_ERROR_BADF
//                         && pgm_error.code != PGM_ERROR_FAULT
//                         && pgm_error.code != PGM_ERROR_NOPROTOOPT
//                         && pgm_error.code != PGM_ERROR_FAILED))
//                 {}

//                 //  User, host, or network configuration or transient error.
//                 // todo
//                 // goto err_abort;

//                 //  Fatal OpenPGM internal error.
//                 // zmq_assert (false);
//             }

//             //  All options are of data type int
//             let encapsulation_port: i32 = port_number as i32;
//             if (!pgm_setsockopt(
//                 sock,
//                 IPPROTO_PGM,
//                 PGM_UDP_ENCAP_UCAST_PORT,
//                 &encapsulation_port,
//                 mem::size_of_val(&encapsulation_port),
//             )) {
//                 // goto err_abort;
//             }
//             if (!pgm_setsockopt(
//                 sock,
//                 IPPROTO_PGM,
//                 PGM_UDP_ENCAP_MCAST_PORT,
//                 &encapsulation_port,
//                 mem::size_of_val(&encapsulation_port),
//             )) {
//                 // goto err_abort;
//             }
//         } else {
//             if (!pgm_socket(&sock, sa_family, SOCK_SEQPACKET, IPPROTO_PGM, &pgm_error)) {
//                 //  Invalid parameters don't set pgm_error_t.
//                 // zmq_assert (pgm_error != null_mut());
//                 if (pgm_error.domain == PGM_ERROR_DOMAIN_SOCKET
//                     && (pgm_error.code != PGM_ERROR_BADF
//                         && pgm_error.code != PGM_ERROR_FAULT
//                         && pgm_error.code != PGM_ERROR_NOPROTOOPT
//                         && pgm_error.code != PGM_ERROR_FAILED))
//                 {}

//                 //  User, host, or network configuration or transient error.
//                 // goto err_abort;

//                 //  Fatal OpenPGM internal error.
//                 // zmq_assert (false);
//             }
//         }

//         {
//             let rcvbuf: i32 = self.options.rcvbuf;
//             if (rcvbuf >= 0) {
//                 if (!pgm_setsockopt(
//                     sock,
//                     SOL_SOCKET,
//                     SO_RCVBUF,
//                     &rcvbuf,
//                     4,
//                 )) {
//                     // goto
//                     // err_abort;
//                 }
//             }

//             let sndbuf: i32 = self.options.sndbuf;
//             if (sndbuf >= 0) {
//                 if (!pgm_setsockopt(
//                     sock,
//                     SOL_SOCKET,
//                     SO_SNDBUF,
//                     &sndbuf,
//                     4,
//                 )) {
//                     // goto
//                     // err_abort;
//                 }
//             }

//             let max_tpdu: i32 = self.options.multicast_maxtpdu;
//             if (!pgm_setsockopt(
//                 sock,
//                 IPPROTO_PGM,
//                 PGM_MTU,
//                 &max_tpdu,
//                 4,
//             )) {
//                 // goto
//                 // err_abort;
//             }
//         }

//         if (receiver) {
//             let recv_only: i32 = 1;
//             let mut rxw_max_tpdu = self.options.multicast_maxtpdu;
//             let mut rxw_sqns = compute_sqns(rxw_max_tpdu);
//             let mut peer_expiry = pgm_secs(300);
//             let mut spmr_expiry = pgm_msecs(25);
//             let mut nak_bo_ivl = pgm_msecs(50);
//             let mut nak_rpt_ivl = pgm_msecs(200);
//             let mut nak_rdata_ivl = pgm_msecs(200);
//             let mut nak_data_retries = 50;
//             nak_ncf_retries = 50;

//             if (!pgm_setsockopt(
//                 sock,
//                 IPPROTO_PGM,
//                 PGM_RECV_ONLY,
//                 &recv_only,
//                 4,
//             ) || !pgm_setsockopt(
//                 sock,
//                 IPPROTO_PGM,
//                 PGM_RXW_SQNS,
//                 &rxw_sqns,
//                 4,
//             ) || !pgm_setsockopt(
//                 sock,
//                 IPPROTO_PGM,
//                 PGM_PEER_EXPIRY,
//                 &peer_expiry,
//                 4,
//             ) || !pgm_setsockopt(
//                 sock,
//                 IPPROTO_PGM,
//                 PGM_SPMR_EXPIRY,
//                 &spmr_expiry,
//                 4,
//             ) || !pgm_setsockopt(
//                 sock,
//                 IPPROTO_PGM,
//                 PGM_NAK_BO_IVL,
//                 &nak_bo_ivl,
//                 4,
//             ) || !pgm_setsockopt(
//                 sock,
//                 IPPROTO_PGM,
//                 PGM_NAK_RPT_IVL,
//                 &nak_rpt_ivl,
//                 4,
//             ) || !pgm_setsockopt(
//                 sock,
//                 IPPROTO_PGM,
//                 PGM_NAK_RDATA_IVL,
//                 &nak_rdata_ivl,
//                 4,
//             ) || !pgm_setsockopt(
//                 sock,
//                 IPPROTO_PGM,
//                 PGM_NAK_DATA_RETRIES,
//                 &nak_data_retries,
//                 4,
//             ) || !pgm_setsockopt(
//                 sock,
//                 IPPROTO_PGM,
//                 PGM_NAK_NCF_RETRIES,
//                 &nak_ncf_retries,
//                 4,
//             )) {
//                 // goto
//                 // err_abort;
//             }
//         } else {
//             let send_only: i32 = 1;
//             let mut max_rte = ((self.options.rate * 1000) / 8);
//             let mut txw_max_tpdu = self.options.multicast_maxtpdu;
//             let mut txw_sqns = compute_sqns(txw_max_tpdu);
//             let mut ambient_spm = pgm_secs(30);
//             let mut heartbeat_spm: [(); 9] = [
//                 pgm_msecs(100),
//                 pgm_msecs(100),
//                 pgm_msecs(100),
//                 pgm_msecs(100),
//                 pgm_msecs(1300),
//                 pgm_secs(7),
//                 pgm_secs(16),
//                 pgm_secs(25),
//                 pgm_secs(30),
//             ];

//             if (!pgm_setsockopt(
//                 sock,
//                 IPPROTO_PGM,
//                 PGM_SEND_ONLY,
//                 &send_only,
//                 4,
//             ) || !pgm_setsockopt(
//                 sock,
//                 IPPROTO_PGM,
//                 PGM_ODATA_MAX_RTE,
//                 &max_rte,
//                 4,
//             ) || !pgm_setsockopt(
//                 sock,
//                 IPPROTO_PGM,
//                 PGM_TXW_SQNS,
//                 &txw_sqns,
//                 4,
//             ) || !pgm_setsockopt(
//                 sock,
//                 IPPROTO_PGM,
//                 PGM_AMBIENT_SPM,
//                 &ambient_spm,
//                 4,
//             ) || !pgm_setsockopt(
//                 sock,
//                 IPPROTO_PGM,
//                 PGM_HEARTBEAT_SPM,
//                 &heartbeat_spm,
//                 4,
//             )) {
//                 // goto
//                 // err_abort;
//             }
//         }

//         //  PGM transport GSI.
//         // struct pgm_sockaddr_t addr;
//         let mut addr = pgm_sockaddr_t::default();

//         // memset (&addr, 0, mem::size_of::<addr>());
//         addr.sa_port = port_number;
//         addr.sa_addr.sport = DEFAULT_DATA_SOURCE_PORT;

//         //  Create random GSI.
//         let mut buf = [0u32; 2];
//         buf[0] = generate_random();
//         buf[1] = generate_random();
//         if (!pgm_gsi_create_from_data(&addr.sa_addr.gsi, buf, 8)) {
//             // goto
//             // err_abort;
//         }

//         //  Bind a transport to the specified network devices.
//         // struct pgm_interface_req_t if_req;
//         let mut if_req = pgm_interface_req_t::default();
//         // memset (&if_req, 0, mem::size_of::<if_req>());
//         if_req.ir_interface = res.ai_recv_addrs[0].gsr_interface;
//         if_req.ir_scope_id = 0;
//         if AF_INET6 == sa_family as u16 {
//             // struct sockaddr_in6 sa6;
//             // memcpy (&sa6, &res.ai_recv_addrs[0].gsr_group, mem::size_of::<sa6>());
//             let mut sa6 = sockaddr_in6::default();
//             // if_req.ir_scope_id = sa6.sin6_scope_id;
//             // TODO
//         }
//         if (!pgm_bind3(
//             sock,
//             &addr,
//             mem::size_of::<addr>(),
//             &if_req,
//             mem::size_of::<if_req>(),
//             &if_req,
//             mem::size_of::<if_req>(),
//             &pgm_error,
//         )) {
//             //  Invalid parameters don't set pgm_error_t.
//             // zmq_assert (pgm_error != null_mut());
//             if ((pgm_error.domain == PGM_ERROR_DOMAIN_SOCKET
//                 || pgm_error.domain == PGM_ERROR_DOMAIN_IF)
//                 && (pgm_error.code != PGM_ERROR_INVAL
//                     && pgm_error.code != PGM_ERROR_BADF
//                     && pgm_error.code != PGM_ERROR_FAULT))
//             {}

//             //  User, host, or network configuration or transient error.
//             // goto err_abort;

//             //  Fatal OpenPGM internal error.
//             // zmq_assert (false);
//         }

//         //  Join IP multicast groups.
//         // for (unsigned i = 0; i < res.ai_recv_addrs_len; i+= 1)
//         for i in 0..res.ai_recv_addrs_len {
//             if (!pgm_setsockopt(
//                 sock,
//                 IPPROTO_PGM,
//                 PGM_JOIN_GROUP,
//                 &res.ai_recv_addrs[i],
//                 sizeof(group_req),
//             )) {
//                 // goto err_abort;
//             }
//         }
//         if (!pgm_setsockopt(
//             sock,
//             IPPROTO_PGM,
//             PGM_SEND_GROUP,
//             &res.ai_send_addrs[0],
//             sizeof(group_req),
//         )) {
//             // goto err_abort;
//         }

//         pgm_freeaddrinfo(res);
//         res = null_mut();

//         //  Set IP level parameters.
//         {
//             // Multicast loopback disabled by default
//             let multicast_loop: i32 = 0;
//             if (!pgm_setsockopt(
//                 sock,
//                 IPPROTO_PGM,
//                 PGM_MULTICAST_LOOP,
//                 &multicast_loop,
//                 mem::size_of::<multicast_loop>(),
//             )) {
//                 // goto
//                 // err_abort;
//             }

//             let multicast_hops: i32 = options.multicast_hops;
//             if (!pgm_setsockopt(
//                 sock,
//                 IPPROTO_PGM,
//                 PGM_MULTICAST_HOPS,
//                 &multicast_hops,
//                 mem::size_of::<multicast_hops>(),
//             )) {
//                 // goto
//                 // err_abort;
//             }

//             //  Expedited Forwarding PHB for network elements, no ECN.
//             //  Ignore return value due to varied runtime support.
//             let dscp: i32 = 0x2e << 2;
//             if (AF_INET6 != sa_family as u16) {
//                 pgm_setsockopt(sock, IPPROTO_PGM, PGM_TOS, &dscp, mem::size_of::<dscp>());
//             }

//             let nonblocking: i32 = 1;
//             if (!pgm_setsockopt(
//                 sock,
//                 IPPROTO_PGM,
//                 PGM_NOBLOCK,
//                 &nonblocking,
//                 mem::size_of::<nonblocking>(),
//             )) {
//                 // goto
//                 // err_abort;
//             }
//         }

//         //  Connect PGM transport to start state machine.
//         if (!pgm_connect(sock, &pgm_error)) {
//             //  Invalid parameters don't set pgm_error_t.
//             // zmq_assert (pgm_error != null_mut());
//             // goto err_abort;
//         }

//         //  For receiver transport preallocate pgm_msgv array.
//         if (receiver) {
//             // zmq_assert (options.in_batch_size > 0);
//             let max_tsdu_size = get_max_tsdu_size();
//             pgm_msgv_len = options.in_batch_size / max_tsdu_size;
//             if (options.in_batch_size % max_tsdu_size) {
//                 pgm_msgv_len += 1;
//             }
//             // zmq_assert (pgm_msgv_len);

//             // todo
//             // pgm_msgv = (pgm_msgv_t *) malloc (mem::size_of::<pgm_msgv_t>() * pgm_msgv_len);
//             // alloc_assert (pgm_msgv);
//         }

//         return 0;

//         // err_abort:
//         if (sock != null_mut()) {
//             pgm_close(sock, FALSE);
//             sock = null_mut();
//         }
//         if (res != null_mut()) {
//             pgm_freeaddrinfo(res);
//             res = null_mut();
//         }
//         if (pgm_error != null_mut()) {
//             pgm_error_free(pgm_error);
//             pgm_error = null_mut();
//         }
//       // errno = EINVAL;
//         return -1;
//     }

//     //   Get receiver fds and store them into user allocated memory.
//     // void get_receiver_fds (ZmqFileDesc *receive_fd_, ZmqFileDesc *waiting_pipe_fd_);
//     //  Get receiver fds. receive_fd_ is signaled for incoming packets,
//     //  waiting_pipe_fd_ is signaled for state driven events and data.
//     pub fn get_receiver_fds(&mut self, receive_fd: ZmqFileDesc, waiting_pipe_fd: ZmqFileDesc) {
//         // socklen_t socklen;
//         // rc: bool

//         // zmq_assert (receive_fd_);
//         // zmq_assert (waiting_pipe_fd_);

//         let mut socklen = sizeof(*receive_fd_);
//         let mut rc = pgm_getsockopt(sock, IPPROTO_PGM, PGM_RECV_SOCK, receive_fd_, &socklen);
//         // zmq_assert (rc);
//         // zmq_assert (socklen == sizeof (*receive_fd_));

//         socklen = sizeof(*waiting_pipe_fd_);
//         rc = pgm_getsockopt(
//             sock,
//             IPPROTO_PGM,
//             PGM_PENDING_SOCK,
//             waiting_pipe_fd_,
//             &socklen,
//         );
//         // zmq_assert (rc);
//         // zmq_assert (socklen == sizeof (*waiting_pipe_fd_));
//     }

//     //   Get sender and receiver fds and store it to user allocated
//     //   memory. Receive fd is used to process NAKs from peers.
//     // void get_sender_fds (ZmqFileDesc *send_fd_,
//     //                      ZmqFileDesc *receive_fd_,
//     //                      ZmqFileDesc *rdata_notify_fd_,
//     //                      ZmqFileDesc *pending_notify_fd_);

//     //  Get fds and store them into user allocated memory.
//     //  send_fd is for non-blocking send wire notifications.
//     //  receive_fd_ is for incoming back-channel protocol packets.
//     //  rdata_notify_fd_ is raised for waiting repair transmissions.
//     //  pending_notify_fd_ is for state driven events.
//     pub fn get_sender_fds(
//         &mut self,
//         send_fd_: &mut ZmqFileDesc,
//         receive_fd_: &mut ZmqFileDesc,
//         rdata_notify_fd_: &mut ZmqFileDesc,
//         pending_notify_fd_: &mut ZmqFileDesc,
//     ) {
//         // socklen_t socklen;
//         // rc: bool

//         // zmq_assert (send_fd_);
//         // zmq_assert (receive_fd_);
//         // zmq_assert (rdata_notify_fd_);
//         // zmq_assert (pending_notify_fd_);

//         socklen = sizeof(*send_fd_);
//         rc = pgm_getsockopt(sock, IPPROTO_PGM, PGM_SEND_SOCK, send_fd_, &socklen);
//         // zmq_assert (rc);
//         // zmq_assert (socklen == sizeof (*receive_fd_));

//         socklen = sizeof(*receive_fd_);
//         rc = pgm_getsockopt(sock, IPPROTO_PGM, PGM_RECV_SOCK, receive_fd_, &socklen);
//         // zmq_assert (rc);
//         // zmq_assert (socklen == sizeof (*receive_fd_));

//         socklen = sizeof(*rdata_notify_fd_);
//         rc = pgm_getsockopt(
//             sock,
//             IPPROTO_PGM,
//             PGM_REPAIR_SOCK,
//             rdata_notify_fd_,
//             &socklen,
//         );
//         // zmq_assert (rc);
//         // zmq_assert (socklen == sizeof (*rdata_notify_fd_));

//         socklen = sizeof(*pending_notify_fd_);
//         rc = pgm_getsockopt(
//             sock,
//             IPPROTO_PGM,
//             PGM_PENDING_SOCK,
//             pending_notify_fd_,
//             &socklen,
//         );
//         // zmq_assert (rc);
//         // zmq_assert (socklen == sizeof (*pending_notify_fd_));
//     }

//     //  Send data as one APDU, transmit window owned memory.
//     // size_t send (data: &mut [u8], data_len_: usize);
//     //  Send one APDU, transmit window owned memory.
//     //  data_len_ must be less than one TPDU.
//     pub fn send(&mut self, &mut data: &mut [u8], data_len_: usize) -> usize {
//         let mut nbytes = 0usize;
//         let status: i32 = pgm_send(sock, data, data_len_, &nbytes);

//         //  We have to write all data as one packet.
//         if (nbytes > 0) {
//             // zmq_assert (status == PGM_IO_STATUS_NORMAL);
//             // zmq_assert (nbytes == data_len_);
//         } else {
//             // zmq_assert (status == PGM_IO_STATUS_RATE_LIMITED
//             //             || status == PGM_IO_STATUS_WOULD_BLOCK);

//             if (status == PGM_IO_STATUS_RATE_LIMITED) {
//               // errno = ENOMEM;
//             } else {
//               // errno = EBUSY;
//             }
//         }

//         //  Save return value.
//         last_tx_status = status;

//         return nbytes;
//     }

//     //  Returns max tsdu size without fragmentation.
//     // size_t get_max_tsdu_size ();

//     //  Receive data from pgm socket.
//     // ssize_t receive (data: *mut *mut c_void const pgm_tsi_t **tsi_);

//     // long get_rx_timeout ();
//     pub fn get_rx_timeout(&mut self) -> c_long {
//         // if (last_rx_status != PGM_IO_STATUS_RATE_LIMITED
//         //     && last_rx_status != PGM_IO_STATUS_TIMER_PENDING) {
//         //     return -1;
//         // }
//         //
//         // let tv: timeval tv;
//         // socklen_t optlen = mem::size_of::<tv>();
//         // const bool rc = pgm_getsockopt (sock, IPPROTO_PGM,
//         //                                 last_rx_status == PGM_IO_STATUS_RATE_LIMITED
//         //                                   ? PGM_RATE_REMAIN
//         //                                   : PGM_TIME_REMAIN,
//         //                                 &tv, &optlen);
//         // // zmq_assert (rc);
//         //
//         // const long timeout = (tv.tv_sec * 1000) + (tv.tv_usec / 1000);
//         //
//         // return timeout;
//         todo!()
//     }

//     // long get_tx_timeout ();
//     pub fn get_tx_timeout(&mut self) -> c_long {
//         // if (last_tx_status != PGM_IO_STATUS_RATE_LIMITED)
//         //     return -1;
//         //
//         // struct timeval tv;
//         // socklen_t optlen = mem::size_of::<tv>();
//         // const bool rc =
//         //   pgm_getsockopt (sock, IPPROTO_PGM, PGM_RATE_REMAIN, &tv, &optlen);
//         // // zmq_assert (rc);
//         //
//         // const long timeout = (tv.tv_sec * 1000) + (tv.tv_usec / 1000);
//         //
//         // return timeout;
//         todo!()
//     }

//     //  POLLIN on sender side should mean NAK or SPMR receiving.
//     //  process_upstream function is used to handle such a situation.
//     // void process_upstream ();

//     //
//     //  Compute size of the buffer based on rate and recovery interval.
//     // int compute_sqns (tpdu_: i32);
//     pub fn get_max_tsdu_size(&mut self) {
//         // int max_tsdu = 0;
//         // socklen_t optlen = mem::size_of::<max_tsdu>();
//         //
//         // bool rc = pgm_getsockopt (sock, IPPROTO_PGM, PGM_MSS, &max_tsdu, &optlen);
//         // // zmq_assert (rc);
//         // // zmq_assert (optlen == mem::size_of::<max_tsdu>());
//         // return (size_t) max_tsdu;
//         todo!()
//     }

//     //  pgm_recvmsgv is called to fill the pgm_msgv array up to  pgm_msgv_len.
//     //  In subsequent calls data from pgm_msgv structure are returned.
//     pub fn receive(&mut self, raw_data_: &mut &mut [u8], tsi_: &mut &mut [pgm_tsi_t]) -> isize {
//         todo!();
//         // // size_t raw_data_len = 0;
//         //
//         // //  We just sent all data from pgm_transport_recvmsgv up
//         // //  and have to return 0 that another engine in this thread is scheduled.
//         // if (nbytes_rec == nbytes_processed && nbytes_rec > 0) {
//         //     //  Reset all the counters.
//         //     nbytes_rec = 0;
//         //     nbytes_processed = 0;
//         //     pgm_msgv_processed = 0;
//         //   // errno = EAGAIN;
//         //     return 0;
//         // }
//         //
//         // //  If we have are going first time or if we have processed all pgm_msgv_t
//         // //  structure previously read from the pgm socket.
//         // if (nbytes_rec == nbytes_processed) {
//         //     //  Check program flow.
//         //     // zmq_assert (pgm_msgv_processed == 0);
//         //     // zmq_assert (nbytes_processed == 0);
//         //     // zmq_assert (nbytes_rec == 0);
//         //
//         //     //  Receive a vector of Application Protocol Domain Unit's (APDUs)
//         //     //  from the transport.
//         //     pgm_error_t *pgm_error = null_mut();
//         //
//         //     let status: i32 = pgm_recvmsgv (sock, pgm_msgv, pgm_msgv_len,
//         //                                     MSG_ERRQUEUE, &nbytes_rec, &pgm_error);
//         //
//         //     //  Invalid parameters.
//         //     // zmq_assert (status != PGM_IO_STATUS_ERROR);
//         //
//         //     last_rx_status = status;
//         //
//         //     //  In a case when no ODATA/RDATA fired POLLIN event (SPM...)
//         //     //  pgm_recvmsg returns PGM_IO_STATUS_TIMER_PENDING.
//         //     if (status == PGM_IO_STATUS_TIMER_PENDING) {
//         //         // zmq_assert (nbytes_rec == 0);
//         //
//         //         //  In case if no RDATA/ODATA caused POLLIN 0 is
//         //         //  returned.
//         //         nbytes_rec = 0;
//         //       // errno = EBUSY;
//         //         return 0;
//         //     }
//         //
//         //     //  Send SPMR, NAK, ACK is rate limited.
//         //     if (status == PGM_IO_STATUS_RATE_LIMITED) {
//         //         // zmq_assert (nbytes_rec == 0);
//         //
//         //         //  In case if no RDATA/ODATA caused POLLIN 0 is returned.
//         //         nbytes_rec = 0;
//         //       // errno = ENOMEM;
//         //         return 0;
//         //     }
//         //
//         //     //  No peers and hence no incoming packets.
//         //     if (status == PGM_IO_STATUS_WOULD_BLOCK) {
//         //         // zmq_assert (nbytes_rec == 0);
//         //
//         //         //  In case if no RDATA/ODATA caused POLLIN 0 is returned.
//         //         nbytes_rec = 0;
//         //       // errno = EAGAIN;
//         //         return 0;
//         //     }
//         //
//         //     //  Data loss.
//         //     if (status == PGM_IO_STATUS_RESET) {
//         //         struct pgm_sk_buff_t *skb = pgm_msgv[0].msgv_skb[0];
//         //
//         //         //  Save lost data TSI.
//         //         *tsi_ = &skb.tsi;
//         //         nbytes_rec = 0;
//         //
//         //         //  In case of dala loss -1 is returned.
//         //       // errno = EINVAL;
//         //         pgm_free_skb (skb);
//         //         return -1;
//         //     }
//         //
//         //     // zmq_assert (status == PGM_IO_STATUS_NORMAL);
//         // } else {
//         //     // zmq_assert (pgm_msgv_processed <= pgm_msgv_len);
//         // }
//         //
//         // // Zero byte payloads are valid in PGM, but not 0MQ protocol.
//         // // zmq_assert (nbytes_rec > 0);
//         //
//         // // Only one APDU per pgm_msgv_t structure is allowed.
//         // // zmq_assert (pgm_msgv[pgm_msgv_processed].msgv_len == 1);
//         //
//         // struct pgm_sk_buff_t *skb = pgm_msgv[pgm_msgv_processed].msgv_skb[0];
//         //
//         // //  Take pointers from pgm_msgv_t structure.
//         // *raw_data_ = skb.data;
//         // raw_data_len = skb.len;
//         //
//         // //  Save current TSI.
//         // *tsi_ = &skb.tsi;
//         //
//         // //  Move the the next pgm_msgv_t structure.
//         // pgm_msgv_processed+= 1;
//         // // zmq_assert (pgm_msgv_processed <= pgm_msgv_len);
//         // nbytes_processed += raw_data_len;
//         //
//         // return raw_data_len;
//     }

//     pub fn process_upstream(&mut self) {
//         todo!();
//         // pgm_msgv_t dummy_msg;
//         //
//         // size_t dummy_bytes = 0;
//         // pgm_error_t *pgm_error = null_mut();
//         //
//         // let status: i32 = pgm_recvmsgv (sock, &dummy_msg, 1, MSG_ERRQUEUE,
//         //                                  &dummy_bytes, &pgm_error);
//         //
//         // //  Invalid parameters.
//         // // zmq_assert (status != PGM_IO_STATUS_ERROR);
//         //
//         // //  No data should be returned.
//         // // zmq_assert (dummy_bytes == 0
//         //             && (status == PGM_IO_STATUS_TIMER_PENDING
//         //                 || status == PGM_IO_STATUS_RATE_LIMITED
//         //                 || status == PGM_IO_STATUS_WOULD_BLOCK));
//         //
//         // last_rx_status = status;
//         //
//         // if (status == PGM_IO_STATUS_TIMER_PENDING)
//         //   // errno = EBUSY;
//         // else if (status == PGM_IO_STATUS_RATE_LIMITED)
//         //   // errno = ENOMEM;
//         // else
//         //   // errno = EAGAIN;
//     }

//     pub fn compute_sqns(&mut self, tpdu_: i32) -> i32 {
//         todo!();
//         //  Convert rate into B/ms.
//         // u64 rate = u64 (options.rate) / 8;
//         //
//         // //  Compute the size of the buffer in bytes.
//         // u64 size = u64 (options.recovery_ivl) * rate;
//         //
//         // //  Translate the size into number of packets.
//         // u64 sqns = size / tpdu_;
//         //
//         // //  Buffer should be able to hold at least one packet.
//         // if (sqns == 0)
//         //     sqns = 1;
//         //
//         // return  sqns;
//     }
// }
