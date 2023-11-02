

use std::collections::HashMap;
use std::ffi::{c_char, c_void};
use chrono::{Duration, Utc};
use crate::ctx::ZmqThreadCtx;
use crate::defines::atomic_counter::ZmqAtomicCounter;
use crate::io::thread::ZmqThread;
use crate::poll::i_poll_events::IPollEvents;


pub struct TimerInfo
{
    pub sink: *mut dyn IPollEvents,
    pub id: i32,
}

pub type ZmqTimers = HashMap<u64, TimerInfo>;
pub struct ZmqPollerBase
{
    pub _clock: Duration,
    pub _timers: ZmqTimers,
    pub _load: ZmqAtomicCounter,
}

pub struct ZmqWorkerPollerBase<'a>
{
    pub _poller_base: ZmqPollerBase,
    pub _active: bool,pub _ctx: &'a mut ZmqThreadCtx,
    pub _worker: ZmqThread<'a>,
}

impl ZmqPollerBase {
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

    pub fn add_timer(&mut self, timeout_: i32, sink_: *mut dyn IPollEvents, id_: i32) {
        let expiration = Utc::now() + Duration::milliseconds(timeout_ as i64);
        let info: TimerInfo = TimerInfo {
            sink: sink_,
            id: id_,
        };
        self._timers.insert(expiration.timestamp_millis() as u64, info);
    }

    pub fn cancel_timer(&mut self, sink_: *mut dyn IPollEvents, id_: i32) {
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

impl <'a> ZmqWorkerPollerBase<'a> {
    pub fn new(ctx_: &mut ZmqThreadCtx) -> Self {
        Self {
            _poller_base: ZmqPollerBase {
                _clock: Duration::milliseconds(0),
                _timers: ZmqTimers::new(),
                _load: ZmqAtomicCounter::new(0),
            },
            _active: false,
            _ctx: ctx_,
            _worker: ZmqThread::new(),
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
        let worker: &mut ZmqWorkerPollerBase = unsafe { &mut *(arg_ as *mut ZmqWorkerPollerBase) };
        worker.loop_();
    }
}
