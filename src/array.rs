pub struct array_item_t {
    pub array_index: i32,
}

impl array_item_t {
    pub fn new() -> array_item_t {
        array_item_t {}
    }

    pub fn set_array_index(index_: i32) {
        self.array_index = index_;
    }

    pub fn get_array_index() -> i32 {
        return self.array_index;
    }
}

pub type item_t = array_item_t;

pub struct array_t {
    pub items: Vec<item_t>,
}

impl array_t {
    pub fn new() -> array_t {
        array_t {}
    }

    pub fn size() -> i32 {
        return self.items.len();
    }

    pub fn push_back(item: *mut item_t) {
        self.items.push(item);
    }

    pub fn erase(item: *mut item_t) {
        let index = item.get_array_index();
        self.items.remove(index as usize);
    }

    pub fn erase(index: i32) {
        self.items.remove(index as usize);
    }

    pub fn swap(index1: i32, index2: i32) {
        self.items.swap(index1 as usize, index2 as usize);
    }

    pub fn clear() {
        self.items.clear();
    }

    pub fn index(item: *mut item_t) -> i32 {
        return item.get_array_index();
    }
}

impl Index<usize> for array_t {
    type Output = item_t;

    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

impl IndexMut<usize> for array_t {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.items[index]
    }
}
