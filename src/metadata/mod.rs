use std::collections::HashMap;
use crate::atomic_counter::ZmqAtomicCounter;

pub type ZmqDict = HashMap<String, String>;

#[derive(Default,Debug,Clone)]
pub struct ZmqMetadata {
    pub ref_cnt: ZmqAtomicCounter,
    pub dict: HashMap<String, String>,
}

impl ZmqMetadata {
    pub fn new(dict_: &HashMap<String, String>) -> Self {
        Self {
            ref_cnt: ZmqAtomicCounter::new(1),
            dict: dict_.clone(),
        }
    }

    pub fn get(&self, property_: &str) -> &str {
        self.dict.get(property_).unwrap()
    }

    pub fn add_ref(&mut self) {
        self.ref_cnt.add(1);
    }

    pub fn drop_ref(&mut self) -> bool {
        if self.ref_cnt.sub(1) == 0 {
            // delete self;
            return true;
        }
        false
    }
}
