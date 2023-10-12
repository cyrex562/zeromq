#![allow(non_camel_case_types)]

use std::collections::HashMap;
use std::ffi::{c_char, c_void};
use chrono::{Duration, Utc};
use libc::clock_t;
use crate::atomic_counter::atomic_counter_t;
use crate::ctx::thread_ctx_t;
use crate::i_poll_events::i_poll_events;
use crate::thread::thread_t;

pub struct timer_info_t
{
    pub sink: *mut dyn i_poll_events,
    pub id: i32,
}

pub type timers_t = HashMap<u64, timer_info_t>;
pub struct poller_base_t
{
    pub _clock: Duration,
    pub _timers: timers_t,
    pub _load: atomic_counter_t,
}

pub struct worker_poller_base_t<'a>
{
    pub _poller_base: poller_base_t,
    pub _active: bool,pub _ctx: &'a mut thread_ctx_t,
    pub _worker: thread_t,
}

impl poller_base_t {
    pub fn get_load(&mut self) -> i32 {
        return self._load.get();
    }

    pub fn adjust_load(&mut self, amount_: i32)
    {
        if amount_ > 0 {
            self._load.add(amount_);
        } else {
            self._load.sub(-amount_);
        }
    }

    pub fn add_timer(&mut self, timeout_: i32, sink_: *mut dyn i_poll_events,id_: i32) {
        let expiration = Utc::now() + Duration::milliseconds(timeout_ as i64);
        let info: timer_info_t = timer_info_t {
            sink: sink_,
            id: id_,
        };
        self._timers.insert(expiration.timestamp_millis() as u64, info);
    }

    pub fn cancel_timer(&mut self, sink_: *mut dyn i_poll_events, id_: i32) {
        let mut to_remove: Vec<u64> = Vec::new();
        for (key, value) in &self._timers {
            if value.sink == sink_ && value.id == id_ {
                to_remove.push(*key);
            }
        }
        for key in to_remove {
            self._timers.remove(&key);
        }
    }

    pub fn execute_timers(&mut self) -> u64
    {
        if self._timers.len() == 0 {
            return 0;
        }

        let mut res = 0u64;

        let current = Utc::now().timestamp_millis() as u64;

        for k in self._timers.keys() {
            let info = self._timers.get_mut(k).unwrap();
            if *k > current {
                continue;
            }

            info.sink.timer_event(info.id);
            self._timers.remove(k);

        }

        res
    }
}

impl <'a> worker_poller_base_t <'a> {
    pub fn new(ctx_: &mut thread_ctx_t) -> Self {
        Self {
            _poller_base: poller_base_t {
                _clock: Duration::milliseconds(0),
                _timers: timers_t::new(),
                _load: atomic_counter_t::new(0),
            },
            _active: false,
            _ctx: ctx_,
            _worker: thread_t::new(),
        }
    }

    pub fn stop_worker(&mut self)
    {
        self._worker.stop();
    }

    pub fn start(&mut self, name_: *const c_char) {
        self._ctx.start_thread(&mut self._worker, Self::worker_routine, self, name_);
    }

    pub fn check_thread(&mut self)
    {
        unimplemented!("check_thread")
    }

    pub fn worker_routine(arg_: *mut c_void)
    {
        let worker: &mut worker_poller_base_t = unsafe { &mut *(arg_ as *mut worker_poller_base_t) };
        worker.loop_();
    }
}
