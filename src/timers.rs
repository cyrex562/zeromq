/*
Copyright (c) 2007-2016 Contributors as noted in the AUTHORS file

This file is part of libzmq, the ZeroMQ core engine in C++.

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

// #include <algorithm>
pub struct timers_t
{
// public:
    timers_t ();
    ~timers_t ();

    //  Add timer to the set, timer repeats forever, or until cancel is called.
    //  Returns a timer_id that is used to cancel the timer.
    //  Returns -1 if there was an error.
    int add (interval_: usize, timers_timer_fn handler_, arg_: &mut [u8]);

    //  Set the interval of the timer.
    //  This method is slow, cancelling exsting and adding a new timer yield better performance.
    //  Returns 0 on success and -1 on error.
    int set_interval (timer_id_: i32, interval_: usize);

    //  Reset the timer.
    //  This method is slow, cancelling exsting and adding a new timer yield better performance.
    //  Returns 0 on success and -1 on error.
    int reset (timer_id_: i32);

    //  Cancel a timer.
    //  Returns 0 on success and -1 on error.
    int cancel (timer_id_: i32);

    //  Returns the time in millisecond until the next timer.
    //  Returns -1 if no timer is due.
    long timeout ();

    //  Execute timers.
    //  Return 0 if all succeed and -1 if error.
    int execute ();

    //  Return false if object is not a timers class.
    bool check_tag () const;

  // private:
    //  Used to check whether the object is a timers class.
    u32 _tag;

    _next_timer_id: i32;

    //  Clock instance.
    clock_t _clock;

    typedef struct timer_t
    {
        timer_id: i32;
        interval: usize;
        timers_timer_fn *handler;
        arg: *mut c_void;
    } timer_t;

    typedef std::multimap<u64, timer_t> timersmap_t;
    timersmap_t _timers;

    typedef std::set<int> cancelled_timers_t;
    cancelled_timers_t _cancelled_timers;

    struct match_by_id;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (timers_t)
};

timers_t::timers_t () : _tag (0xCAFEDADA), _next_timer_id (0)
{
}

timers_t::~timers_t ()
{
    //  Mark the timers as dead
    _tag = 0xdeadbeef;
}

bool timers_t::check_tag () const
{
    return _tag == 0xCAFEDADA;
}

int timers_t::add (interval_: usize, timers_timer_fn handler_, arg_: &mut [u8])
{
    if (handler_ == null_mut()) {
        errno = EFAULT;
        return -1;
    }

    u64 when = _clock.now_ms () + interval_;
    timer_t timer = {++_next_timer_id, interval_, handler_, arg_};
    _timers.insert (timersmap_t::value_type (when, timer));

    return timer.timer_id;
}

struct timers_t::match_by_id
{
    match_by_id (timer_id_: i32) : _timer_id (timer_id_) {}

    bool operator() (timersmap_t::value_type const &entry_) const
    {
        return entry_.second.timer_id == _timer_id;
    }

  // private:
    _timer_id: i32;
};

int timers_t::cancel (timer_id_: i32)
{
    // check first if timer exists at all
    if (_timers.end ()
        == std::find_if (_timers.begin (), _timers.end (),
                         match_by_id (timer_id_))) {
        errno = EINVAL;
        return -1;
    }

    // check if timer was already canceled
    if (_cancelled_timers.count (timer_id_)) {
        errno = EINVAL;
        return -1;
    }

    _cancelled_timers.insert (timer_id_);

    return 0;
}

int timers_t::set_interval (timer_id_: i32, interval_: usize)
{
    const timersmap_t::iterator end = _timers.end ();
    const timersmap_t::iterator it =
      std::find_if (_timers.begin (), end, match_by_id (timer_id_));
    if (it != end) {
        timer_t timer = it->second;
        timer.interval = interval_;
        u64 when = _clock.now_ms () + interval_;
        _timers.erase (it);
        _timers.insert (timersmap_t::value_type (when, timer));

        return 0;
    }

    errno = EINVAL;
    return -1;
}

int timers_t::reset (timer_id_: i32)
{
    const timersmap_t::iterator end = _timers.end ();
    const timersmap_t::iterator it =
      std::find_if (_timers.begin (), end, match_by_id (timer_id_));
    if (it != end) {
        timer_t timer = it->second;
        u64 when = _clock.now_ms () + timer.interval;
        _timers.erase (it);
        _timers.insert (timersmap_t::value_type (when, timer));

        return 0;
    }

    errno = EINVAL;
    return -1;
}

long timers_t::timeout ()
{
    const u64 now = _clock.now_ms ();
    long res = -1;

    const timersmap_t::iterator begin = _timers.begin ();
    const timersmap_t::iterator end = _timers.end ();
    timersmap_t::iterator it = begin;
    for (; it != end; ++it) {
        if (0 == _cancelled_timers.erase (it->second.timer_id)) {
            //  Live timer, lets return the timeout
            res = std::max (static_cast<long> (it->first - now), 0l);
            break;
        }
    }

    //  Remove timed-out timers
    _timers.erase (begin, it);

    return res;
}

int timers_t::execute ()
{
    const u64 now = _clock.now_ms ();

    const timersmap_t::iterator begin = _timers.begin ();
    const timersmap_t::iterator end = _timers.end ();
    timersmap_t::iterator it = _timers.begin ();
    for (; it != end; ++it) {
        if (0 == _cancelled_timers.erase (it->second.timer_id)) {
            //  Timer is not cancelled

            //  Map is ordered, if we have to wait for current timer we can stop.
            if (it->first > now)
                break;

            const timer_t &timer = it->second;

            timer.handler (timer.timer_id, timer.arg);

            _timers.insert (
              timersmap_t::value_type (now + timer.interval, timer));
        }
    }
    _timers.erase (begin, it);

    return 0;
}
