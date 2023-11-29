use std::sync::atomic::AtomicU32;
use libc::size_t;
// use crate::defines::atomic_counter::ZmqAtomicCounter;
use crate::msg::MsgFreeFn;

#[derive(Default, Debug, Clone)]
pub struct ZmqContent {
    pub data: Vec<u8>,
    pub size: size_t,
    pub hint: Vec<u8>,
    pub refcnt: AtomicU32,
    pub ffn: Option<MsgFreeFn>,
}
