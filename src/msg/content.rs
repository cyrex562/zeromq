use std::sync::atomic::{AtomicU32, Ordering};
use libc::size_t;
// use crate::defines::atomic_counter::ZmqAtomicCounter;
use crate::msg::MsgFreeFn;

#[derive(Default, Debug)]
pub struct ZmqContent {
    pub data: Vec<u8>,
    pub size: size_t,
    pub hint: Vec<u8>,
    pub refcnt: AtomicU32,
    pub ffn: Option<MsgFreeFn>,
}

impl Clone for ZmqContent {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            size: self.size,
            hint: self.hint.clone(),
            refcnt: AtomicU32::new(self.refcnt.load(Ordering::Relaxed)),
            ffn: self.ffn,
        }
    }
}
