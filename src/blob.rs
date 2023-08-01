use std::cmp::Ordering;
use libc::size_t;

pub struct reference_tag_t {

}

pub struct blob_t {
    pub _data: *mut u8,
    pub _size: usize,
    pub _owned: bool,
}

impl blob_t {
    pub fn new() -> Self {
        blob_t {
            _data: std::ptr::null_mut(),
            _size: 0,
            _owned: false,
        }
    }

    pub fn new2(size_: size_t) -> Self {
        let mut out = blob_t {
            _data: std::ptr::null_mut(),
            _size: size_,
            _owned: false,
        };
        out._data = unsafe { libc::malloc(size_) as *mut u8 };
        out
    }

    pub fn new3(data_: *mut u8, size_: size_t) -> Self {
        blob_t {
            _data: data_,
            _size: size_,
            _owned: false,
        }
    }

    pub fn size(&self) -> usize {
        self._size
    }

    pub fn data_mut(&self) -> *mut u8 {
        self._data
    }

    pub fn data(&self) -> *const u8 {
        self._data
    }

    pub fn set_deep_copy(&mut self, other_: &Self)
    {
        self.clear();
        self._data = unsafe { libc::malloc(other_._size) as *mut u8 };
        self._size = other_._size;
        self._owned = true;
        if self._size != 0 && self._data != std::ptr::null_mut() {
            unsafe {
                std::ptr::copy_nonoverlapping(other_._data, self._data, self._size);
            }
        }
    }

    pub fn set(&mut self, data_: *mut u8, size_: size_t) {
        self.clear();
        // self._data = data_;
        self._data = unsafe { libc::malloc(size_) as *mut u8 };
        self._size = size_;
        self._owned = false;
        if self._size != 0 && self._data != std::ptr::null_mut() {
            unsafe {
                std::ptr::copy_nonoverlapping(data_, self._data, self._size);
            }
        }
    }

    pub fn clear(&mut self) {
        if self._owned && self._data != std::ptr::null_mut() {
            unsafe {
                libc::free(self._data as *mut libc::c_void);
            }
        }
        self._data = std::ptr::null_mut();
        self._size = 0;
        self._owned = false;
    }
}

impl PartialEq<Self> for blob_t {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }

    fn ne(&self, other: &Self) -> bool {
        todo!()
    }
}

impl Eq for blob_t {

}

impl PartialOrd<Self> for blob_t {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        todo!()
    }

    fn ge(&self, other: &Self) -> bool {
        todo!()
    }

    fn gt(&self, other: &Self) -> bool {
        todo!()
    }

    fn le(&self, other: &Self) -> bool {
        todo!()
    }

    fn lt(&self, other: &Self) -> bool {
        todo!()
    }
}

impl Ord for blob_t {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        todo!()
    }

    fn max(self, other: Self) -> Self where Self: Sized {
        todo!()
    }

    fn min(self, other: Self) -> Self where Self: Sized {
        todo!()
    }

    fn clamp(self, min: Self, max: Self) -> Self where Self: Sized {
        todo!()
    }
}
