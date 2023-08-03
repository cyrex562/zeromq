use std::ptr::null_mut;
use crate::mutex::mutex_t;

pub struct dbuffer_t<T: Copy + Default>
{
    pub _storage: [T;2],
    pub _back: *mut T,
    pub _front: *mut T,
    pub _sync: mutex_t,
    pub _has_msg: bool,
}

impl <T: Copy + Default>dbuffer_t<T>{
    pub fn new() -> Self
    {
        let mut out = Self {
            _storage: [T::default();2],
            _back: null_mut(),
            _front: null_mut(),
            _sync: mutex_t::new(),
            _has_msg: false,
        };

        out._back = &mut out._storage[0];
        out._front = &mut out._storage[1];
        out
    }

    pub unsafe fn write(&mut self, value_: &mut T) {
        *self._back = value_.clone();
    }

    pub unsafe fn read(&mut self, value_: *mut T) -> bool {
        if value_ == null_mut() {
            return false;
        }

        if !self._has_msg {
            return false;
        }

        *value_ = *self._front.clone();
        self._has_msg = false;

        return true;
    }

    pub fn check_read(&mut self) -> bool {
        return self._has_msg;
    }

    pub fn probe(&mut self, func: fn(*mut T)->bool) -> bool {
        func(self._front)
    }
}