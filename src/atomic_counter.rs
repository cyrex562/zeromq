use std::sync::atomic::{AtomicU32, Ordering};

pub type integer_t = AtomicU32;

pub struct atomic_counter_t {
    pub value: integer_t,
    pub sync: Mutex<()>,
}

impl atomic_counter_t {
    pub fn new(value: integer_t) -> atomic_counter_t {
        atomic_counter_t {
            value: value,
        }
    }

    pub fn sub(decrement: integer_t) -> bool {
        let result = self.value.fetch_sub(decrement, Ordering::SeqCst);
        result != 0
    }

    pub fn add(increment: integer_t) -> integer_t {
        let result = self.value.fetch_add(increment, Ordering::SeqCst);
        result
    }

    pub fn get() -> integer_t {
        return self.value;
    }

    pub fn set(value: integer_t) {      
        self.value = value;
    }
}