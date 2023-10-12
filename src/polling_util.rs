use std::cmp;
use std::mem::size_of_val;
use crate::select::fd_set;

pub type fast_vector_t<T> = Vec<T>;

pub type resizable_fast_vector_t<T> = Vec<T>;

pub type timeout_t = i32;

pub fn compute_timeout(first_pass_: bool, timeout_: i32, now_: u64, end_: u64) -> timeout_t {
    if first_pass_ {return 0;}

    if timeout_ < 0 {return -1;}

    cmp::min((end_ - now_) as timeout_t, i32::MAX)
}

pub fn valid_pollset_bytes(pollset_: &fd_set)
{
    size_of_val(pollset_);
}

pub type optimized_fd_set = fd_set;

pub type resizable_optimized_fd_set_t = fd_set;
