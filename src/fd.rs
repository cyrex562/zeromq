#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

use crate::defines::zmq_fd_t;

pub type fd_t = zmq_fd_t;

pub const retired_fd: i32 = -1;
