mod address;


mod command;

mod decoder;
mod defines;


mod dist;
mod encoder;
mod endpoint;
mod engine;

mod io;
mod ip;

mod mechanism;

mod metadata;
mod msg;


mod net;

mod object;
mod options;

mod own;

mod pipe;
mod poll;

mod socket;
mod stream;
mod stream_connecter;
mod stream_listener;
mod tcp;
mod utils;
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
