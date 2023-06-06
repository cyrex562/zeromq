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

// #include "precompiled.hpp"
// #include "macros.hpp"

// #include <limits.h>
// #include <string.h>

// #ifndef ZMQ_HAVE_WINDOWS
// #include <unistd.h>
// #endif

// #include <new>
// #include <sstream>

use crate::endpoint::EndpointUriPair;
use crate::fd::ZmqFileDesc;
use crate::message::ZmqMessage;
use crate::metadata::ZmqMetadata;

use crate::raw_decoder::RawDecoder;
use crate::raw_encoder::RawEncoder;
use crate::stream_engine_base::ZmqStreamEngineBase;

// #include "raw_engine.hpp"
// #include "io_thread.hpp"
// #include "session_base.hpp"
// #include "v1_encoder.hpp"
// #include "v1_decoder.hpp"
// #include "v2_encoder.hpp"
// #include "v2_decoder.hpp"
// #include "null_mechanism.hpp"
// #include "plain_client.hpp"
// #include "plain_server.hpp"
// #include "gssapi_client.hpp"
// #include "gssapi_server.hpp"
// #include "curve_client.hpp"
// #include "curve_server.hpp"
// #include "raw_decoder.hpp"
// #include "raw_encoder.hpp"
// #include "config.hpp"
// #include "err.hpp"
// #include "ip.hpp"
// #include "tcp.hpp"
// #include "likely.hpp"
// #include "wire.hpp"
pub struct RawEngine
{
    // : public ZmqStreamEngineBase
    pub stream_engine_base: ZmqStreamEngineBase,
//
//     RawEngine (fd: ZmqFileDesc, options: &ZmqOptions, const EndpointUriPair &endpoint_uri_pair_);
//     ~RawEngine ();
//     void // error (ZmqErrorReason reason_);
//     void plug_internal ();
//     bool handshake ();
//     int push_raw_ZmqMessageo_session (msg: &mut ZmqMessage);

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (raw_engine_t)
}

impl RawEngine {
    pub fn new (
        fd: ZmqFileDesc,
        options: &mut ZmqContext,
        endpoint_uri_pair_: &EndpointUriPair) -> Self

    {
// ZmqStreamEngineBase (fd, options_, endpoint_uri_pair_, false)
        Self {
            stream_engine_base: ZmqStreamEngineBase::new(fd,options,endpoint_uri_pair_, false),
        }
    }


    pub fn plug_internal (&mut self)
    {
        // no handshaking for raw sock, instantiate raw encoder and decoders
        self._encoder =  RawEncoder (self._options.out_batch_size);
        // alloc_assert (_encoder);

        self._decoder =  RawDecoder (self._options.in_batch_size);
        // alloc_assert (_decoder);

        self._next_msg = self.stream_engine_base.pull_msg_from_session;
        self._process_msg = self.push_raw_msg_to_session;

        // properties_t properties;
        let mut properties: properties_t = properties_t::Default();
        if init_properties (properties) {
            //  Compile metadata.
            // zmq_assert (_metadata == null_mut());
            self._metadata =  ZmqMetadata::new (properties);
            // alloc_assert (_metadata);
        }

        if self._options.raw_notify {
            //  For raw sockets, send an initial 0-length message to the
            // application so that it knows a peer has connected.
            // ZmqMessage connector;
            let mut connector = ZmqMessage::default();
            connector.init2();
            push_raw_msg_to_session (&connector);
            connector.close ();
            session ().flush ();
        }

        set_pollin ();
        set_pollout ();
        //  Flush all the data that may have been already received downstream.
        in_event ();
    }

    pub fn handshake (&mut self) -> bool
    {
        return true;
    }

    pub fn error(&mut self, reason_: ZmqErrorReason)
    {
        if self._options.raw_socket && self._options.raw_notify {
            //  For raw sockets, send a final 0-length message to the application
            //  so that it knows the peer has been disconnected.
            let mut terminator = ZmqMessage::default();// terminator;
            terminator.init2();
            push_raw_ZmqMessageo_session (&terminator);
            terminator.close ();
        }
        // ZmqStreamEngineBase::// error (reason_);
        self.stream_engine_base.error(reason_);
    }

    pub fn push_raw_msg_to_session (&mut self, msg: &mut ZmqMessage)->i32
    {
        if (self._metadata && self._metadata != msg.metadata ()) {
            msg.set_metadata(_metadata);
        }
        return self.stream_engine_base.push_msg_to_session (msg);
    }
}



// RawEngine::~RawEngine ()
// {
// }








