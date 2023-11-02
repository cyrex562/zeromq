use std::cmp;
use std::collections::{HashMap, HashSet};
use crate::defines::clock::ZmqClock;
use crate::err::ZmqError;

pub type TimersTimerFn = fn(i32, &mut [u8]);

pub struct Timer<'a> {
    pub timer_id: i32,
    pub interval: usize,
    pub handler: TimersTimerFn,
    pub arg: &'a mut [u8],
}

pub type TimersMap<'a> = HashMap<u64, Timer<'a>>;

pub type CancelledTimers = HashSet<i32>;

pub struct MatchById {
    pub _timer_id: i32,
}

pub struct Timers<'a> {
    pub _tag: u32,
    pub _next_timer_id: i32,
    pub _clock: ZmqClock,
    pub _timers: TimersMap<'a>,
    pub _cancelled_timers: CancelledTimers,
    pub match_by_id: MatchById,
}

impl Timers {
    pub fn new() -> Timers<'static> {
        Timers {
            _tag: 0,
            _next_timer_id: 0,
            _clock: ZmqClock::new(),
            _timers: HashMap::new(),
            _cancelled_timers: HashSet::new(),
            match_by_id: MatchById { _timer_id: 0 },
        }
    }

    pub unsafe fn check_tag(&mut self) -> bool {
        self._tag == 0xCAFEDADA
    }

    pub unsafe fn add(&mut self, interval_: i32, handler_: TimersTimerFn, arg_: &mut [u8]) -> Result<i32, ZmqError> {
        // if (handler_ == NULL) {
        //     errno = EFAULT;
        //     return -1;
        // }

        let when = self._clock.now_ms() + interval_;
        self._next_timer_id += 1;
        let timer = Timer { timer_id: self._next_timer_id, interval: interval_ as usize, handler: handler_, arg: arg_ };
        self._timers.insert(when, timer);

        return Ok(timer.timer_id.clone());
    }

    // int zmq::timers_t::cancel (int timer_id_)
    pub unsafe fn cancel(&mut self, timer_id_: i32) -> i32 {
        // check first if timer exists at all
        // if (_timers.end ()
        //     == std::find_if (_timers.begin (), _timers.end (),
        //                      match_by_id (timer_id_))) {
        //     // errno = EINVAL;
        //     return -1;
        // }
        if self._timers.iter().find(|x| x.1.timer_id == timer_id_).is_none() {
            return -1;
        }

        // check if timer was already canceled
        // if (self._cancelled_timers.count (timer_id_)) {
        //     // errno = EINVAL;
        //     return -1;
        // }
        if self._cancelled_timers.iter().find(|x| x == &timer_id_).is_some() {
            return -1;
        }

        self._cancelled_timers.insert(timer_id_);

        return 0;
    }

    // int zmq::timers_t::set_interval (int timer_id_, size_t interval_)
    pub unsafe fn set_interval(&mut self, timer_id: i32, interval_: usize) -> i32 {
        // const timersmap_t::iterator end = _timers.end ();
        // const timersmap_t::iterator it =
        //   std::find_if (_timers.begin (), end, match_by_id (timer_id_));
        let y = self._timers.iter_mut().find(|x| x.1.timer_id == timer_id);
        if y.is_some() {
            let mut timer = y.unwrap().1;
            timer.interval = interval_;
            let when = self._clock.now_ms() + interval_;
            // _timers.erase (it);
            self._timers.remove(&y.unwrap().0);
            self._timers.insert(when, timer.clone());

            return 0;
        }

        // errno = EINVAL;
        return -1;
    }

    // int zmq::timers_t::reset (int timer_id_)
    pub unsafe fn reset(&mut self, timer_id: i32) -> i32 {
        // const timersmap_t::iterator end = _timers.end ();
        // const timersmap_t::iterator it = std::find_if (_timers.begin (), end, match_by_id (timer_id_));
        let y = self._timers.iter_mut().find(|x| x.1.timer_id == timer_id);

        if y.is_some() {
            let timer = y.unwrap().1;
            let when = self._clock.now_ms() + timer.interval;
            // self._timers.erase (it);
            self._timers.remove(&y.unwrap().0);
            self._timers.insert(when, timer.clone());

            return 0;
        }

        // errno = EINVAL;
        return -1;
    }

    // long zmq::timers_t::timeout ()
    pub unsafe fn timeout(&mut self) -> i32 {
        let now = self._clock.now_ms() as i32;
        let mut res: i32 = -1;

        // const timersmap_t::iterator begin = _timers.begin ();
        // const timersmap_t::iterator end = _timers.end ();
        // timersmap_t::iterator it = begin;
        // for (; it != end; ++it)
        for it in self._timers.iter_mut() {
            if 0 == self._cancelled_timers.remove(&it.1.timer_id) {
                //  Live timer, lets return the timeout
                res = cmp::max(it.0 - now, 0i32);
                break;
            }
        }

        //  Remove timed-out timers
        // _timers.erase (begin, it);
        self._timers.retain(|x, _| x > &(now as u64));

        return res;
    }

    // int zmq::timers_t::execute ()
    pub unsafe fn execute(&mut self) -> i32 {
        let now = self._clock.now_ms();

        // const timersmap_t::iterator begin = _timers.begin ();
        // const timersmap_t::iterator end = _timers.end ();
        // timersmap_t::iterator it = _timers.begin ();
        // for (; it != end; ++it)
        for it in self._timers.iter_mut() {
            if 0 == self._cancelled_timers.remove(&it.1.timer_id) {
                //  Timer is not cancelled

                //  Map is ordered, if we have to wait for current timer we can Stop.
                if it.0 > &now {
                    break;
                }

                let timer = it.1;

                timer.handler(timer.timer_id, timer.arg);

                self._timers.insert(
                    now + timer.interval, timer.clone());
            }
        }
        // self._timers.erase (begin, it);
        self._timers.retain(|x, _| x > &now);

        return 0;
    }
}
