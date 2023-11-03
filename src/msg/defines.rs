use std::ffi::c_void;
use libc::size_t;
use crate::metadata::ZmqMetadata;
use crate::msg::content::ZmqContent;
use crate::msg::ZmqGroup;

pub const MSG_T_SIZE: usize = 64;

pub const PING_CMD_NAME_SIZE: i32 = 5;
pub const CANCEL_CMD_NAME_SIZE: i32 = 7;
pub const SUB_CMD_NAME_SIZE: i32 = 10;

pub const MAX_VSM_SIZE: usize = MSG_T_SIZE - 3 + 16 + 4 + std::mem::size_of::<*mut ZmqMetadata>();

pub const TYPE_MIN: u8 = 101;
pub const TYPE_VSM: u8 = 101;
pub const TYPE_LMSG: u8 = 102;
pub const TYPE_DELIMITER: u8 = 103;
pub const TYPE_CMSG: u8 = 104;
pub const TYPE_ZCLMSG: u8 = 105;
pub const TYPE_JOIN: u8 = 106;
pub const TYPE_LEAVE: u8 = 107;
pub const TYPE_MAX: u8 = 107;

pub const METADATA_T_PTR_SIZE: usize = std::mem::size_of::<*mut ZmqMetadata>();
pub const CONTENT_T_PTR_SIZE: usize = std::mem::size_of::<*mut ZmqContent>();
pub const GROUP_T_SIZE: usize = std::mem::size_of::<ZmqGroup>();
pub const VOID_PTR_SIZE: usize = std::mem::size_of::<*mut c_void>();
pub const SIZE_T_SIZE: usize = std::mem::size_of::<size_t>();
