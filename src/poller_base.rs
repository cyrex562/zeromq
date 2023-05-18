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
// #include "poller_base.hpp"
// #include "i_poll_events.hpp"
// #include "err.hpp"

use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use libc::clock_t;
use crate::io_thread::ZmqIoThread;
use crate::poll_events_interface::ZmqPollEventsInterface;
use crate::thread_ctx::ThreadCtx;
use crate::timers::timers_t;

#[derive(Default,Debug,Clone)]
struct timer_info_t
{
    // ZmqPollEventsInterface *sink;
    pub sink: ZmqPollEventsInterface,
    // id: i32;
    pub id: i32,
}

#[derive(Default, Debug, Clone)]
pub struct PollerBase {
    //  Clock instance private to this I/O thread.
    // clock_t _clock;
    pub _clock: clock_t,
    //  List of active timers.
    // typedef std::multimap<u64, timer_info_t> timers_t;
    // timers_t _timers;
    pub _timers: HashMap<u64, timer_info_t>,
    //  Load of the poller. Currently the number of file descriptors
    //  registered.
    // AtomicCounter _load;
    pub _load: AtomicU64,

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (poller_base_t)
}

impl PollerBase {
    //
//     PollerBase () ZMQ_DEFAULT;
//     virtual ~PollerBase ();
    // Methods from the poller concept.
    // int get_load () const;
    // void add_timer (timeout: i32, ZmqPollEventsInterface *sink_, id_: i32);
    // void cancel_timer (ZmqPollEventsInterface *sink_, id_: i32);
    //  Called by individual poller implementations to manage the load.
    // void adjust_load (amount_: i32);
    //  Executes any timers that are due. Returns number of milliseconds
    //  to wait to match the next timer or 0 meaning "no timers".
    // u64 execute_timers ();
    pub fn get_load (&mut self)
    {
        return self._load.get ();
    }

    pub fn adjust_load (&mut self, amount_: i32)
    {
        if (amount_ > 0) {
            self._load.add(amount_);
        }
        else if (amount_ < 0) {
            self._load.sub(-amount_);
        }
    }

    pub fn add_timer (&mut self, timeout: i32, sink_: &mut ZmqPollEventsInterface, id_: i32)
    {
        let expiration = self._clock.now_ms () + timeout;
        let info = timer_info_t{sink: sink_, id: id_};
        self._timers.insert(expiration, info);
    }

    pub fn cancel_timer (&mut self, sink_: &mut ZmqPollEventsInterface, id_: i32)
    {
        //  Complexity of this operation is O(n). We assume it is rarely used.
        // for (timers_t::iterator it = _timers.begin (), end = _timers.end ();
        //      it != end; += 1it)
        for it in self._timers
        {
            if it.1.sink == sink_ && it.1.id == id_ {
                self._timers.erase(it);
                return;
            }
        }

        //  We should generally never get here. Calling 'cancel_timer ()' on
        //  an already expired or canceled timer (or even worse - on a timer which
        //  never existed, supplying bad sink_ and/or id_ values) does not make any
        //  sense.
        //  But in some edge cases this might happen. As described in issue #3645
        //  `timer_event ()` call from `execute_timers ()` might call `cancel_timer ()`
        //  on already canceled (deleted) timer.
        //  As soon as that is resolved an 'assert (false)' should be put here.
    }

    pub fn execute_timers (&mut self) -> u64
    {
        //  Fast track.
        if self._timers.empty () {
            return 0;
        }

        //  Get the current time.
        let current = self._clock.now_ms ();

        //  Execute the timers that are already due.
        let mut res = 0;
        let mut timer_temp = timer_info_t::default();
        // timers_t::iterator it;

        for it in self._timers {
            // it = _timers.begin();

            //  If we have to wait to execute the item, same will be true for
            //  all the following items because multimap is sorted. Thus we can
            //  stop checking the subsequent timers.
            if it.0 > current {
                res = it.0 - current;
                break;
            }

            //  Save and remove the timer because timer_event() call might delete
            //  exactly this timer and then the iterator will be invalid.
            timer_temp = it.1;
            self._timers.erase(&it);

            //  Trigger the timer.
            timer_temp.sink.timer_event(timer_temp.id);
        }
        // } \while (!_timers.empty ());

        //  Return the time to wait for the next timer (at least 1ms), or 0, if
        //  there are no more timers.
        return res;
    }
}

//  Base class for a poller with a single worker thread.
pub struct WorkerPollerBase<'a>
{
    //  : public PollerBase
    pub poller_base: PollerBase,
    // Reference to ZMQ context.
    // const ThreadCtx &ctx;
    pub ctx: &'a ThreadCtx,
    //  Handle of the physical thread doing the I/O work.
    // thread_t _worker;
    pub _worker: ZmqIoThread<'a>,
}

impl WorkerPollerBase {
    // WorkerPollerBase (const ThreadCtx &ctx);
    // Methods from the poller concept.
    // void start (const char *name = null_mut());
    //  Checks whether the currently executing thread is the worker thread
    //  via an assertion.
    //  Should be called by the add_fd, removed_fd, set_*, reset_* functions
    //  to ensure correct usage.
    // void check_thread () const;
    //  Stops the worker thread. Should be called from the destructor of the
    //  leaf class.
    // void stop_worker ();
    //  Main worker thread routine.
    // static void worker_routine (arg_: &mut [u8]);
//
//     virtual void loop () = 0;
    pub fn new(ctx: &ThreadCtx) -> Self {
        Self {
            poller_base: Default::default(),
            ctx,
            _worker: Default::default(),
        }
    }

    pub fn stop_worker (&mut self)
    {
        self._worker.stop ();
    }

    pub fn start (&mut self, name: &str)
    {
        // zmq_assert (get_load () > 0);
        ctx.start_thread (_worker, worker_routine, this, name);
    }

    pub fn check_thread (&mut self)
    {
// #ifndef NDEBUG
        // zmq_assert (!_worker.get_started () || _worker.is_current_thread ());
// #endif
    }

    pub fn worker_routine (&mut self, arg_: &mut [u8])
    {
        // ( (arg_))->loop ();
    }
}



