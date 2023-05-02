use crate::atomic_counter::AtomicCounter;
use crate::zmq_draft_hdr::ZMQ_MSG_PROPERTY_ROUTING_ID;
use std::collections::HashMap;

// public:
//     typedef std::map<std::string, std::string> dict_t;

// #include "precompiled.hpp"
// #include "metadata.hpp"
pub struct ZmqMetadata {
    pub _dict: HashMap<String, String>,
    pub _ref_cnt: AtomicCounter,
    // // private:
    //   //  Reference counter.
    //   AtomicCounter _ref_cnt;
    //
    //   //  Dictionary holding metadata.
    //   const dict_t _dict;
    //
    //   // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqMetadata)
}

impl ZmqMetadata {
    // ZmqMetadata (const dict_t &dict_);
    // ZmqMetadata::ZmqMetadata (const dict_t &dict_) : _ref_cnt (1), _dict (dict_)
    pub fn new(dict_: &mut HashMap<String, String>) -> Self {
        Self {
            _ref_cnt: AtomicCounter::new(),
            _dict: dict_.clone(),
        }
    }

    //  Returns pointer to property value or NULL if
    //  property is not found.
    // const char *get (property_: &str) const;
    // const char *ZmqMetadata::get (property_: &str) const
    pub fn get(&mut self, property_: &str) -> Option<String> {
        // const dict_t::const_iterator it = _dict.find (property_);
        let it = self._dict.get(property_);
        if it.is_none() {
            /** \todo remove this when support for the deprecated name "Identity" is dropped */
            if property_ == "Identity" {
                return self.get(ZMQ_MSG_PROPERTY_ROUTING_ID);
            }

            return None;
        }
        let x = it.unwrap().clone();
        return Some(x);
    }

    // void add_ref ();
    pub fn add_ref(&mut self) {
        self._ref_cnt.add(1);
    }

    //  Drop reference. Returns true iff the reference
    //  counter drops to zero.
    // bool drop_ref ();
    pub fn drop_ref(&mut self) -> bool {
        !self._ref_cnt.sub(1)
    }
}
