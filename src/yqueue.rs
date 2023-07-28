use std::ffi::c_void;
use std::ptr::{null, null_mut};

pub struct chunk_t<T, const N: usize> {
    pub values: [T;N],
    pub prev: *mut c_void, // really chunk_t
    pub next: *mut c_void, // really chunk_t
}

pub struct yqueue_t<T, const N: usize>
{
    pub _begin_chunk: *mut chunk_t<T,N>,
    pub _begin_pos: usize,
    pub _back_chunk: *mut chunk_t<T,N>,
    pub _back_pos: usize,
    pub _end_chunk: *mut chunk_t<T,N>,
    pub _end_pos: usize,
    pub _spare_chunk: *mut chunk_t<T,N>,
}

impl <T, const N: usize>yqueue_t<T,N>
{
    pub fn new() -> Self
    {
        let mut out = Self {
            _begin_chunk: allocate_chunk(),
            _begin_pos: 0,
            _back_chunk: null_mut(),
            _back_pos: 0,
            _end_chunk: null_mut(),
            _end_pos: 0,
            _spare_chunk: null_mut(),
        };
        out._end_chunk = out._begin_chunk;
        out
    }

    pub unsafe fn front(&mut self) -> &T {
        &(*self._begin_chunk).values[self._begin_pos as usize]
    }

    pub unsafe fn back(&mut self) -> &T {
        &(*self._back_chunk).values[self._back_pos as usize]
    }

    pub unsafe fn back_mut(&mut self) -> &mut T {
        &mut (*self._back_chunk).values[self._back_pos as usize]
    }

    pub unsafe fn push(&mut self) {
        self._back_chunk = self._end_chunk;
        self._back_pos = self._end_pos;

        self._end_pos += 1;
        if self._end_pos != N {
            return;
        }

        let sc: *mut chunk_t<T,N> = self._spare_chunk;
        if sc != null_mut() {
            (*self._end_chunk).next = sc as *mut c_void;
            (*sc).prev = self._end_chunk as *mut c_void;
        } else {
            (*self._end_chunk).next = allocate_chunk();
           (*((* self._end_chunk).next as *mut chunk_t<T,N>)).prev = self._end_chunk as *mut c_void;
        }

        (*self)._end_chunk = (*self._end_chunk).next as *mut chunk_t<T,N>;
        self._end_pos = 0;
    }

    pub unsafe fn unpush(&mut self)
    {
        if self._back_pos != 0 {
            self._back_pos -= 1;
        }
        else {
            self._back_pos = N - 1;
            self._back_chunk = (*self._back_chunk).prev as *mut chunk_t<T,N>;
        }

        if self._end_pos != 0usize
        {
            self._end_pos -= 1;
        }
        else
        {
            self._end_pos = N - 1;
            self._end_chunk = (*self._end_chunk).prev as *mut chunk_t<T,N>;
            // TODO:
            libc::free((*self._end_chunk).next);
            (*self._end_chunk).next = null_mut();
        }
    }

    pub unsafe fn pop(&mut self)
    {
        self._begin_pos += 1;
        if self._begin_pos == N {
            let mut o: *mut chunk_t<T,N> = self._begin_chunk;
            self._begin_chunk = (*self._begin_chunk).next as *mut chunk_t<T,N>;
            (*self._begin_chunk).prev = null_mut();
            self._begin_pos = 0;
            let cs: *mut chunk_t<T,N> = self._spare_chunk;
            libc::free(cs as *mut c_void);
        }
    }

    pub unsafe fn allocate_chunk() -> *mut chunk_t<T,N> {
        let out: *mut chunk_t<T,N> = libc::malloc(std::mem::size_of::<chunk_t<T,N>>()) as *mut chunk_t<T,N>;
        (*out).prev = null_mut();
        (*out).next = null_mut();
        out
    }
}