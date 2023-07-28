pub struct ypipe_base_t<T> {
    pub write: fn(value_: &T, incomplete_: bool),
    pub unwrite: fn(value_: *mut T),
    pub flush: fn(),
    pub check_read: fn(),
    pub read: fn(value: *mut T),
    pub probe: fn(&T) -> bool,
}

impl <T> ypipe_base_t<T> {
    pub fn new() -> Self {
        Self {
            write: |_, _| {},
            unwrite: |_| {},
            flush: || {},
            check_read: || {},
            read: |_| {},
            probe: |_| false,
        }
    }
}