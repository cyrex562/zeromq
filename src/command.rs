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

pub struct args_t_own<'a> {
    pub object: &'a mut own_t<'a>,
}

pub struct args_t_attach<'a> {
    pub engine: &'a mut dyn i_engine,
}

pub struct args_t_bind<'a> {
    pub pipe: &'a mut pipe_t<'a>,
}

pub struct args_t_activate_read {}

pub struct args_t_activate_write {
    pub msgs_read: u64,
}

pub struct args_t_hiccup<'a> {
    pub pipe: &'a mut pipe_t<'a>,
}

pub struct args_t_pipe_term {}

pub struct args_t_pipe_term_ack {}

pub struct args_t_pipe_hwm {
    pub inhwm: i32,
    pub outhwm: i32,
}

pub struct args_t_term_req<'a> {
    pub object: &'a mut own_t<'a>,
}

pub struct args_t_term {
    pub linger: i32,
}

pub struct args_t_term_ack {}

pub struct args_t_term_endpoint {
    pub endpoint: String,
}

pub struct args_t_reap<'a> {
    pub socket: &'a mut socket_base_t<'a>,
}

pub struct args_t_reaped {}

pub struct args_t_pipe_peer_stats<'a> {
    pub queue_count: u64,
    pub socket_base: &'a mut own_t<'a>,
    pub endpoint_pair: *mut endpoint_uri_pair_t,
}

pub struct args_t_pipe_stats_publish<'a> {
    pub outbound_queue_count: u64,
    pub inbound_queue_count: u64,
    pub endpoint_pair: &'a mut endpoint_uri_pair_t,
}

pub struct args_t_done {}

pub union args_t<'a> {
    pub stop: args_t_stop,
    pub plug: args_t_plug,
    pub own: args_t_own<'a>,
    pub attach: args_t_attach<'a>,
    pub bind: args_t_bind<'a>,
    pub activate_read: args_t_activate_read,
    pub activate_write: args_t_activate_write,
    pub hiccup: args_t_hiccup<'a>,
    pub pipe_term: args_t_pipe_term,
    pub pipe_term_ack: args_t_pipe_term_ack,
    pub pipe_hwm: args_t_pipe_hwm,
    pub term_req: args_t_term_req<'a>,
    pub term: args_t_term,
    pub term_ack: args_t_term_ack,
    pub term_endpoint: args_t_term_endpoint,
    pub reap: args_t_reap<'a>,
    pub reaped: args_t_reaped,
    pub pipe_peer_stats: args_t_pipe_peer_stats<'a>,
    pub pipe_stats_publish: args_t_pipe_stats_publish<'a>,
    pub done: args_t_done,
}

pub struct command_t<'a> {
    pub destination: Option<&'a mut object_t<'a>>,
    pub args: args_t<'a>,
    pub type_: type_t,
}

impl command_t {
    pub fn new() -> Self {
        Self {
            destination: None,
            args: args_t { stop: args_t_stop {} },
            type_: type_t::stop,
        }
    }
}


impl PartialEq for command_t {
    fn eq(&self, other: &Self) -> bool {
        self.destination == other.destination
    }
}
