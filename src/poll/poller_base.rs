use std::collections::HashMap;
use std::ffi::{c_char, c_void};
use std::sync::atomic::AtomicU32;

use chrono::{Duration, Utc};
use crate::ctx::ZmqContext;

use crate::io::thread::ZmqThread;
use crate::poll::poller_event::ZmqPollerEvent;

// pub struct TimerInfo<'a> {
//     pub sink: &'a mut ZmqPollerEvent<'a>,
//     pub id: i32,
// }

// pub type ZmqTimers<'a> = HashMap<u64, TimerInfo<'a>>;

// #[derive(Default, Debug, Clone)]
// pub struct ZmqPollerBase<'a> {
//     pub _clock: Duration,
//     pub _timers: ZmqTimers<'a>,
//     pub _load: AtomicU32,
// }

// pub struct ZmqWorkerPollerBase<'a> {
//     pub _poller_base: ZmqPollerBase<'a>,
//     pub _active: bool,
//     pub _ctx: &'a mut ZmqContext<'a>,
//     pub _worker: ZmqThread<'a>,
// }

// impl<'a> ZmqPollerBase<'a> {
//     pub fn get_load(&mut self) -> i32 {
//         return self._load.get();
//     }
//
//     pub fn adjust_load(&mut self, amount_: i32) {
//         if amount_ > 0 {
//             self._load.add(amount_);
//         } else {
//             self._load.sub(-amount_);
//         }
//     }
//
//     pub fn add_timer(&mut self, timeout_: i32, sink_: Option<&mut ZmqPollerEvent>, id_: i32) {
//         let expiration = Utc::now() + Duration::milliseconds(timeout_ as i64);
//
//         let l_sink = if sink_.is_some() {
//             sink_.unwrap()
//         } else {
//             &mut ZmqPollerEvent::default();
//         };
//
//
//         let info: TimerInfo = TimerInfo {
//             sink: l_sink,
//             id: id_,
//         };
//         self._timers.insert(expiration.timestamp_millis() as u64, info);
//     }
//
//     pub fn cancel_timer(&mut self, sink_: Option<&ZmqPollerEvent>, id_: i32) {
//         let mut to_remove: Vec<u64> = Vec::new();
//         for (key, value) in &self._timers {
//             if sink_.is_some() {
//                 if value.sink != sink_.unwrap() {
//                     continue;
//                 }
//             }
//             if value.id != id_ {
//                 continue;
//             }
//             to_remove.push(*key);
//         }
//         for key in to_remove {
//             self._timers.remove(&key);
//         }
//     }
//
//     pub fn execute_timers(&mut self) -> u64 {
//         if self._timers.len() == 0 {
//             return 0;
//         }
//
//         let mut res = 0u64;
//
//         let current = Utc::now().timestamp_millis() as u64;
//
//         for k in self._timers.keys() {
//             let info = self._timers.get_mut(k).unwrap();
//             if *k > current {
//                 continue;
//             }
//
//             info.sink.timer_event(info.id);
//             self._timers.remove(k);
//         }
//
//         res
//     }
// }

// impl<'a> ZmqWorkerPollerBase<'a> {
//     pub fn new(ctx_: &mut ZmqContext) -> Self {
//         Self {
//             _poller_base: ZmqPollerBase {
//                 _clock: Duration::milliseconds(0),
//                 _timers: HashMap::new(),
//                 _load: AtomicU32::new(0),
//             },
//             _active: false,
//             _ctx: ctx_,
//             _worker: ZmqThread::default(),
//         }
//     }
//
//     pub fn stop_worker(&mut self) {
//         self._worker.stop();
//     }
//
//     pub fn start(&mut self, name_: &str) {
//         // TODO: figure out how to pass ZmqWorkerPoller to thread as arg
//         self._ctx.start_thread(&mut self._worker, Self::worker_routine, &mut [0u8], name_);
//     }
//
//     pub fn check_thread(&mut self) {
//         unimplemented!("check_thread")
//     }
//
//     // TODO: fix up to get worker instance from arg
//     pub fn worker_routine(arg_: *mut c_void) {
//         let worker: &mut ZmqWorkerPollerBase = unsafe { &mut *(arg_ as *mut ZmqWorkerPollerBase) };
//         worker.loop_();
//     }
// }
