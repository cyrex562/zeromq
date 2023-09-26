use std::collections::HashMap;
use crate::defines::{fd_t, WSAEVENT};
use crate::i_poll_events::i_poll_events;

pub const fd_family_cache_size: usize = 8;

// typedef struct fd_set {
//   u_int  fd_count;
//   SOCKET fd_array[FD_SETSIZE];
// } fd_set, FD_SET, *PFD_SET, *LPFD_SET;
pub struct fd_set {
    pub fd_count: u32,
    pub fd_array: [fd_t; 64],
}


pub struct fds_set_t {
    pub read: fd_set,
    pub write: fd_set,
    pub error: fd_set,
}

pub struct fd_entry_t
{
    pub fd: fd_t,
    pub events: *mut dyn i_poll_events,
}

pub type fd_entries_t = Vec<fd_entry_t>;

pub struct family_entry_t
{
    pub fd_entries: fd_entries_t,
    pub fds_set: fds_set_t,
pub has_retired: bool,
}

#[cfg(target_os="windows")]
pub type family_entries_t = HashMap<u16,family_entry_t>;

#[cfg(target_os="windows")]
pub struct wsa_events_t {
    pub events: [WSAEVENT; 4],
}

pub struct select_t {
    #[cfg(target_os="windows")]
    pub _family_entries: family_entries_t,
    #[cfg(target_os="windows")]
    pub _fd_family_cache: [(fd_t, u16);fd_family_cache_size],
    #[cfg(not(target_os="windows"))]
    pub _family_entry: family_entry_t,
    #[cfg(not(target_os="windows"))]
    pub _max_fd: fd_t,
}

// #[cfg(target_feature="select")]
