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
mod decoder_allocators;
mod defines;
mod dgram;
mod dish;
mod dist;
mod encoder;
mod endpoint;
mod err;
mod fd;
mod fq;
mod gather;
mod generic_mtrie;
mod i_decoder;
mod i_encoder;
mod i_engine;
mod i_mailbox;
mod i_poll_events;
mod io_object;
mod io_thread;
mod ip;
mod ip_resolver;
mod lb;
mod mailbox;
mod mailbox_safe;
mod mechanism;
mod mechanism_base;
mod metadata;
mod msg;
mod mtrie;
mod mutex;
mod null_mechanism;
mod object;
mod options;
mod own;
mod pair;
mod peer;
mod pipe;
mod poller;
mod poller_base;
mod polling_util;
mod proxy;
mod r#pub;
mod pull;
mod push;
mod radio;
mod radix_tree;
mod random;
mod raw_decoder;
mod raw_encoder;
mod raw_engine;
mod reaper;
mod select;
mod server;
mod session_base;
mod signaler;
mod socket_base;
mod socket_poller;
mod stream;
mod stream_connecter_base;
mod stream_engine_base;
mod stream_listener_base;
mod tcp;
mod tcp_address;
mod tcp_connecter;
mod thread;
mod trie;
mod udp_address;
mod udp_engine;
mod utils;
mod v1_decoder;
mod v1_encoder;
mod v2_decoder;
mod v2_encoder;
mod v2_protocol;
mod v3_1_encoder;
mod xpub;
mod xsub;
mod ypipe;
mod ypipe_base;
mod ypipe_conflate;
mod yqueue;
mod zap_client;
mod zmtp_engine;
mod timers;
mod tcp_listener;
mod rep;
mod req;
mod router;
mod scatter;
mod sub;
mod zmq_draft;
mod zmq_utils;
mod zmq_ops;

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
