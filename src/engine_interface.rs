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

// #ifndef __ZMQ_I_ENGINE_HPP_INCLUDED__
// #define __ZMQ_I_ENGINE_HPP_INCLUDED__

// #include "endpoint.hpp"
// #include "macros.hpp"

//  Abstract interface to be implemented by various engines.

use crate::endpoint_uri::EndpointUriPair;
use crate::session_base::ZmqSessionBase;
use crate::thread_context::ZmqThreadContext;

enum ZmqErrorReason {
    ProtocolError,
    ConnectionError,
    TimeoutError,
}

pub trait ZmqEngineInterface {
    // virtual ~ZmqIEngine () ZMQ_DEFAULT;

    //  Indicate if the engine has an handshake stage.
    //  If engine has handshake stage, engine must call session.engine_ready when the handshake is complete.
    // virtual bool has_handshake_stage () = 0;
    fn has_handshake_state(&self) -> bool;

    //  Plug the engine to the session.
    // virtual void Plug (ZmqIoThread *io_thread_, pub struct ZmqSessionBase *session_) = 0;
    fn plug(&mut self, io_thread: &mut ZmqThreadContext, session: &mut ZmqSessionBase);

    //  Terminate and deallocate the engine. Note that 'detached'
    //  events are not fired on termination.
    // virtual void terminate () = 0;
    fn terminate(&mut self);

    //  This method is called by the session to signalise that more
    //  messages can be written to the pipe.
    //  Returns false if the engine was deleted due to an error.
    //  TODO it is probably better to change the design such that the engine
    //  does not delete itself
    // virtual bool restart_input () = 0;
    fn restart_input(&mut self) -> bool;

    //  This method is called by the session to signalise that there
    //  are messages to send available.
    // virtual void restart_output () = 0;
    fn restart_output(&mut self);

    // virtual void zap_msg_available () = 0;
    fn zap_msg_available(&mut self);

    // virtual const EndpointUriPair &get_endpoint () const = 0;
    fn get_endpoint(&mut self) -> &EndpointUriPair;
}
// }

// #endif
