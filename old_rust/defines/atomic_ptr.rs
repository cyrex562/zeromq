use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};

pub unsafe fn atomic_xchg_ptr(ptr: *mut *mut c_void, newval: *mut c_void) -> *mut c_void {
    let some_ptr = AtomicPtr::new(*ptr);
    let x = some_ptr.load(Ordering::SeqCst);
    some_ptr.store(newval, Ordering::SeqCst);
    x
}

pub unsafe fn atomic_cas_ptr(ptr: *mut *mut c_void, oldval: *mut c_void, newval: *mut c_void) -> bool {
    let some_ptr = AtomicPtr::new(*ptr);
    let x = some_ptr.compare_exchange(oldval, newval, Ordering::SeqCst, Ordering::SeqCst);
    x.is_ok()
}

pub struct ZmqAtomicPtr<T> {
    pub ptr: AtomicPtr<T>,
}

impl<T> ZmqAtomicPtr<T> {
    pub fn new() -> Self {
        ZmqAtomicPtr {
            ptr: AtomicPtr::new(std::ptr::null_mut())
        }
    }

    pub fn xchg(&self, newval: *mut T) -> *mut T {
        unsafe {
            let x = self.ptr.load(Ordering::SeqCst);
            self.ptr.store(newval, Ordering::SeqCst);
            x
        }
    }

    pub fn get(&self) -> *mut T {
        unsafe {
            self.ptr.load(Ordering::SeqCst)
        }
    }

    pub fn set(&self, ptr: *mut T) {
        unsafe {
            self.ptr.store(ptr, Ordering::SeqCst);
        }
    }

    pub fn cas(&self, oldval: *mut T, newval: *mut T) -> bool {
        unsafe {
            self.ptr.compare_exchange(oldval, newval, Ordering::SeqCst, Ordering::SeqCst).is_ok()
        }
    }
}
