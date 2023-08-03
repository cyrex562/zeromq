use std::collections::HashMap;
use crate::atomic_counter::atomic_counter_t;

pub struct metadata_t
{
    pub _ref_cnt: atomic_counter_t,
    pub _dict: HashMap<String,String>,
}

impl metadata_t {
    pub fn new(dict_: &HashMap<String,String>) -> Self {
        Self {
            _ref_cnt: atomic_counter_t::new(1),
            _dict: dict_.clone(),
        }
    }

    pub fn get(&self, property_: &str) -> &str {
        self._dict.get(property_).unwrap()
    }

    pub fn add_ref(&mut self) {
        self._ref_cnt.add(1);
    }

    pub fn drop_ref(&mut self) -> bool {
        if self._ref_cnt.sub(1) == 0 {
            // delete self;
            return true;
        }
        false
    }
}