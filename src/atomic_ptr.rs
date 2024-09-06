pub fn atomic_xchg_ptr<T>(ptr: &AtomicPtr<T>, new_ptr: *mut T) -> *mut T {
    ptr.swap(new_ptr, Ordering::SeqCst)
}

pub fn atomic_cas<T>(ptr: &AtomicPtr<T>, old_ptr: *mut T, new_ptr: *mut T) -> bool {
    ptr.compare_and_swap(old_ptr, new_ptr, Ordering::SeqCst) == old_ptr
}

pub struct atomic_ptr_t<T> {
    pub ptr: AtomicPtr<T>,
}

impl atomic_ptr_t {
    pub fn set(&mut self, ptr: *mut T) {
        self.ptr.store(ptr, Ordering::SeqCst);
    }

    pub fn xchg(&mut self, val: *mut T) -> *mut T {
        self.ptr.compare_exchange(val, Ordering::SeqCst)
    }

    pub fn cas(&mut self, cmp: *mut T, val: *mut T) -> *mut T {
        self.ptr.compare_exchange(cmp, val, Ordering::Acquire)
    }
}

pub struct atomic_value_t {
    pub value: AtomicI32,
}

impl atomic_value_t {
    pub fn new(value: i32) -> Self {
        Self {
            value: AtomicI32::new(value),
        }
    }

    pub fn load() -> i32 {
        self.value.load(Ordering::Acquire)
    }
}
