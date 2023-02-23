
//  yqueue is an efficient queue implementation. The main goal is
//  to minimise number of allocations/deallocations needed. Thus yqueue
//  allocates/deallocates elements in batches of N.
//
//  yqueue allows one thread to use push/back function and another one
//  to use pop/front functions. However, user must ensure that there's no
//  pop on the empty queue and that both threads don't access the same
//  element in unsynchronised manner.
//
//  T is the type of the object in the queue.
//  N is granularity of the queue (how many pushes have to be done till
//  actual memory allocation is required).
// #if defined HAVE_POSIX_MEMALIGN
// ALIGN is the memory alignment size to use in the case where we have
// posix_memalign available. Default value is 64, this alignment will
// prevent two queue chunks from occupying the same CPU cache line on
// architectures where cache lines are <= 64 bytes (e.g. most things
// except POWER). It is detected at build time to try to account for other
// platforms like POWER and s390x.
// template <typename T, N: i32, size_t ALIGN = ZMQ_CACHELINE_SIZE> class yqueue_t
// #else
// template <typename T, int N> class yqueue_t
// #endif

use std::ptr::null_mut;
use std::sync::atomic::AtomicPtr;

#[derive(Default,Debug,Clone)]
pub struct chunk_t<T>
{
    // pub values: [T;N],
    pub values: Vec<T>,
    // chunk_t *prev;
    // chunk_t *next;
};

#[derive(Default,Debug,Clone)]
pub struct yqueue_t<T>
{
    //  Back position may point to invalid memory if the queue is empty,
    //  while begin & end positions are always valid. Begin position is
    //  accessed exclusively be queue reader (front/pop), while back and
    //  end positions are accessed exclusively by queue writer (back/push).
    // chunk_t *_begin_chunk;
    pub _begin_chunk: *mut chunk_t<T>,
    // _begin_pos: i32;
    pub _begin_pos: i32,
    // chunk_t *_back_chunk;
    pub _back_chunk: *mut chunk_t<T>,
    // _back_pos: i32;
    pub _back_pos: i32,
    // chunk_t *_end_chunk;
    pub _end_chunk: *mut chunk_t<T>,
    pub _end_pos: i32,

    //  People are likely to produce and consume at similar rates.  In
    //  this scenario holding onto the most recently freed chunk saves
    //  us from having to call malloc/free.
    // atomic_ptr_t<chunk_t> _spare_chunk;
    pub _spare_chunk: AtomicPtr<chunk_t<T>>,

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (yqueue_t)
}

impl yqueue_t<T> {
    // public:
    //  Create the queue.
    pub fn new() -> Self
    {
        let mut out = Self::default();

        out._begin_chunk = allocate_chunk();
        // alloc_assert (_begin_chunk);
        out._begin_pos = 0;
        out._back_chunk = null_mut();
        out._back_pos = 0;
        out._end_chunk = out._begin_chunk;
        out._end_pos = 0;
        out
    }

    //  Destroy the queue.
    // inline ~yqueue_t ()
    // {
    // while (true) {
    // if (_begin_chunk == _end_chunk) {
    // free (_begin_chunk);
    // break;
    // }
    // chunk_t *o = _begin_chunk;
    // _begin_chunk = _begin_chunk->next;
    // free (o);
    // }
    //
    // chunk_t *sc = _spare_chunk.xchg (NULL);
    // free (sc);
    // }

    //  Returns reference to the front element of the queue.
    //  If the queue is empty, behaviour is undefined.
    // inline T &front () { return _begin_chunk->values[_begin_pos]; }
    pub fn front(&mut self) -> &mut T {
        self._begin_chunk.values.get_mut(self._begin_pos).unwrap()
    }

    //  Returns reference to the back element of the queue.
    //  If the queue is empty, behaviour is undefined.
    // inline T &back () { return _back_chunk->values[_back_pos]; }
    pub fn back(&mut self) -> &mut T {
        self._back_chunk.values.get_mut(self._back_pos).unwrap()
    }

    //  Adds an element to the back end of the queue.
    pub fn push (&mut self)
    {
    self._back_chunk = self._end_chunk;
    self._back_pos = self._end_pos;

    //     self._end_pos += 1;
    // if (end_pos != N){
    //     return;
    // }

    let sc = self._spare_chunk.xchg (null_mut());
    if (sc.is_null_mut() == false) {
    self._end_chunk.next = sc;
    sc.prev = self._end_chunk;
    } else {
    // self._end_chunk.next = allocate_chunk ();
    // alloc_assert (_end_chunk->next);
    // _end_chunk->next->prev = _end_chunk;
    }
    // _end_chunk = _end_chunk->next;
    // _end_pos = 0;
    }

    //  Removes element from the back end of the queue. In other words
    //  it rollbacks last push to the queue. Take care: Caller is
    //  responsible for destroying the object being unpushed.
    //  The caller must also guarantee that the queue isn't empty when
    //  unpush is called. It cannot be done automatically as the read
    //  side of the queue can be managed by different, completely
    //  unsynchronised thread.
    pub fn unpush (&mut self)
    {
    //  First, move 'back' one position backwards.
    if (self._back_pos) {
        self._back_pos -= 1;
    }
    // else {
    // self._back_pos = N - 1;
    // _back_chunk = _back_chunk->prev;
    // }

    //  Now, move 'end' position backwards. Note that obsolete end chunk
    //  is not used as a spare chunk. The analysis shows that doing so
    //  would require free and atomic operation per chunk deallocated
    //  instead of a simple free.
    if self._end_pos {
        self._end_pos -= 1;
    }
    // else {
    // self._end_pos = N - 1;
    // _end_chunk = _end_chunk->prev;
    // free (_end_chunk->next);
    // _end_chunk->next = NULL;
    // }
    }

    //  Removes an element from the front end of the queue.
    pub fn pop (&mut self)
    {
    if (++_begin_pos == N) {
    chunk_t *o = _begin_chunk;
    _begin_chunk = _begin_chunk->next;
    _begin_chunk->prev = NULL;
    _begin_pos = 0;

    //  'o' has been more recently used than _spare_chunk,
    //  so for cache reasons we'll get rid of the spare and
    //  use 'o' as the spare.
    chunk_t *cs = _spare_chunk.xchg (o);
    free (cs);
    }
    }

    static inline chunk_t *allocate_chunk ()
    {
    // #if defined HAVE_POSIX_MEMALIGN
    pv: *mut c_void;
    if (posix_memalign (&pv, ALIGN, mem::size_of::<chunk_t>()) == 0)
    return (chunk_t *) pv;
    return NULL;
// #else
    return static_cast<chunk_t *> (malloc (mem::size_of::<chunk_t>()));
// #endif
    }
}


// #endif
