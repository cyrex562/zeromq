use crate::blob::blob_t;
use crate::ypipe_base::ypipe_base_t;

pub trait i_pipe_events
{
    fn read_activated(&self, pipe_: pipe_t);
    fn write_activated(&self, pipe_: pipe_t);
    fn hiccuped(&self, pipe_: pipe_t);
    fn pipe_terminated(&self, pipe_: pipe_t);

}

pub enum pipe_state
{
    active,
delimiter_received,
waiting_for_delimiter,
term_ack_sent,
term_req_sent1,
term_req_sent2,
}

pub struct pipe_t{
    pub _in_pipe: *mut ypipe_base_t<msg_t>,
    pub _out_pipe: *mut ypipe_base_t<msg_t>,
    pub _in_active: bool,
    pub _out_active: bool,
    pub _hwm: i32,
    pub _lwm: i32,
    pub _in_hwm_boost: i32,
    pub _out_hwm_boost: i32,
    pub _msgs_read: u64,
    pub _msgs_written: u64,
    pub _peers_msgs_read: u64,
    pub _peer: *mut pipe_t,
    pub _sink: *mut dyn i_pipe_events,
    pub _state: pipe_state,
    pub _delay: bool,
    pub _router_socket_routing_id: blob_t,
    pub _server_socket_routing_id: i32,
    pub _conflate: bool,
    pub _endpoint_pair: endpoint_uri_pair_t,
    pub _disconnect_msg: msg_t,
}

impl pipe_t {

}