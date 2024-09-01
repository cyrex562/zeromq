use std::cmp;
use std::mem::size_of_val;
use crate::poll::select::ZmqFdSet;

pub type ZmqFastVector<T> = Vec<T>;

pub type ZmqResizableFastVector<T> = Vec<T>;

pub type ZmqTimeout = i32;

pub fn compute_timeout(first_pass_: bool, timeout_: i32, now_: u64, end_: u64) -> ZmqTimeout {
    if first_pass_ {return 0;}

    if timeout_ < 0 {return -1;}

    cmp::min((end_ - now_) as ZmqTimeout, i32::MAX)
}

pub fn valid_pollset_bytes(pollset_: &ZmqFdSet) -> usize {
    size_of_val(pollset_)
}

pub type OptimizedFdSet = ZmqFdSet;

pub type ResizableOptimizedFdSetT = ZmqFdSet;
