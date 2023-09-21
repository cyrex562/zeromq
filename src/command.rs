#![allow(non_camel_case_types)]

use crate::object::object_t;
use crate::own::own_t;
use crate::socket_base::socket_base_t;
use std::ffi::c_void;
use crate::endpoint::endpoint_uri_pair_t;
use crate::i_engine::i_engine;
use crate::pipe::pipe_t;

pub enum type_t {
    stop,
    plug,
    own,
    attach,
    bind,
    activate_read,
    activate_write,
    hiccup,
    pipe_term,
    pipe_term_ack,
    pipe_hwm,
    term_req,
    term,
    term_ack,
    term_endpoint,
    reap,
    reaped,
    inproc_connected,
    conn_failed,
    pipe_peer_stats,
    pipe_stats_publish,
    done,
}

pub struct args_t_stop {}

pub struct args_t_plug {}

pub struct args_t_own {
    pub object: *mut own_t,
}

pub struct args_t_attach {
    pub engine: *mut dyn i_engine,
}

pub struct args_t_bind {
    pub pipe: *mut pipe_t,
}

pub struct args_t_activate_read {}

pub struct args_t_activate_write {
    pub msgs_read: u64,
}

pub struct args_t_hiccup {
    pub pipe: *mut c_void,
}

pub struct args_t_pipe_term {}

pub struct args_t_pipe_term_ack {}

pub struct args_t_pipe_hwm {
    pub inhwm: i32,
    pub outhwm: i32,
}

pub struct args_t_term_req {
    pub object: *mut own_t,
}

pub struct args_t_term {
    pub linger: i32,
}

pub struct args_t_term_ack {}

pub struct args_t_term_endpoint {
    endpoint: *mut String,
}

pub struct args_t_reap {
    pub socket: *mut socket_base_t,
}

pub struct args_t_reaped {}

pub struct args_t_pipe_peer_stats {
    pub queue_count: u64,
    pub socket_base: *mut own_t,
    pub endpoint_pair: *mut endpoint_uri_pair_t,
}

pub struct args_t_pipe_stats_publish {
    pub outbound_queue_count: u64,
    pub inbound_queue_count: u64,
    pub endpoint_pair: *mut endpoint_uri_pair_t,
}

pub struct args_t_done {}

pub union args_t {
    pub stop: args_t_stop,
    pub plug: args_t_plug,
    pub own: args_t_own,
    pub attach: args_t_attach,
    pub bind: args_t_bind,
    pub activate_read: args_t_activate_read,
    pub activate_write: args_t_activate_write,
    pub hiccup: args_t_hiccup,
    pub pipe_term: args_t_pipe_term,
    pub pipe_term_ack: args_t_pipe_term_ack,
    pub pipe_hwm: args_t_pipe_hwm,
    pub term_req: args_t_term_req,
    pub term: args_t_term,
    pub term_ack: args_t_term_ack,
    pub term_endpoint: args_t_term_endpoint,
    pub reap: args_t_reap,
    pub reaped: args_t_reaped,
    pub pipe_peer_stats: args_t_pipe_peer_stats,
    pub pipe_stats_publish: args_t_pipe_stats_publish,
    pub done: args_t_done,
}

pub struct command_t {
    pub destination: *mut object_t,
    pub type_: type_t,
    pub args: args_t,
}

impl command_t {
    pub fn new() -> Self {
        Self {
            destination: std::ptr::null_mut(),
            type_: type_t::stop,
            args: args_t { stop: args_t_stop {} },
        }
    }
}

impl Clone for command_t {
    fn clone(&self) -> Self {
        Self {
            destination: self.destination,
            type_: self.type_.clone(),
            args: self.args.clone(),
        }
    }
}

impl PartialEq for command_t {
    fn eq(&self, other: &Self) -> bool {
        self.destination == other.destination
    }
}
