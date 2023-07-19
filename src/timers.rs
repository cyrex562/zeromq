/*
Copyright (c) 2007-2016 Contributors as noted in the AUTHORS file

This file is part of libzmq, the ZeroMQ core engine in C+= 1.

libzmq is free software; you can redistribute it and/or modify it under
the terms of the GNU Lesser General Public License (LGPL) as published
by the Free Software Foundation; either version 3 of the License, or
(at your option) any later version.

As a special exception, the Contributors give you permission to link
this library with independent modules to produce an executable,
regardless of the license terms of these independent modules, and to
copy and distribute the resulting executable under terms of your choice,
provided that you also meet, for each linked independent module, the
terms and conditions of the license of that module. An independent
module is a module which is not derived from or based on this library.
If you modify this library, you must extend this exception to your
version of the library.

libzmq is distributed in the hope that it will be useful, but WITHOUT
ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
FITNESS FOR A PARTICULAR PURPOSE. See the GNU Lesser General Public
License for more details.

You should have received a copy of the GNU Lesser General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

// #include "precompiled.hpp"
// #include "timers.hpp"
// #include "err.hpp"

use std::collections::{HashMap, HashSet};
use std::ptr::null_mut;
use libc::{clock_t, EFAULT, EINVAL};

pub struct timer_t {
    pub timer_id: i32,
    pub interval: usize,
    // ZmqTimersimer_fn *handler;
    pub handler: ZmqTimersFn,
    pub arg: Vec<u8>,
}


// #include <algorithm>
pub struct ZmqTimers {
//
//     ZmqTimers ();
//     ~ZmqTimers ();

    //  Add timer to the set, timer repeats forever, or until cancel is called.
    //  Returns a timer_id that is used to cancel the timer.
    //  Returns -1 if there was an error.
    // int add (interval_: usize, ZmqTimersimer_fn handler_, arg_: &mut [u8]);

    //  Set the interval of the timer.
    //  This method is slow, cancelling exsting and adding a new timer yield better performance.
    //  Returns 0 on success and -1 on error.
    // int set_interval (timer_id_: i32, interval_: usize);

    //  Reset the timer.
    //  This method is slow, cancelling exsting and adding a new timer yield better performance.
    //  Returns 0 on success and -1 on error.
    // int reset (timer_id_: i32);

    //  Cancel a timer.
    //  Returns 0 on success and -1 on error.
    // int cancel (timer_id_: i32);

    //  Returns the time in millisecond until the next timer.
    //  Returns -1 if no timer is due.
    // long timeout ();

    //  Execute timers.
    //  Return 0 if all succeed and -1 if error.
    // int execute ();

    //  Return false if object is not a timers class.
    // bool check_tag () const;

    //
    //  Used to check whether the object is a timers class.
    // u32 _tag;
    pub _tag: u32,
    // _next_timer_id: i32;
    pub _next_timer_id: i32,
    //  Clock instance.
    // clock_t _clock;
    pub _clock: clock_t,

    // typedef std::multimap<u64, timer_t> timersmap_t;
    // timersmap_t _timers;
    pub _timers: HashMap<u64, timer_t>,

    // typedef std::set<int> cancelled_ZmqTimers;
    // cancelled_ZmqTimers _cancelled_timers;
    pub _cancelled_timers: HashSet<i32>,

    // struct match_by_id;

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ZmqTimers)
}

// pub struct match_by_id
// {
// match_by_id (timer_id_: i32) : _timer_id (timer_id_) {}
//
// bool operator() (timersmap_t::value_type const &entry_) const
// {
// return entry_.second.timer_id == _timer_id;
// }
//
// impl match_by_id {
//
// }
//
// //
// _timer_id: i32;
// };

impl ZmqTimers {
    // ZmqTimers::ZmqTimers () : _tag (0xCAFEDADA), _next_timer_id (0)
    pub fn new() -> Self {
        Self {
            _tag: 0xCAFEDADA,
            _next_timer_id: 0,
            _clock: 0,
            _timers: Default::default(),
            _cancelled_timers: Default::default(),
        }
    }

    // bool ZmqTimers::check_tag () const
    pub fn check_tag(&mut self) -> bool {
        return self._tag == 0xCAFEDADA;
    }

    // int ZmqTimers::add (interval_: usize, ZmqTimersimer_fn handler_, arg_: &mut [u8])
    pub fn add(&mut self, interval_: usize, handler_: ZmqTimersFn, arg_: &mut [u8]) -> i32 {
        if (handler_ == null_mut()) {
          // errno = EFAULT;
            return -1;
        }

        let mut when = _clock.now_ms() + interval_;
        // timer_t timer = {+= 1_next_timer_id, interval_, handler_, arg_};
        let mut timer = timer_t {
            timer_id: self._next_timer_id,
            interval: interval_,
            handler: handler_,
            arg: arg_.to_vec(),
        };
        // self._timers.insert (timersmap_t::value_type (when, timer));
        self._timers.insert(when, timer);

        return timer.timer_id.clone();
    }

    pub fn cancel(&mut self, timer_id_: i32) -> i32 {
        // check first if timer exists at all
        // if (_timers.end ()
        //     == std::find_if (_timers.begin (), _timers.end (),
        //                      match_by_id (timer_id_))) {
        //   // errno = EINVAL;
        //     return -1;
        // }
        for timer in self._timers {
            if timer.1.timer_id == timer_id_ {
                return -1;
            }
        }

        // check if timer was already canceled
        if (self._cancelled_timers.count(timer_id_)) {
          // errno = EINVAL;
            return -1;
        }

        self._cancelled_timers.insert(timer_id_);

        return 0;
    }

    pub fn set_interval(&mut self, timer_id_: i32, interval_: usize) -> i32 {
        // const timersmap_t::iterator end = _timers.end ();
        // const timersmap_t::iterator it =
        //   std::find_if (_timers.begin (), end, match_by_id (timer_id_));
        for timer in self._timers.iter_mut() {
            if timer.1.timer_id == timer_id {
                timer.1.interval = interval_;
                let mut when = _clock.now_ms() + interval_;
                self._timers.erase(timer.0);
                self._timers.insert(when, timer.1.clone());
                return 0;
            }
        }

      // errno = EINVAL;
        return -1;
    }

    pub fn reset(&mut self, timer_id_: i32) -> i32 {
        // const timersmap_t::iterator end = _timers.end ();
        // const timersmap_t::iterator it =
        //   std::find_if (_timers.begin (), end, match_by_id (timer_id_));
        for timer in self._timers.iter_mut() {
            if timer.1.timer_id == timer_id {
                let mut when = _clock.now_ms() + timer.1.interval;
                self._timers.erase(timer.0);
                self._timers.insert(when, timer.1.clone());
                return 0;
            }
        }

      // errno = EINVAL;
        return -1;
    }

    pub fn timeout(&mut self) -> i32 {
        let now = _clock.now_ms();
        let mut res = -1;

        // const timersmap_t::iterator begin = _timers.begin ();
        // const timersmap_t::iterator end = _timers.end ();
        // timersmap_t::iterator it = begin;
        // for (; it != end; += 1it) {
        //     if (0 == _cancelled_timers.erase (it.second.timer_id)) {
        //         //  Live timer, lets return the timeout
        //         res = std::max (static_cast<long> (it.first - now), 0l);
        //         break;
        //     }
        // }
        for timer in self._timers.iter_mut() {
            if timer.1.timer_id == timer_id {
                if 0 == _cancelled_timers.erase(timer.1.timer_id) {
                    //  Live timer, lets return the timeout
                    res = i32::max((timer.0 - now), 0);
                    break;
                }
            }
        }

        //  Remove timed-out timers
        // self._timers.erase (begin, it);

        return res;
    }

    pub fn execute(&mut self) -> i32 {
        let now = _clock.now_ms();

        // const timersmap_t::iterator begin = _timers.begin ();
        // const timersmap_t::iterator end = _timers.end ();
        // timersmap_t::iterator it = _timers.begin ();
        // for (; it != end; += 1it) {
        //     if (0 == _cancelled_timers.erase (it.second.timer_id)) {
        //         //  Timer is not cancelled
        //
        //         //  Map is ordered, if we have to wait for current timer we can Stop.
        //         if (it.first > now)
        //             break;
        //
        //         const timer_t &timer = it.second;
        //
        //         timer.handler (timer.timer_id, timer.arg);
        //
        //         _timers.insert (
        //           timersmap_t::value_type (now + timer.interval, timer));
        //     }
        // }
        for timer in self._timers.iter_mut() {
            if timer.1.timer_id == timer_id {
                if 0 == _cancelled_timers.erase(timer.1.timer_id) {
                    //  Timer is not cancelled

                    //  Map is ordered, if we have to wait for current timer we can Stop.
                    if (timer.0 > now) {
                        break;
                    }

                    let timer = timer.1.clone();

                    timer.handler(timer.timer_id, timer.arg);

                    self._timers.insert(
                        now + timer.interval, timer);
                }
            }
        }
        // TODO
        // _timers.erase (begin, it);

        return 0;
    }
}

