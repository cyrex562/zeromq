use crate::mechanism_base::mechanism_base_t;
use crate::msg::msg_t;
use crate::options::options_t;
use crate::session_base::session_base_t;

pub struct zap_client_t {
    pub mechanism_base: mechanism_base_t,
    pub status_code: String,
}

impl zap_client_t {
    pub fn new(session_: &mut session_base_t, peer_address_: &str, options: &options_t) -> Self {
        Self {
            mechanism_base: mechanism_base_t::new(session_, options),
            status_code: String::new(),
        }
    }

    pub fn send_zap_request(&mut self, mechanism_: &str, credentials_: &[u8]) {
        let credential_list = [credentials_];
        self.send_zap_request2(mechanism_, &credential_list, 1)
    }

    pub unsafe fn send_zap_request2(&mut self, mechanism_: &str, credentials_: &[&[u8]]) {
        // write_zap_msg cannot fail. It could only fail if the HWM was exceeded,
        // but on the ZAP socket, the HWM is disabled.

        // int rc;
        let mut rc = 0i32;
        // msg_t msg;
        let mut msg_t = msg_t::new();

        //  Address delimiter frame
        rc = msg.init2 ();
        // errno_assert (rc == 0);
        msg.set_flags (msg_t::more);
        rc = self.mechanism_base.session.write_zap_msg (&msg);
        errno_assert (rc == 0);

        //  Version frame
        rc = msg.init_size (zap_version_len);
        errno_assert (rc == 0);
        memcpy (msg.data (), zap_version, zap_version_len);
        msg.set_flags (msg_t::more);
        rc = session->write_zap_msg (&msg);
        errno_assert (rc == 0);

        //  Request ID frame
        rc = msg.init_size (id_len);
        errno_assert (rc == 0);
        memcpy (msg.data (), id, id_len);
        msg.set_flags (msg_t::more);
        rc = session->write_zap_msg (&msg);
        errno_assert (rc == 0);

        //  Domain frame
        rc = msg.init_size (options.zap_domain.length ());
        errno_assert (rc == 0);
        memcpy (msg.data (), options.zap_domain.c_str (),
                options.zap_domain.length ());
        msg.set_flags (msg_t::more);
        rc = session->write_zap_msg (&msg);
        errno_assert (rc == 0);

        //  Address frame
        rc = msg.init_size (peer_address.length ());
        errno_assert (rc == 0);
        memcpy (msg.data (), peer_address.c_str (), peer_address.length ());
        msg.set_flags (msg_t::more);
        rc = session->write_zap_msg (&msg);
        errno_assert (rc == 0);

        //  Routing id frame
        rc = msg.init_size (options.routing_id_size);
        errno_assert (rc == 0);
        memcpy (msg.data (), options.routing_id, options.routing_id_size);
        msg.set_flags (msg_t::more);
        rc = session->write_zap_msg (&msg);
        errno_assert (rc == 0);

        //  Mechanism frame
        rc = msg.init_size (mechanism_length_);
        errno_assert (rc == 0);
        memcpy (msg.data (), mechanism_, mechanism_length_);
        if (credentials_count_)
            msg.set_flags (msg_t::more);
        rc = session->write_zap_msg (&msg);
        errno_assert (rc == 0);

        //  Credentials frames
        for (size_t i = 0; i < credentials_count_; ++i) {
            rc = msg.init_size (credentials_sizes_[i]);
            errno_assert (rc == 0);
            if (i < credentials_count_ - 1)
                msg.set_flags (msg_t::more);
            memcpy (msg.data (), credentials_[i], credentials_sizes_[i]);
            rc = session->write_zap_msg (&msg);
            errno_assert (rc == 0);
        }
    }
}

pub enum zap_client_state {
    waiting_for_hello,
    sending_welcome,
    waiting_for_initiate,
    waiting_for_zap_reply,
    sending_ready,
    sending_error,
    error_sent,
    ready,
}

pub struct zap_client_common_handshake_t {
    pub zap_client: zap_client_t,
    pub state: zap_client_state,
}
