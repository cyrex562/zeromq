mod address;
mod channel;
mod ip_resolver;
mod socket_base;
mod tcp_address;
mod utils;
mod udp_address;
mod ctx;
mod own;
mod object;
mod command;
mod i_engine;
mod io_thread;
mod i_poll_events;
mod mailbox;
mod i_mailbox;
mod ypipe;
mod ypipe_base;
mod yqueue;
mod config;
mod singaler;
mod fd;
mod defines;
mod ip;
mod options;
mod array;

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
