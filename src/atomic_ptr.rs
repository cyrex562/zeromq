pub fn atomic_xchg_ptr<T>(ptr: &AtomicPtr<T>, new_ptr: *mut T) -> *mut T {
    ptr.swap(new_ptr, Ordering::SeqCst)
}

pub fn atomic_cas<T>(ptr: &AtomicPtr<T>, old_ptr: *mut T, new_ptr: *mut T) -> bool {
    ptr.compare_and_swap(old_ptr, new_ptr, Ordering::SeqCst) == old_ptr
}

pub struct atomic_ptr_t<T> {
    pub ptr: AtomicPtr<T>,
}