use crate::fq::fq_t;
use crate::msg::msg_t;
use crate::socket_base::socket_base_t;

pub type subscriptions_t = HashSet<String>;

pub struct dish_t<'a> {
    pub socket_base: socket_base_t<'a>,
    pub _fq: fq_t,
    pub _dist: dist_t,
    pub _subscriptions: subscriptions_t,
    pub _has_message: bool,
    pub _message: msg_t,
}

impl dish_t {
    
}
