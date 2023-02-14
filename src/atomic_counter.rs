use std::sync::atomic::{AtomicU32, Ordering};

pub struct atomic_counter_t {
    _value: AtomicU32,
}

impl atomic_counter_t {
    pub fn new() -> Self {
        Self {
            _value: AtomicU32::new(0)
        }
    }

    pub fn set(&mut self, new_value: u32) {
        self._value.store(new_value, Ordering::Relaxed);
    }

    pub fn add(&mut self, increment: u32) -> u32 {
        let mut old_val = 0u32;
        old_val = self._value.fetch_add(increment, Ordering::Relaxed);
        old_val
    }

    pub fn sub(&mut self, decrement: u32) -> bool {
        let mut old_val = self._value.fetch_sub(decrement, Ordering::Relaxed);
        old_val - decrement != 0
    }

    pub fn get(&self) -> u32 {
        self._value.into_inner()
    }
}
