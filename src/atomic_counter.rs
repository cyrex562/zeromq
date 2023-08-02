use std::sync::atomic::{AtomicI32, Ordering};

pub struct atomic_counter_t
{
    pub _value: AtomicI32,
}

impl atomic_counter_t
{
    pub fn new(value_: i32) -> Self {
        Self {
            _value: AtomicI32::new(value_)
        }
    }

    pub fn set(value_: i32) {
        self._value.store(value_, Ordering::Relaxed);
    }

    pub fn add(&mut self, increment_: i32) -> i32 {
        self._value.fetch_add(increment_, Ordering::Relaxed)
    }

    pub fn sub(&mut self, decrement_: i32) -> i32 {
        self._value.fetch_sub(decrement_, Ordering::Relaxed)
    }

    pub fn get(&mut self) -> i32 {
        self._value.load(Ordering::Relaxed)
    }
}