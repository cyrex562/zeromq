use std::alloc::{dealloc, Layout};
use std::ffi::c_void;
use std::mem;
use std::ptr::null_mut;

use crate::defines::atomic_counter::ZmqAtomicCounter;
use crate::msg::content::ZmqContent;
use crate::msg::defines::MAX_VSM_SIZE;

pub trait ZmqAllocator {
    unsafe fn allocate(&mut self) -> &mut [u8];
    unsafe fn deallocate(&mut self);
    unsafe fn size(&self) -> usize;
    unsafe fn resize(&mut self, new_size_: usize);
}

pub struct ZmqCSingleAllocator {
    pub _buf_size: usize,
    pub _buf: Vec<u8>,
}

impl ZmqAllocator for ZmqCSingleAllocator {
    unsafe fn allocate(&mut self) -> &mut [u8] {
        self._buf.as_mut_slice()
    }

    unsafe fn deallocate(&mut self) {
        unimplemented!()
    }

    unsafe fn size(&self) -> usize {
        self._buf_size
    }

    unsafe fn resize(&mut self, new_size_: usize) {
        unimplemented!()
    }
}

impl ZmqCSingleAllocator {
    pub unsafe fn new(buf_size_: usize) -> Self {
        Self {
            _buf_size: buf_size_,
            _buf: vec![],
        }
    }
}

pub struct ZmqSharedMessageMemoryAllocator<'a> {
    pub _buf: Vec<u8>,
    pub _buf_size: usize,
    pub _max_size: usize,
    pub _msg_content: &'a mut ZmqContent,
    pub _max_counters: usize,
}

impl<'a> ZmqAllocator for ZmqSharedMessageMemoryAllocator<'a> {
    unsafe fn allocate(&mut self) -> &mut [u8] {
        if self._buf != null_mut() {
            let c = self._buf.as_mut_ptr() as *mut ZmqAtomicCounter;
            (*c).sub(1);
            if (*c).get() != 0 {
                self.release();
            }
        }

        if self._buf == null_mut() {
            let allocationsize = self._max_size + mem::size_of::<ZmqAtomicCounter>() + self._max_counters * mem::size_of::<ZmqContent>;
            self._buf = vec![];
            // new _buf atomoic_counter_t (1)
        } else {
            let c = self._buf.as_mut_ptr() as *mut ZmqAtomicCounter;
            (*c).set(1);
        }

        self._buf_size = self._max_size;
        // TODO
        // self._msg_content = self._buf.add(self._max_size + mem::size_of::<ZmqAtomicCounter>()) as *mut ZmqContent;
        self._buf.add(mem::size_of::<ZmqAtomicCounter>())
    }

    unsafe fn deallocate(&mut self) {
        let c = self._buf.as_mut_ptr() as *mut ZmqAtomicCounter;
        (*c).sub(1);
        if (*c).get() == 0 {
            dealloc(self._buf.as_mut_ptr(), Layout::from_size_align_unchecked(self._max_size + mem::size_of::<ZmqAtomicCounter>() + self._max_counters * mem::size_of::<ZmqContent>(), mem::align_of::<u8>()));
        }
    }

    unsafe fn size(&mut self) -> usize {
        self._buf_size
    }

    unsafe fn resize(&mut self, new_size_: usize) {
        self._buf_size = new_size_;
    }
}

impl<'a> ZmqSharedMessageMemoryAllocator<'a> {
    pub unsafe fn new(bufsize_: usize, max_messages_: usize) -> Self {
        Self {
            _buf: vec![],
            _buf_size: 0,
            _max_size: bufsize_,
            _msg_content: &mut ZmqContent::default(),
            _max_counters: max_messages_,
        }
    }

    pub unsafe fn new2(bufsize_: usize) -> Self {
        let mut out = Self {
            _buf: vec![],
            _buf_size: 0,
            _max_size: bufsize_,
            _msg_content: &mut ZmqContent::default(),
            _max_counters: 0,
        };

        out._max_counters = ((out._max_size + MAX_VSM_SIZE - 1) / MAX_VSM_SIZE);
        out
    }


    pub unsafe fn release(&mut self) -> &mut [u8] {
        let mut b = &mut self._buf;
        self.clear();
        b.as_mut_slice()
    }

    pub unsafe fn clear(&mut self) {
        self._buf.clear();
        self._buf_size = 0;
        self._msg_content = &mut ZmqContent::default();
    }

    pub unsafe fn inc_ref(&mut self) {
        let c = self._buf.as_mut_ptr() as *mut ZmqAtomicCounter;
        (*c).add(1);
    }

    pub unsafe fn call_dec_ref(&mut self, x: *mut c_void, hint_: *mut c_void) {
        let mut buf = hint_ as *mut u8;
        let c = buf as *mut ZmqAtomicCounter;
        (*c).sub(1);
        if (*c).get() == 0 {
            dealloc(buf, Layout::from_size_align_unchecked(self._max_size + mem::size_of::<ZmqAtomicCounter>() + self._max_counters * mem::size_of::<ZmqContent>(), mem::align_of::<u8>()));
            buf = null_mut();
        }
    }


    pub unsafe fn data(&mut self) -> *mut u8 {
        self._buf.add(mem::size_of::<ZmqAtomicCounter>())
    }

    pub unsafe fn buffer(&mut self) -> &mut [u8] {
        self._buf.as_mut_slice()
    }

    pub unsafe fn provide_content(&mut self) -> *mut ZmqContent {
        unimplemented!()
    }

    pub unsafe fn advance_content(&mut self) {
        self._msg_content = self._msg_content.add(1);
    }
}
