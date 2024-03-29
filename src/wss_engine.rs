/*
Copyright (c) 2007-2019 Contributors as noted in the AUTHORS file

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

use std::ptr::null_mut;
use libc::{EAGAIN, EINTR, EINVAL, EPIPE, ssize_t};
use crate::context::ZmqContext;
use crate::endpoint_uri::EndpointUriPair;
use crate::defines::ZmqFileDesc;

use crate::ws_address::WsAddress;
use crate::ws_engine::ZmqWsEngine;

// #include "precompiled.hpp"
// #include "wss_engine.hpp"
#[derive(Default, Debug, Clone)]
pub struct WssEngine {
    //  : public ZmqWsEngine
    pub ws_engine: ZmqWsEngine,
    pub _established: bool,
    pub _tls_client_cred: gnutls_certificate_credentials_t,
    pub gnutls_session_t: _tls_session,
}

impl WssEngine {
    // WssEngine (fd: ZmqFileDesc,
    //            options: &ZmqOptions,
    //             const endpoint_uri_ZmqPair &endpoint_uri_pair_,
    //             WsAddress &address_,
    //             client_: bool,
    //             tls_server_cred_: &mut [u8],
    //             hostname_: &str)

    pub fn new(fd: ZmqFileDesc,
               ctx: &mut ZmqContext,
               endpoint_uri_pair_: &mut EndpointUriPair,
               address_: &mut WsAddress,
               client_: bool,
               tls_server_cred_: Option<&mut [u8]>,
               hostname_: &str) -> Self {
// ZmqWsEngine (fd, options_, endpoint_uri_pair_, address_, client_),
//     _established (false),
//     _tls_client_cred (null_mut())
        let mut rc = 0;

        if (client_) {
            // TODO: move to session_base, to allow changing the socket options between connect calls
            rc = gnutls_certificate_allocate_credentials(&_tls_client_cred);
            // zmq_assert (rc == 0);

            if (options_.wss_trust_system) {
                gnutls_certificate_set_x509_system_trust(_tls_client_cred);
            }

            // if (options_.wss_trust_pem.length () > 0) {
            //     let mut trust = gnutls_datum_t{
            //        options_.wss_trust_pem,
            //       (unsigned int) options_.wss_trust_pem.length ()};
            //     rc = gnutls_certificate_set_x509_trust_mem (
            //       _tls_client_cred, &trust, GNUTLS_X509_FMT_PEM);
            //     // zmq_assert (rc >= 0);
            // }

            gnutls_certificate_set_verify_function(_tls_client_cred,
                                                   verify_certificate_callback);

            rc = gnutls_init(&_tls_session, GNUTLS_CLIENT | GNUTLS_NONBLOCK);
            // zmq_assert (rc == GNUTLS_E_SUCCESS);

            if (!hostname_.empty()) {
                gnutls_server_name_set(_tls_session, GNUTLS_NAME_DNS,
                                       hostname_, hostname_.size());
            }

            gnutls_session_set_ptr(
                _tls_session,
                if hostname_.is_empty() { null_mut() } else { (hostname_) });

            rc = gnutls_credentials_set(_tls_session, GNUTLS_CRD_CERTIFICATE,
                                        _tls_client_cred);
            // zmq_assert (rc == GNUTLS_E_SUCCESS);
        } else {
            // zmq_assert (tls_server_cred_);

            rc = gnutls_init(&_tls_session, GNUTLS_SERVER | GNUTLS_NONBLOCK);
            // zmq_assert (rc == GNUTLS_E_SUCCESS);

            rc = gnutls_credentials_set(_tls_session, GNUTLS_CRD_CERTIFICATE,
                                        tls_server_cred_);
            // zmq_assert (rc == GNUTLS_E_SUCCESS);
        }

        gnutls_set_default_priority(_tls_session);
        gnutls_transport_set_int(_tls_session, fd);
        Self {
            ws_engine: ZmqWsEngine::new(fd, ctx, endpoint_uri_pair_, address_, client_),
            _established: false,
            _tls_client_cred: null_mut(),
            // _tls_session: _tls_session,..Default::default()
            gnutls_session_t: (),
        }
    }

    // ~WssEngine ();

    // void out_event ();
    pub fn out_event(&mut self) {
        if (self._established) {
            return self.ws_engine.out_event();
        }

        self.do_handshake();
    }

    // bool handshake ();

    // void plug_internal ();
    pub fn plug_internal(&mut self) {
        set_pollin();
        in_event();
    }

    // int read (data: &mut [u8], size: usize);

    // int write (const data: &mut [u8], size: usize);

    // bool do_handshake ();
    pub fn do_handshake(&mut self) -> bool {
        let rc = gnutls_handshake(_tls_session);

        reset_pollout();

        if (rc == GNUTLS_E_SUCCESS) {
            start_ws_handshake();
            _established = true;
            return false;
        } else if (rc == GNUTLS_E_AGAIN) {
            let direction = gnutls_record_get_direction(_tls_session);
            if (direction == 1) {
                set_pollout();
            }

            return false;
        } else if (rc == GNUTLS_E_INTERRUPTED || rc == GNUTLS_E_WARNING_ALERT_RECEIVED) {
            return false;
        } else {
            // error (ZmqIEngine::connection_error);
            return false;
        }

        return true;
    }

    pub fn handshake(&mut self) -> bool {
        if (!self._established) {
            if (!self.do_handshake()) {
                return false;
            }
        }

        return self.we_engine.handshake();
    }

    pub fn read(&mut self, data: &mut [u8], size: usize) -> i32 {
        let rc = gnutls_record_recv(_tls_session, data, size);

        if (rc == GNUTLS_E_REHANDSHAKE) {
            gnutls_alert_send(_tls_session, GNUTLS_AL_WARNING,
                              GNUTLS_A_NO_RENEGOTIATION);
            return 0;
        }

        if (rc == GNUTLS_E_INTERRUPTED) {
          // errno = EINTR;
            return -1;
        }

        if (rc == GNUTLS_E_AGAIN) {
          // errno = EAGAIN;
            return -1;
        }

        if (rc == 0) {
          // errno = EPIPE;
            return -1;
        }

        if (rc < 0) {
          // errno = EINVAL;
            return -1;
        }

        // TODO: change return type to ssize_t (signed)
        return rc;
    }

    pub fn write(&mut self, data: &mut [u8], size: usize) -> i32 {
        let rc = gnutls_record_send(_tls_session, data, size);

        if (rc == GNUTLS_E_INTERRUPTED || rc == GNUTLS_E_AGAIN) {
            return 0;
        }

        if (rc < 0) {
          // errno = EINVAL;
            return -1;
        }

        // TODO: change return type to ssize_t (signed)
        return rc;
    }
} // WssEngine

pub fn verify_certificate_callback(session: gnutls_session_t) -> i32 {
    // unsigned int status;
    let mut status = 0u32;
    // const char *hostname;
    let mut hostname: &str = "";

    // read hostname
    hostname = gnutls_session_get_ptr(session);

    let rc = gnutls_certificate_verify_peers3(session, hostname, &status);
    // zmq_assert (rc >= 0);

    if (status != 0) {
        // TODO: somehow log the error
        // Certificate is not trusted
        return GNUTLS_E_CERTIFICATE_ERROR;
    }

    // notify gnutls to continue handshake normally
    return 0;
}


// WssEngine::~WssEngine ()
// {
//     gnutls_deinit (_tls_session);
//
//     if (_tls_client_cred)
//         gnutls_certificate_free_credentials (_tls_client_cred);
// }












