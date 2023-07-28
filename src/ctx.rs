use std::collections::{HashMap, HashSet};
use crate::socket_base::socket_base_t;

#[cfg(target_os="windows")]
pub type pid_t = i32;

pub struct endpoint_t
{
    pub socket: *mut socket_base_t,
    pub options: options_t,
}

pub struct thread_ctx_t
{
    pub _opt_sync: mutex_t,
    pub _thread_priority: i32,
    pub _thread_sched_policy: i32,
    pub _thread_affinity_cpus: HashSet<i32>,
    pub _thread_name_prefix: String,

}

pub const term_tid: i32 = 0;
pub const reaper_tid: i32 = 1;

pub struct pending_connection_t
{
    pub endpoint: endpoint_t,
    pub connect_pipe: *mut pipe_t,
    pub bind_pipe: *mut pipe_t,
}

pub enum side {
    connect_side,
    bind_side,
}

pub struct ctx_t
{
    pub _thread_ctx: thread_ctx_t,
    pub _tag: u32,
    pub _sockets: Vec<socket_base_t>,
    pub _empty_slots: Vec<u32>,
    pub _starting: bool,
    pub _slot_sync: mutex_t,
    pub _reaper: *mut reaper_t,
    pub _io_threads: io_threads_t,
    pub _slots: Vec<*mut i_mailbox>,
    pub _term_mailbox: mailbox_t,
    pub _endpoints: HashMap<String, enpoint_t>,
    pub _pending_connections: HashMap<String, pending_connection_t>,
    pub _endpoints_sync: mutex_t,
    pub max_socket_id: aomic_counter_t,
    pub _max_sockets: i32,
    pub _max_msgsz: i32,
    pub _io_thread_count: i32,
    pub _blocky: bool,
    pub _ipv6: bool,
    pub _zero_copy: bool,
    #[cfg(feature="fork")]
    pub _pid: pid_t,
}

