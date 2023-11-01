use crate::ctx::ZmqContext;
use crate::defines::{
    ERROR_COMMAND_NAME, ERROR_COMMAND_NAME_LEN, ERROR_REASON_LEN_SIZE, READY_COMMAND_NAME,
    READY_COMMAND_NAME_LEN, ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR,
    ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND,
};
use crate::err::ZmqError;
use crate::err::ZmqError::MechanismError;
use crate::mechanism;
use crate::mechanism::MechanismStatus::{Error, Handshaking, Ready};
use crate::mechanism::ZmqMechanism;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;
use libc::EFAULT;
use std::ffi::c_void;

// pub struct ZmqNullMechanism {
//     pub zap_client: ZapClient<'a>,
//     pub _ready_command_sent: bool,
//     pub _error_command_sent: bool,
//     pub _ready_command_received: bool,
//     pub _error_command_received: bool,
//     pub _zap_request_sent: bool,
//     pub _zap_reply_received: bool,
// }
//
// impl ZmqNullMechanism {
//     pub fn new(
//         session_: &mut ZmqSession,
//         peer_address_: &str,
//         options_: &ZmqOptions,
//     ) -> ZmqNullMechanism {
//         let mut out = ZmqNullMechanism {
//             zap_client: ZapClient::new(session_, peer_address_, options_),
//             _ready_command_sent: false,
//             _error_command_sent: false,
//             _ready_command_received: false,
//             _error_command_received: false,
//             _zap_request_sent: false,
//             _zap_reply_received: false,
//         };
//
//         out
//     }
// }

pub fn null_next_handshake_command(
    mechanism: &mut ZmqMechanism,
    options: &ZmqOptions,
    ctx: &mut ZmqContext,
    msg_: &mut ZmqMsg,
) -> Result<(), ZmqError> {
    if mechanism._ready_command_sent || mechanism._error_command_sent {
        // errno = EAGAIN;
        return Err(MechanismError("EAGAIN"));
    }

    if mechanism.zap_required(options) && !mechanism._zap_reply_received {
        if mechanism._zap_request_sent {
            // errno = EAGAIN;
            return Err(MechanismError("EAGAIN"));
        }
        //  Given this is a backward-incompatible change, it's behind a socket
        //  option disabled by default.
        if mechanism
            .zap_client
            .mechanism
            .session
            .zap_connect(ctx)
            .is_err()
        {
            mechanism
                .zap_client
                .mechanism
                .session
                .get_socket()
                .event_handshake_failed_no_detail(
                    options,
                    mechanism.zap_client.mechanism.session.get_endpoint(),
                    EFAULT,
                );
            return Err(MechanismError("EFAULT"));
        } else {
            mechanism.send_zap_request();
            mechanism._zap_request_sent = true;

            //  TODO actually, it is quite unlikely that we can read the ZAP
            //  reply already, but removing this has some strange side-effect
            //  (probably because the pipe's in_active flag is true until a read
            //  is attempted)
            rc = mechanism.zap_client.receive_and_process_zap_reply();
            if rc != 0 {
                return -1;
            }

            mechanism._zap_reply_received = true;
        }
    }

    if mechanism._zap_reply_received && mechanism.zap_client.status_code != "200" {
        mechanism._error_command_sent = true;
        if mechanism.zap_client.status_code != "300" {
            let status_code_len = 3;
            let rc =
                msg_.init_size(ERROR_COMMAND_NAME_LEN + ERROR_REASON_LEN_SIZE + status_code_len);
            // zmq_assert (rc == 0);
            let mut msg_data = (msg_.data_mut());
            // libc::memcpy(
            //     msg_data,
            //     ERROR_COMMAND_NAME.as_ptr() as *const c_void,
            //     ERROR_COMMAND_NAME_LEN,
            // );
            msg_data.copy_from_slice(ERROR_COMMAND_NAME.as_bytes());
            msg_data = &mut msg_data[ERROR_COMMAND_NAME_LEN..];
            msg_data[0] = status_code_len as u8;
            msg_data = &mut [ERROR_REASON_LEN_SIZE..];
            // libc::memcpy(
            //     msg_data,
            //     mechanism.zap_client.status_code.as_ptr() as *const c_void,
            //     status_code_len,
            // );
            msg_data.copy_from_slice(mechanism.zap_client.status_code.as_bytes());
            return 0;
        }
        // errno = EAGAIN;
        return -1;
    }

    mechanism
        .zap_client
        .mechanism
        .make_command_with_basic_properties(
            options,
            msg_,
            READY_COMMAND_NAME,
            READY_COMMAND_NAME_LEN,
        )?;

    mechanism._ready_command_sent = true;

    return 0;
}

pub unsafe fn process_handshake_command(mechanism: &mut ZmqMechanism, msg_: &mut ZmqMsg) -> i32 {
    if (mechanism._ready_command_received || mechanism._error_command_received) {
        mechanism
            .zap_client
            .mechanism
            .session
            .get_socket()
            .event_handshake_failed_protocol(
                mechanism.zap_client.mechanism.session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND,
            );
        // errno = EPROTO;
        return -1;
    }

    // const unsigned char *cmd_data = static_cast<unsigned char *> (msg_->data ());
    let cmd_data = msg_.data_mut();
    // const size_t data_size = msg_->size ();
    let data_size = msg_.size();

    let mut rc = 0;
    if (data_size >= READY_COMMAND_NAME_LEN
        && libc::memcmp(cmd_data, READY_COMMAND_NAME, READY_COMMAND_NAME_LEN) == 0)
    {
        rc = mechanism.process_ready_command(cmd_data, data_size);
    } else if (data_size >= ERROR_COMMAND_NAME_LEN
        && libc::memcmp(cmd_data, ERROR_COMMAND_NAME, ERROR_COMMAND_NAME_LEN) == 0)
    {
        rc = mechanism.process_error_command(cmd_data, data_size);
    } else {
        mechanism
            .zap_client
            .mechanism
            .session
            .get_socket()
            .event_handshake_failed_protocol(
                mechanism.zap_client.mechanism.session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND,
            );
        // errno = EPROTO;
        rc = -1;
    }

    if (rc == 0) {
        rc = msg_.close();
        // errno_assert (rc == 0);
        rc = msg_.init2();
        // errno_assert (rc == 0);
    }
    return rc;
}

pub unsafe fn process_ready_command(
    mechanism: &mut ZmqMechanism,
    cmd_data: &[u8],
    data_size_: usize,
) -> i32 {
    mechanism._ready_command_received = true;
    return mechanism.zap_client.mechanism.parse_metadata(
        cmd_data + READY_COMMAND_NAME_LEN,
        data_size_ - READY_COMMAND_NAME_LEN,
    );
}

pub unsafe fn process_error_command(
    mechanism: &mut ZmqMechanism,
    cmd_data: &[u8],
    data_size_: usize,
) -> i32 {
    let fixed_prefix_size = ERROR_COMMAND_NAME_LEN + ERROR_REASON_LEN_SIZE;
    if (data_size_ < fixed_prefix_size) {
        mechanism
            .zap_client
            .mechanism
            .session
            .get_socket()
            .event_handshake_failed_protocol(
                mechanism.zap_client.mechanism.session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR,
            );

        // errno = EPROTO;
        return -1;
    }
    let error_reason_len = cmd_data[ERROR_COMMAND_NAME_LEN];
    if (error_reason_len > (data_size_ - fixed_prefix_size) as u8) {
        mechanism
            .zap_client
            .mechanism
            .session
            .get_socket()
            .event_handshake_failed_protocol(
                mechanism.zap_client.mechanism.session.get_endpoint(),
                ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR,
            );

        // errno = EPROTO;
        return -1;
    }
    let error_reason = cmd_data[fixed_prefix_size..];
    mechanism
        .zap_client
        .handle_error_reason(error_reason, error_reason_len);
    mechanism._error_command_received = true;
    return 0;
}

pub unsafe fn zap_msg_available() -> i32 {
    if (mechanism._zap_reply_received) {
        // errno = EFSM;
        return -1;
    }
    let rc = mechanism.zap_client.receive_and_process_zap_reply();
    if (rc == 0) {
        mechanism._zap_reply_received = true;
    }
    return if rc == -1 { -1 } else { 0 };
}

pub unsafe fn status(mechanism: &mut ZmqMechanism) -> mechanism::MechanismStatus {
    if (mechanism._ready_command_sent && mechanism._ready_command_received) {
        return Ready;
    }

    let command_sent = mechanism._ready_command_sent || mechanism._error_command_sent;
    let command_received = mechanism._ready_command_received || mechanism._error_command_received;
    return if command_sent && command_received {
        Error
    } else {
        Handshaking
    };
}

pub unsafe fn send_zap_request(mechanism: &mut ZmqMechanism, options: &mut ZmqOptions) {
    mechanism.zap_client.send_zap_request(options, "NULL", &[]);
}
