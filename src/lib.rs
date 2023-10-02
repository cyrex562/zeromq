mod address;
mod array;
mod atomic_counter;
mod atomic_ptr;
mod blob;
mod channel;
mod command;
mod config;
mod ctx;
mod dbuffer;
mod defines;
mod endpoint;
mod fd;
mod i_decoder;
mod i_engine;
mod i_mailbox;
mod i_poll_events;
mod io_thread;
mod ip;
mod ip_resolver;
mod mailbox;
mod metadata;
mod msg;
mod mutex;
mod object;
mod options;
mod own;
mod pipe;
mod singaler;
mod socket_base;
mod tcp_address;
mod udp_address;
mod utils;
mod ypipe;
mod ypipe_base;
mod ypipe_conflate;
mod yqueue;
mod reaper;
mod select;
mod poller_base;
mod thread;
mod poller;
mod clock;
mod io_object;
mod decoder;
mod decoder_allocators;
mod encoder;
mod i_encoder;
mod err;
mod fq;
mod generic_mtrie;
mod lb;
mod mailbox_safe;
mod condition_variable;
mod mechanism_base;
mod mechanism;
mod mtrie;
mod polling_util;
mod proxy;
<<<<<<< Updated upstream
mod socket_poller;
=======
>>>>>>> Stashed changes

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
