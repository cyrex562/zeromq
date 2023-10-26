use std::ops::{Index, IndexMut};
use std::ptr;
use std::ptr::null_mut;

pub struct ArrayItem<const I: i32> {
    pub _array_index: i32
}

impl <const I: i32> ArrayItem<I>
{
    pub fn new() -> Self {
        Self {
        _array_index: -1
        }
    }

    pub fn set_array_index(&mut self, index_: i32) {
        self._array_index = index_;
    }

    pub fn get_array_index(&mut self) -> i32 {
        self._array_index
    }
}

#[derive(Default,Debug,Clone)]
pub struct ZmqArray<T, const I: i32> {
    pub items: Vec<T>
}

impl <T, const I: i32> ZmqArray<T,I> {
    pub fn size(&self) -> usize {
        self.items.len()
    }

    pub fn empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn push_back(&mut self, item_: T) {
        if item_ != ptr::null_mut() {
            self.items.push(item_);
        }
    }

    pub fn erase(&mut self, item_: &T) {
        if item_ != ptr::null_mut() {
            let mut index = 0;
            for i in &self.items {
                if *i == item_ {
                    self.items.remove(index);
                    break;
                }
                index += 1;
            }
        }
    }

    pub fn erase2(&mut self, index_: usize) {
        if index_ < self.items.len() {
            self.items.remove(index_);
        }
    }

    pub fn swap(&mut self, index1_: usize, index2_: usize) {
        if index1_ < self.items.len() && index2_ < self.items.len() {
            let temp = self.items[index1_];
            self.items[index1_] = self.items[index2_];
            self.items[index2_] = temp;
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn index(&self, item_: &T) -> Option<usize> {
        if item_ != null_mut() {
            let mut index = 0;
            for i in &self.items {
                if *i == item_ {
                    return Some(index);
                }
                index += 1;
            }
        }

        None
    }
}

impl <T, const I: i32> Index<usize>  for ZmqArray<T, I> {
    type Output = *mut T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

impl <T, const I: i32> IndexMut<usize> for ZmqArray<T,I> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.items[index]
    }
}
