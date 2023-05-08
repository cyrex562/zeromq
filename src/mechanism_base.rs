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

// #include "precompiled.hpp"

use bincode::options;
use libc::{EPROTO};
use crate::defines::ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_UNSPECIFIED;
use crate::mechanism::ZmqMechanism;
use crate::message::ZmqMessage;
use crate::options::ZmqOptions;
use crate::session_base::ZmqSessionBase;

// #include "mechanism_base.hpp"
// #include "session_base.hpp"
// public mechanism_t
#[derive(Default,Debug,Clone)]
pub struct ZmqMechanismBase
{
pub mechanism: ZmqMechanism,
// ZmqSessionBase *const session;
pub session: ZmqSessionBase,

}

impl ZmqMechanismBase {
    // ZmqMechanismBase (ZmqSessionBase *session_, options: &ZmqOptions);
    pub fn new(session: &mut ZmqSessionBase, options: &mut ZmqOptions) -> Self {
        let out = Self {
            mechanism: ZmqMechanism::new(options),
            session: session.clone(),
        };
        out
        }

    // int check_basic_command_structure (msg: &mut ZmqMessage) const;
    pub fn check_basic_command_structure (&mut self, msg: &mut ZmqMessage) -> i32
    {
        if msg.size () <= 1
            || msg.size () <= ((msg.data()))[0] as usize {
            session.get_socket().event_handshake_failed_protocol (
                session.get_endpoint (),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_UNSPECIFIED);
            errno = EPROTO;
            return -1;
        }
        return 0;
    }

    // void handle_error_reason (error_reason_: *const c_char, error_reason_len_: usize);
    pub fn handle_error_reason (&mut self, error_reason_: &str, error_reason_len_: usize)
    {
        let mut status_code_len = 3;
        let mut zero_digit = '0';
        let mut significant_digit_index = 0;
        let mut first_zero_digit_index = 1;
        let mut second_zero_digit_index = 2;
        let factor: i32 = 100;
        if (error_reason_len_ == status_code_len
            && error_reason_[first_zero_digit_index] == zero_digit
            && error_reason_[second_zero_digit_index] == zero_digit
            && error_reason_[significant_digit_index] >= '3'
            && error_reason_[significant_digit_index] <= '5') {
            // it is a ZAP error status code (300, 400 or 500), so emit an authentication failure event
            session.get_socket ().event_handshake_failed_auth (
                session.get_endpoint (),
                (error_reason_[significant_digit_index] - zero_digit) * factor);
        } else {
            // this is a violation of the ZAP protocol
            // TODO zmq_assert in this case?
        }
    }
    // bool zap_required () const;
    pub fn zap_required (&mut self) -> bool
    {
        return !self.options.zap_domain.is_empty();
    }
} // impl ZmqMechanismBase

// ZmqMechanismBase::ZmqMechanismBase (ZmqSessionBase *const session_,
//                                          options: &ZmqOptions) :
//     ZmqMechanism (options_), session (session_)
// {
// }






