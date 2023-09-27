use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::ffi::c_void;
use std::mem;
use std::ptr::null_mut;
use crate::atomic_counter::atomic_counter_t;
use crate::msg::{content_t, max_vsm_size};

pub struct c_single_allocator
{
    pub _buf_size: usize,
    pub _buf: *mut u8,
}

impl c_single_allocator {
    pub unsafe fn new(buf_size_: usize) -> Self {
        Self {
            _buf_size: buf_size_,
            _buf: alloc_zeroed(Layout::from_size_align_unchecked(buf_size_, mem::align_of::<u8>())),
        }
    }

    pub unsafe fn allocate(&mut self) -> *mut u8 {
        self._buf
    }

    pub unsafe fn deallocate() {
        unimplemented!()
    }

    pub unsafe fn size(&self) -> usize {
        self._buf_size
    }

    pub unsafe fn resize(&mut self, new_size_: usize) {
        unimplemented!()
    }
}

pub struct shared_message_memory_allocator
{
    pub _buf: *mut u8,
    pub _buf_size: usize,
    pub _max_size: usize,
    pub _msg_content: *mut content_t,
    pub _max_counters: usize,
}

impl shared_message_memory_allocator {
    pub unsafe fn new(bufsize_: usize, max_messages_: usize) -> Self
    {
        Self {
            _buf: null_mut(),
            _buf_size: 0,
            _max_size: bufsize_,
            _msg_content: null_mut(),
            _max_counters: max_messages_,
        }
    }

    pub unsafe fn new2(bufsize_: usize) -> Self
    {
        let mut out = Self {
            _buf: null_mut(),
            _buf_size: 0,
            _max_size: bufsize_,
            _msg_content: null_mut(),
            _max_counters: 0,
        };

        out._max_counters = ((out._max_size + max_vsm_size - 1) / max_vsm_size);
        out
    }

    pub unsafe fn allocate(&mut self) -> *mut u8 {
        if self._buf != null_mut() {
            let c = self._buf as *mut atomic_counter_t;
            (*c).sub(1);
            if (*c).get() != 0 {
                self.release();
            }
        }

        if self._buf == null_mut() {
            let allocationsize = self._max_size + mem::size_of::<atomic_counter_t>() + self._max_counters * mem::size_of::<content_t>;
            self._buf = alloc_zeroed(Layout::from_size_align_unchecked(allocationsize, mem::align_of::<u8>()));
            // new _buf atomoic_counter_t (1)
        } else {
            let c = self._buf as *mut atomic_counter_t;
            (*c).set(1);
        }

        self._buf_size = self._max_size;
        self._msg_content = self._buf.add(self._max_size + mem::size_of::<atomic_counter_t>()) as *mut content_t;
        self._buf.add(mem::size_of::<atomic_counter_t>())
    }

    pub unsafe fn deallocate(&mut self) {
        let c = self._buf as *mut atomic_counter_t;
        (*c).sub(1);
        if (*c).get() == 0 {
            dealloc(self._buf, Layout::from_size_align_unchecked(self._max_size + mem::size_of::<atomic_counter_t>() + self._max_counters * mem::size_of::<content_t>(), mem::align_of::<u8>()));
        }
    }

    pub unsafe fn release(&mut self) -> *mut u8 {
        let b = self._buf;
        self.clear();
        b
    }

    pub unsafe fn clear(&mut self) {
        self._buf = null_mut();
        self._buf_size = 0;
        self._msg_content = null_mut();
    }

    pub unsafe fn inc_ref(&mut self) {
        let c = self._buf as *mut atomic_counter_t;
        (*c).add(1);
    }

    pub unsafe fn call_dec_ref(&mut self, x: *mut c_void, hint_: *mut c_void) {
        let mut buf = hint_ as *mut u8;
        let c = buf as *mut atomic_counter_t;
        (*c).sub(1);
        if (*c).get() == 0 {
            dealloc(buf, Layout::from_size_align_unchecked(self._max_size + mem::size_of::<atomic_counter_t>() + self._max_counters * mem::size_of::<content_t>(), mem::align_of::<u8>()));
            buf = null_mut();
        }
    }

    pub unsafe fn size(&mut self) -> usize {
        self._buf_size
    }

    pub unsafe fn resize(&mut self, new_size_: usize) {
        self._buf_size = new_size_;
    }

    pub unsafe fn data(&mut self) -> *mut u8 {
        self._buf.add(mem::size_of::<atomic_counter_t>())
    }

    pub unsafe fn buffer(&mut self) -> *mut u8 {
        self._buf
    }

    pub unsafe fn provide_content(&mut self) -> *mut content_t {
        unimplemented!()
    }

    pub unsafe fn advance_content(&mut self) {
        self._msg_content = self._msg_content.add(1);
    }
}
