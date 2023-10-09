use crate::ctx::ctx_t;
use crate::defines::ZMQ_DEALER;
use crate::fq::fq_t;
use crate::lb::lb_t;
use crate::options::options_t;
use crate::socket_base::socket_base_t;

pub struct dealer_t
{
    pub socket_base: socket_base_t,
    pub _fq: fq_t,
    pub _lb: lb_t,
    pub _probe_router: bool,
}

impl dealer_t {
    pub fn new(options: &mut options_t, parent_: &mut ctx_t, tid_: u32, sid_: i32)
    {
        options.type_ = ZMQ_DEALER;
        options.can_send_hello_msg = true;
        options.can_recv_hiccup_msg = true;


        Self {

        }
    }
}