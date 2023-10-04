use std::collections::HashMap;
use std::ffi::c_void;
use libc::clock_t;
use crate::array::array_item_t;
use crate::blob::blob_t;
use crate::defines::handle_t;
use crate::i_mailbox::i_mailbox;
use crate::i_poll_events::i_poll_events;
use crate::mutex::mutex_t;
use crate::own::own_t;
use crate::pipe::{i_pipe_events, pipe_t};
use crate::poller::poller_t;
use crate::signaler::signaler_t;

#[derive(PartialEq)]
pub struct socket_base_t
{
    pub own: own_t,
    pub array_item: array_item_t,
    pub poll_events: i_poll_events,
    pub pipe_events: i_pipe_events,
    pub _mailbox: *mut i_mailbox,
    pub _pipes: pipes_t,
    pub _poller: *mut poller_t,
    pub _handle: *mut handle_t,
    pub _last_tsc: u64,
    pub _ticks: i32,
    pub _rcvmore: bool,
    pub _clock: clock_t,
    pub _monitor_socker: *mut c_void,
    pub _monitor_events: i64,
    pub _last_endpoint: String,
    pub _thread_safe: bool,
    pub _reaper_signaler: *mut signaler_t,
    pub _monitor_sync: mutex_t,
    pub _disconnected: bool,
}

pub struct out_pipe_t {
    pub pipe: pipe_t,
    pub active: bool,
}

pub struct routing_socket_base_t
{
    pub base: socket_base_t,
    pub _out_pipes: HashMap<blob_t, out_pipe_t>,
    pub _connect_routing_id: String,
}
