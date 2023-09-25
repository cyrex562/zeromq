#![allow(non_camel_case_types)]

use crate::defines::handle_t;
use crate::i_poll_events::i_poll_events;
use crate::mailbox::mailbox_t;
use crate::object::object_t;

pub struct io_thread_t
{
    pub object: object_t,
    pub poll_events: i_poll_events,
    pub _mailbox: mailbox_t,
    pub _mailbox_handle: handle_t,
    pub _poller: *mut poller_t,
}
