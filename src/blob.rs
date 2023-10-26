use std::cmp::Ordering;
use libc::size_t;

pub struct ZmqReferenceTag {

}

pub struct ZmqBlob {
    pub data: Vec<u8>,
    pub size: usize,
    pub owned: bool,
}

impl ZmqBlob {
    pub fn new() -> Self {
        ZmqBlob {
            data: Vec::new(),
            size: 0,
            owned: false,
        }
    }

    pub fn new2(size_: size_t) -> Self {
        let mut out = ZmqBlob {
            data: Vec::with_capacity(size_ as usize),
            size: size_,
            owned: false,
        };
        // out.data = unsafe { libc::malloc(size_) as *mut u8 };
        out
    }

    pub fn new3(data_: &[u8], size_: size_t) -> Self {
        let mut out = ZmqBlob {
            data: vec![],
            size: size_,
            owned: false,
        };
        out.data.extend_from_slice(data_);
        out
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn data_mut(&mut self) ->&mut [u8] { self.data.as_mut_slice()
    }

    pub fn data(&self) -> & [u8] {
        self.data.as_slice()
    }

    pub fn set_deep_copy(&mut self, other_: &Self)
    {
        self.clear();
        // self.data = unsafe { libc::malloc(other_.size) as *mut u8 };
        self.data = Vec::with_capacity(other_.size);
        self.size = other_.size;
        self.owned = true;
        if self.size != 0 && self.data != std::ptr::null_mut() {
            unsafe {
                std::ptr::copy_nonoverlapping(other_.data, self.data, self.size);
            }
        }
    }

    pub fn set(&mut self, data_: &mut [u8], size_: size_t) {
        self.clear();
        // self._data = data_;
        // self.data = unsafe { libc::malloc(size_) as *mut u8 };
        self.data = Vec::with_capacity(size_ as usize);
        self.size = size_;
        self.owned = false;
        if self.size != 0 && self.data != std::ptr::null_mut() {
            unsafe {
                std::ptr::copy_nonoverlapping(data_, self.data, self.size);
            }
        }
    }

    pub fn clear(&mut self) {
        // if self.owned && self.data != std::ptr::null_mut() {
        //     unsafe {
        //         libc::free(self.data as *mut libc::c_void);
        //     }
        // }
        // self.data = std::ptr::null_mut();
        self.data.clear();
        self.size = 0;
        self.owned = false;
    }
}

impl PartialEq<Self> for ZmqBlob {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }

    fn ne(&self, other: &Self) -> bool {
        todo!()
    }
}

impl Eq for ZmqBlob {

}

impl PartialOrd<Self> for ZmqBlob {
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

impl Ord for ZmqBlob {
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
