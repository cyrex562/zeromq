use libc::{c_void, EFAULT};

use crate::ctx::ZmqContext;
use crate::defines::{
    ERROR_COMMAND_NAME, ERROR_COMMAND_NAME_LEN, ERROR_REASON_LEN_SIZE, READY_COMMAND_NAME,
    READY_COMMAND_NAME_LEN, ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR,
    ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND,
};
use crate::defines::err::ZmqError;
use crate::defines::err::ZmqError::MechanismError;
use crate::mechanism;
use crate::mechanism::MechanismStatus::{Error, Handshaking, Ready};
use crate::mechanism::ZmqMechanism;
use crate::msg::ZmqMsg;
use crate::options::ZmqOptions;

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
        if mechanism.zap_client.mechanism.session.zap_connect(ctx).is_err() {
            mechanism.zap_client.mechanism.session.get_socket().event_handshake_failed_no_detail(
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
            mechanism.zap_client.receive_and_process_zap_reply(options, ctx)?;

            mechanism._zap_reply_received = true;
        }
    }

    if mechanism._zap_reply_received && mechanism.zap_client.status_code != "200" {
        mechanism._error_command_sent = true;
        if mechanism.zap_client.status_code != "300" {
            let status_code_len = 3;
            let rc = msg_.init_size(ERROR_COMMAND_NAME_LEN + ERROR_REASON_LEN_SIZE + status_code_len);
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
            msg_data = &mut msg_data[ERROR_REASON_LEN_SIZE..];
            // libc::memcpy(
            //     msg_data,
            //     mechanism.zap_client.status_code.as_ptr() as *const c_void,
            //     status_code_len,
            // );
            msg_data.copy_from_slice(mechanism.zap_client.status_code.as_bytes());
            return Ok(());
        }
        // errno = EAGAIN;
        return Err(MechanismError("EAGAIN"));
    }

    mechanism.zap_client.mechanism.make_command_with_basic_properties(
        options,
        msg_,
        READY_COMMAND_NAME,
        READY_COMMAND_NAME_LEN,
    )?;

    mechanism._ready_command_sent = true;

    return Ok(());
}

pub fn null_process_handshake_command(
    options: &ZmqOptions,
    mechanism: &mut ZmqMechanism,
    msg_: &mut ZmqMsg,
) -> Result<(), ZmqError> {
    if mechanism._ready_command_received || mechanism._error_command_received {
        mechanism.zap_client.mechanism.session.get_socket().event_handshake_failed_protocol(
            options,
            mechanism.zap_client.mechanism.session.get_endpoint(),
            ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND as i32,
        );
        // errno = EPROTO;
        return Err(MechanismError("EPROTO"));
    }

    // const unsigned char *cmd_data = static_cast<unsigned char *> (msg_->data ());
    let cmd_data = msg_.data_mut();
    // const size_t data_size = msg_->size ();
    let data_size = msg_.size();

    let mut rc = 0;
    if data_size >= READY_COMMAND_NAME_LEN && libc::memcmp(cmd_data.as_ptr() as *const c_void, READY_COMMAND_NAME.as_ptr() as *const c_void, READY_COMMAND_NAME_LEN) == 0 {
        rc = mechanism.process_ready_command(cmd_data, data_size);
    } else if data_size >= ERROR_COMMAND_NAME_LEN && libc::memcmp(cmd_data.as_ptr() as *const c_void, ERROR_COMMAND_NAME.as_ptr() as *const c_void, ERROR_COMMAND_NAME_LEN) == 0 {
        rc = mechanism.process_error_command(cmd_data, data_size);
    } else {
        mechanism.zap_client.mechanism.session.get_socket().event_handshake_failed_protocol(
            options,
            mechanism.zap_client.mechanism.session.get_endpoint(),
            ZMQ_PROTOCOL_ERROR_ZMTP_UNEXPECTED_COMMAND as i32,
        );
        // errno = EPROTO;
        rc = -1;
    }

    if rc == 0 {
        msg_.close()?;
        // errno_assert (rc == 0);
        msg_.init2()?;
        // errno_assert (rc == 0);
    }
    return if rc == 0 {
        Ok(())
    } else {
        Err(MechanismError("EPROTO"))
    };
}

pub fn null_process_ready_command(
    options: &ZmqOptions,
    mechanism: &mut ZmqMechanism,
    cmd_data: &[u8],
    data_size_: usize,
) -> Result<(), ZmqError> {
    mechanism._ready_command_received = true;
    return mechanism.zap_client.mechanism.parse_metadata(
        options,
        &mut cmd_data[READY_COMMAND_NAME_LEN..],
        data_size_ - READY_COMMAND_NAME_LEN,
        false,
    );
}

pub fn null_process_error_command(
    options: &ZmqOptions,
    mechanism: &mut ZmqMechanism,
    cmd_data: &[u8],
    data_size_: usize,
) -> Result<(), ZmqError> {
    let fixed_prefix_size = ERROR_COMMAND_NAME_LEN + ERROR_REASON_LEN_SIZE;
    if data_size_ < fixed_prefix_size {
        mechanism.zap_client.mechanism.session.get_socket().event_handshake_failed_protocol(
            options,
            mechanism.zap_client.mechanism.session.get_endpoint(),
            ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR as i32,
        );

        // errno = EPROTO;
        return Err(MechanismError("EPROTO"));
    }
    let error_reason_len = cmd_data[ERROR_COMMAND_NAME_LEN];
    if error_reason_len > (data_size_ - fixed_prefix_size) as u8 {
        mechanism.zap_client.mechanism.session.get_socket().event_handshake_failed_protocol(
            options,
            mechanism.zap_client.mechanism.session.get_endpoint(),
            ZMQ_PROTOCOL_ERROR_ZMTP_MALFORMED_COMMAND_ERROR as i32,
        );

        // errno = EPROTO;
        return Err(MechanismError("EPROTO"));
    }
    let error_reason = cmd_data[fixed_prefix_size..];
    mechanism.zap_client.handle_error_reason(error_reason, error_reason_len);
    mechanism._error_command_received = true;
    return Ok(());
}

pub fn null_zap_msg_available(ctx: &mut ZmqContext, options: &ZmqOptions, mechanism: &mut ZmqMechanism) -> Result<(), ZmqError> {
    if mechanism._zap_reply_received {
        // errno = EFSM;
        return Err(MechanismError("EFSM"));
    }
    return if mechanism.zap_client.receive_and_process_zap_reply(options, ctx).is_ok() {
        mechanism._zap_reply_received = true;
        Ok(())
    } else {
        Err(MechanismError("EAGAIN"))
    };
}

pub fn null_status(mechanism: &mut ZmqMechanism) -> mechanism::MechanismStatus {
    if mechanism._ready_command_sent && mechanism._ready_command_received {
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

pub fn null_send_zap_request(mechanism: &mut ZmqMechanism, options: &mut ZmqOptions) {
    mechanism.zap_client.send_zap_request(options, "NULL", &[]);
}
