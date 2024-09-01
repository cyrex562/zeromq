// use std::sync::atomic::{AtomicI32, Ordering};

// #[derive(Default,Debug,Clone)]
// pub struct ZmqAtomicCounter
// {
//     pub _value: AtomicI32,
// }
// 
// impl ZmqAtomicCounter
// {
//     pub fn new(value_: i32) -> Self {
//         Self {
//             _value: AtomicI32::new(value_)
//         }
//     }
// 
//     pub fn set(&mut self, value_: i32) {
//         self._value.store(value_, Ordering::Relaxed);
//     }
// 
//     pub fn add(&mut self, increment_: i32) -> i32 {
//         self._value.fetch_add(increment_, Ordering::Relaxed)
//     }
// 
//     pub fn sub(&mut self, decrement_: i32) -> i32 {
//         self._value.fetch_sub(decrement_, Ordering::Relaxed)
//     }
// 
//     pub fn get(&mut self) -> i32 {
//         self._value.load(Ordering::Relaxed)
//     }
// }
