//  Lock-free queue implementation.
//  Only a single thread can read from the pipe at any specific moment.
//  Only a single thread can write to the pipe at any specific moment.
//  T is the type of the object in the queue.
//  N is granularity of the pipe, i.e. how many items are needed to
//  perform next memory allocation.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicPtr, Ordering};
use crate::ypipe_base::YpipeBase;

// template <typename T, int N> class ypipe_t ZMQ_FINAL : public YpipeBase<T>
#[derive(Default, Debug, Clone)]
pub struct Ypipe<T>
{
    //  Allocation-efficient queue to store pipe items.
    //  Front of the queue points to the first prefetched item, back of
    //  the pipe points to last un-flushed item. Front is used only by
    //  reader thread, while back is used only by writer thread.
    // yqueue_t<T, N> _queue;
    pub _queue: VecDeque<T>,

    //  Points to the first un-flushed item. This variable is used
    //  exclusively by writer thread.
    // T *_w;
    pub _w: *mut T,

    //  Points to the first un-prefetched item. This variable is used
    //  exclusively by reader thread.
    // T *_r;
    pub _r: *mut T,

    //  Points to the first item to be flushed in the future.
    // T *_f;
    pub _f: *mut T,

    //  The single point of contention between writer and reader thread.
    //  Points past the last flushed item. If it is NULL,
    //  reader is asleep. This pointer should be always accessed using
    //  atomic operations.
    // atomic_ptr_t<T> pub _c:;
    pub _c: AtomicPtr<T>,

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (ypipe_t)
}

impl Ypipe<T> {
    //  Initialises the pipe.
//     ypipe_t ()
    pub fn new() -> Self
    {
        let mut out = Self::default();
        //  Insert terminator element into the queue.
        // _queue.push ();

        //  Let all the pointers to point to the terminator.
        //  (unless pipe is dead, in which case c is set to NULL).
        // _r = _w = _f = &_queue.back ();
        out._r = out._queue.back_mut().unwrap();
        out._w = out._queue.back_mut().unwrap();
        out._f = out._queue.back_mut().unwrap();
        out._c.store(out._queue.back_mut().unwrap(), Ordering::Relaxed);

        // _c.set (&_queue.back ());
        out
    }
}

impl YpipeBase<T> for Ypipe<T> {
    //  Write an item to the pipe.  Don't flush it yet. If incomplete is
    //  set to true the item is assumed to be continued by items
    //  subsequently written to the pipe. Incomplete items are never
    //  flushed down the stream.
    fn write(&mut self, value: &T, incomplete: bool) {
        //  Place the value to the queue, add new terminator element.
        self._queue.push_back(value);
        // self._queue.push();

        //  Move the "flush up to here" pointer.
        if !incomplete {
            self._f = self._queue.back_mut().unwrap();
        }
    }

    //  Pop an incomplete item from the pipe. Returns true if such
    //  item exists, false otherwise.
    fn unwrite(&mut self, value: &mut T) -> bool {
        if self._f == self._queue.back_mut().unwrap() {
            return false;
        }
        self._queue.unpush();
        *value = self._queue.back();
        return true;
    }

    //  Flush all the completed items into the pipe. Returns false if
    //  the reader thread is sleeping. In that case, caller is obliged to
    //  wake the reader up before using the pipe again.
    fn flush(&mut self) -> bool {
        //  If there are no un-flushed items, do nothing.
        if self._w == self._f {
            return true;
        }

        //  Try to set 'c' to 'f'.
        if self._c.cas(self._w, self._f) != self._w {
            //  Compare-and-swap was unsuccessful because 'c' is NULL.
            //  This means that the reader is asleep. Therefore we don't
            //  care about thread-safeness and update c in non-atomic
            //  manner. We'll return false to let the caller know
            //  that reader is sleeping.
            self._c.set(self._f);
            self._w = self._f;
            return false;
        }

        //  Reader is alive. Nothing special to do now. Just move
        //  the 'first un-flushed item' pointer to 'f'.
        _w = _f;
        return true;
    }

    //  Check whether item is available for reading.
    fn check_read(&mut self) -> bool {
        //  Was the value prefetched already? If so, return.
        if &self._queue.front() != self._r && self._r.is_null() == false {
            return true;
        }

        //  There's no prefetched value, so let us prefetch more values.
        //  Prefetching is to simply retrieve the
        //  pointer from c in atomic fashion. If there are no
        //  items to prefetch, set c to NULL (using compare-and-swap).
        self._r = self._c.cas(&self._queue.front(), null_mut());

        //  If there are no elements prefetched, exit.
        //  During pipe's lifetime r should never be NULL, however,
        //  it can happen during pipe shutdown when items
        //  are being deallocated.
        if self._queue.front_mut().unwrap() == self._r || !self._r.is_null() == false {
            return false;
        }

        //  There was at least one value prefetched.
        return true;
    }

    //  Reads an item from the pipe. Returns false if there is no value.
    //  available.
    fn read(&mut self, value: &mut T) -> bool {
        //  Try to prefetch a value.
        if !self.check_read() {
            return false;
        }

        //  There was at least one value prefetched.
        //  Return it to the caller.
        *value_ = self._queue.front();
        self._queue.pop();
        return true;
    }

    //  Applies the function fn to the first element in the pipe
    //  and returns the value returned by the fn.
    //  The pipe mustn't be empty or the function crashes.
    fn probe(&mut self, probe_fn: fn(&T) -> bool) -> bool {
        let rc = self.check_read();
        // zmq_assert (rc);

        return probe_fn(self._queue.front().unwrap());
    }
}

// }

// #endif
