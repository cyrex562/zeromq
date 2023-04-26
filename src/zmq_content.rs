use crate::atomic_counter::AtomicCounter;

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct ZmqContent {
    pub data: Vec<u8>,
    pub size: usize,
    // msg_free_fn: *ffn;
    pub hint: Vec<u8>,
    pub refcnt: AtomicCounter,
}
