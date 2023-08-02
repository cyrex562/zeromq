use std::ffi::c_void;
use libc::size_t;
use crate::atomic_counter::atomic_counter_t;
use crate::defines::ZMQ_GROUP_MAX_LENGTH;

pub const cancel_cmd_name: &'static str = "\x06CANCEL";
pub const sub_cmd_name: &'static str = "\x09SUBSCRIBE";


pub struct content_t
{
    pub data: *mut c_void,
    pub size: size_t,
    pub hint: *mut c_void,
    pub refcnt: atomic_counter_t,
    pub msg_free_fn: fn(*mut c_void, *mut c_void),
}

pub const more: i32 = 1;
pub const command: i32 = 2;
pub const ping: i32 = 4;
pub const pong: i32 = 8;
pub const subscribe: i32 = 12;
pub const cancel: i32 = 16;
pub const close_cmd: i32 = 20;
pub const credential: i32 = 32;
pub const routing_id: i32 = 64;
pub const shared: i32 = 128;

pub const msg_t_size: usize = 64;

pub const ping_cmd_name_size: i32 = 5;
pub const cancel_cmd_name_size: i32 = 7;
pub const sub_cmd_name_size: i32 = 10;

pub const max_vsm_size: usize = msg_t_size - 3 + 16 + 4 + std::mem::size_of::<*mut metadata_t>();

pub const type_min: i32 = 101;
pub const type_vsm: i32 = 101;
pub const type_lmsg: i32 = 102;
pub const type_delimiter: i32 = 103;
pub const type_cmsg: i32 = 104;
pub const type_zclmsg: i32 = 105;
pub const type_join: i32 = 106;
pub const type_leave: i32 = 107;
pub const type_max: i32 = 107;

pub const metadata_t_ptr_size: usize = std::mem::size_of::<*mut metadata_t>();
pub const content_t_ptr_size: usize = std::mem::size_of::<*mut content_t>();
pub const group_t_size: usize = std::mem::size_of::<group_t>();


pub enum group_type_t{
    group_type_short,
    group_type_long,
}

pub struct long_group_t{
    pub group: [u8;ZMQ_GROUP_MAX_LENGTH + 1],
    pub refcnt: atomic_counter_t,
}

pub struct sgroup
{
    pub type_: u8,
    pub group: [u8;15],
}

pub struct lgroup {
    pub type_: u8,
    pub content: *mut long_group_t,
}

pub struct base{
    pub metadata: *mut metadata_t,
    pub unused: [u8;std::mem::size_of<metadata_t_ptr_size + 2 + 4 + group_t_size],
}

pub struct vsm {
    pub metadata: *mut metadata_t,
    pub data: [u8;max_vsm_size],
    pub size: u8,
    pub type_: u8,
    pub flags: u8,
    pub routing_id: u32,
    pub group: group_t,
}

pub struct lmsg {
    pub metadata: *mut metadata_t,
    pub content: *mut content_t,
    pub unused: [u8;msg_t_size - metadata_t_ptr_size + content_t_ptr_size + 2 + 4 + group_t_size],
}

pub union msg_u {
    pub base: base,
    pub vsm: vsm,
}

pub union group_t {
    pub type_: u8,
    pub sgroup: sgroup,
    pub lgroup: lgroup,
}

pub struct msg_t
{
    pub refcnt: *mut atomic_counter_t,

}