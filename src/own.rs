#![allow(non_camel_case_types)]

use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use crate::atomic_counter::atomic_counter_t;
use crate::object::object_t;
use crate::options::options_t;

pub struct own_t
{
    pub object: object_t,
    pub options: options_t,
    pub _terminating: bool,
    pub _sent_seqnum: atomic_counter_t,
    pub _processed_seqnum: u64,
    pub _owner: *mut c_void, // really own_t
    pub _owned: HashSet<*mut c_void>,
    pub _term_acks: i32,
}