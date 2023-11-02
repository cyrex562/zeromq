mod address;
mod array;
mod atomic_counter;
mod atomic_ptr;
mod blob;
mod channel;
mod client;
mod clock;
mod command;
mod condition_variable;
mod config;
mod ctx;
mod dbuffer;
mod dealer;
mod decoder;
mod defines;
mod dgram;
mod dish;
mod dist;
mod encoder;
mod endpoint;
mod engine;
mod err;
mod fd;
mod gather;
mod generic_mtrie;
mod i_decoder;
mod i_encoder;
mod i_engine;
mod i_mailbox;
mod io;
mod ip;
mod ip_address;
mod ip_resolver;
mod ip_resolver_options;
mod mailbox_safe;
mod mechanism;
mod mechanism_base;
mod metadata;
mod msg;
mod mtrie;
mod mutex;
mod net;
mod null_mechanism;
mod object;
mod options;
mod out_pipe;
mod own;
mod pair;
mod peer;
mod pipe;
mod poll;
mod poller;
mod poller_base;
mod poller_event;
mod polling_util;
mod pollitem;
mod r#pub;
mod pull;
mod push;
mod radio;
mod radix_tree;
mod raw_decoder;
mod raw_encoder;
mod raw_engine;
mod rep;
mod req;
mod router;
mod session_base;
mod socket;
mod stream;
mod stream_connecter_base;
mod stream_listener_base;
mod tcp;
mod utils;
mod v2_protocol;
mod ypipe;
mod zap_client;
mod zmq_ops;
mod session;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
