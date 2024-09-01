use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use std::time::Duration;

use chrono::Utc;
use windows::Win32::Networking::WinSock::{FD_ACCEPT, FD_CLOSE, FD_CONNECT, FD_READ, FD_WRITE, WSAEventSelect, WSAWaitForMultipleEvents};
#[cfg(target_os = "windows")]
use windows::Win32::Networking::WinSock::WSAPoll;

use crate::ctx::ZmqContext;
use crate::defines::{AF_UNSPEC, RETIRED_FD, ZmqFd, ZmqHandle, ZmqPollFd};
use crate::defines::err::ZmqError;
use crate::defines::time::ZmqTimeval;
use crate::io::thread::ZmqThread;
use crate::platform::{platform_poll, platform_select};
use crate::poll::poller_event::ZmqPollerEvent;
use crate::poll::select::{FD_FAMILY_CACHE_SIZE, ZmqFdSet};
use crate::utils::{FD_ISSET, is_retired_fd};

pub mod select;
pub mod socket_poller;

pub mod poller_base;
pub mod poller_event;
pub mod polling_util;
pub mod pollitem;


pub fn FD_ZERO(fdset: *mut ZmqFdSet) {
    unimplemented!()
}

#[allow(non_snake_case)]
pub fn FD_SET(fd: ZmqFd, fds: &mut ZmqFdSet) {
    let mut i = 0;
    while i < fds.fd_array.len() {// Returns a non-zero value if the bit for the file descriptor fd is set in the file descriptor set by fdset, and 0 otherwise.
// void FD_SET(int fd, fd_set *fdset)
//
// Sets the bit for the file descriptor fd in the file descriptor set fdset.
// void FD_ZERO(fd_set *fdset)
        if fds.fd_array[i] == fd {
            return;
        }
        if fds.fd_array[i] == RETIRED_FD {
            fds.fd_array[i] = fd;
            fds.fd_count += 1;
            return;
        }
        i += 1;
    }
}

#[allow(non_snake_case)]
pub fn FD_CLR(fd: ZmqFd, fds: &mut ZmqFdSet) {
    let mut i = 0;
    while i < fds.fd_array.len() {
        if fds.fd_array[i] == fd {
            fds.fd_array[i] = RETIRED_FD;
            fds.fd_count -= 1;
            return;
        }
        i += 1;
    }
}

pub fn zmq_poll_int(poll_fds: &mut [ZmqPollFd], nitems: u32, timeout: u32) -> Result<(), ZmqError> {
    platform_poll(poll_fds, nitems, timeout)
}

#[derive(Default,Debug,Clone, PartialEq)]
pub struct ZmqFdEntry<'a> {
    pub fd: ZmqFd,
    // pub events: &'a mut ZmqPollerEvent<'a>,
    pub events: Vec<ZmqPollerEvent<'a>>
}

pub fn remove_fd_entry(fd_entries_: &mut Vec<ZmqFdEntry>, entry: &ZmqFdEntry) {
    for i in 0..fd_entries_.len() {
        if fd_entries_[i].fd == entry.fd {
            fd_entries_.remove(i);
            break;
        }
    }
}

#[derive(Default,Clone)]
pub struct ZmqFamilyEntry<'a> {
    pub fd_entries: Vec<ZmqFdEntry<'a>>,
    pub fds_set: ZmqFdsSet,
    pub has_retired: bool,
}

pub struct TimerInfo<'a> {
    pub sink: &'a mut ZmqPollerEvent<'a>,
    pub id: i32,
}

#[derive(PartialEq, Default, Debug, Clone)]
pub struct ZmqFdsSet {
    pub read: ZmqFdSet,
    pub write: ZmqFdSet,
    pub error: ZmqFdSet,
}

impl ZmqFdsSet {
    pub fn new() -> Self {
        Self {
            read: ZmqFdSet {
                fd_count: 0,
                fd_array: [0 as ZmqFd; 64],
            },
            write: ZmqFdSet {
                fd_count: 0,
                fd_array: [0 as ZmqFd; 64],
            },
            error: ZmqFdSet {
                fd_count: 0,
                fd_array: [0 as ZmqFd; 64],
            },
        }
    }

    pub fn new2(other_: &crate::poll::select::ZmqFdsSet) -> Self {
        Self {
            read: ZmqFdSet {
                fd_count: other_.read.fd_count,
                fd_array: other_.read.fd_array,
            },
            write: ZmqFdSet {
                fd_count: other_.write.fd_count,
                fd_array: other_.write.fd_array,
            },
            error: ZmqFdSet {
                fd_count: other_.error.fd_count,
                fd_array: other_.error.fd_array,
            },
        }
    }

    pub fn remove_fd(&mut self, fd_: &ZmqFd) {
        FD_CLR(*fd_, &mut self.read);
        FD_CLR(*fd_, &mut self.write);
        FD_CLR(*fd_, &mut self.error);
    }
}

#[derive(Default, Debug, Clone)]
pub struct ZmqPoller<'a> {
    pub _clock: Duration,
    pub _timers: HashMap<u64, TimerInfo<'a>>,
    pub _load: AtomicU32,
    pub _active: bool,
    pub _worker: ZmqThread<'a>,
    #[cfg(target_os = "windows")]
    pub _family_entries: HashMap<u16, ZmqFamilyEntry<'a>>,
    #[cfg(target_os = "windows")]
    pub _fd_family_cache: [(ZmqFd, u16); FD_FAMILY_CACHE_SIZE],
    #[cfg(not(target_os = "windows"))]
    pub _family_entry: ZmqFamilyEntry<'a>,
    #[cfg(not(target_os = "windows"))]
    pub _max_fd: ZmqFd,
    #[cfg(target_os = "windows")]
    pub _current_family_entry_it: &'a mut ZmqFamilyEntry<'a>,
}

impl<'a> ZmqPoller<'a> {
    pub fn adjust_load(&mut self, amount: i32) {
        if amount > 0 {
            self._load.add(amount);
        } else {
            self._load.sub(amount * -1);
        }
    }

    pub fn add_timer(&mut self, timeout_: i32, sink_: Option<&mut ZmqPollerEvent>, id_: i32) {
        let expiration = Utc::now() + chrono::Duration::milliseconds(timeout_ as i64);

        let l_sink = if sink_.is_some() {
            sink_.unwrap()
        } else {
            &mut ZmqPollerEvent::default();
        };


        let info: TimerInfo = TimerInfo {
            sink: l_sink,
            id: id_,
        };
        self._timers.insert(expiration.timestamp_millis() as u64, info);
    }

    pub fn cancel_timer(&mut self, sink_: Option<&ZmqPollerEvent>, id_: i32) {
        let mut to_remove: Vec<u64> = Vec::new();
        for (key, value) in &self._timers {
            if sink_.is_some() {
                if value.sink != sink_.unwrap() {
                    continue;
                }
            }
            if value.id != id_ {
                continue;
            }
            to_remove.push(*key);
        }
        for key in to_remove {
            self._timers.remove(&key);
        }
    }

    pub fn execute_timers(&mut self) -> u64 {
        if self._timers.len() == 0 {
            return 0;
        }

        let mut res = 0u64;

        let current = Utc::now().timestamp_millis() as u64;

        for k in self._timers.keys() {
            let info = self._timers.get_mut(k).unwrap();
            if *k > current {
                continue;
            }

            info.sink.timer_event(info.id);
            self._timers.remove(k);
        }

        res
    }

    pub fn stop_worker(&mut self) {
        self._worker.stop();
    }

    pub fn start(&mut self, name_: &str, ctx: &mut ZmqContext) {
        // TODO: figure out how to pass ZmqWorkerPoller to thread as arg
        ctx.start_thread(&mut self._worker, Self::worker_routine, &[0u8], name_);
    }

    pub fn check_thread(&mut self) {
        unimplemented!("check_thread")
    }

    // TODO: fix up to get worker instance from arg
    pub fn worker_routine(arg_: &[u8]) {
        // let worker: &mut ZmqWorkerPollerBase = unsafe { &mut *(arg_ as *mut ZmqWorkerPollerBase) };
        let mut worker: &mut ZmqPoller;
        worker.loop_op();
    }

    pub fn add_fd(&mut self, fd_: ZmqFd, events_: &mut ZmqPollerEvent) -> ZmqFd {
        self.check_thread();
        let mut fd_entry = ZmqFdEntry {
            fd: fd_,
            events: events_,
        };
        let mut family_entry: &mut ZmqFamilyEntry;
        #[cfg(target_os = "windows")]
        {
            let family = self._worker_poller_base.get_fd_family(fd_);
            family_entry = self._family_entries.get_mut(family).unwrap();
        }
        #[cfg(not(target_os = "windows"))]
        {
            family_entry = &mut self._family_entry;
        }

        family_entry.fd_entries.push(fd_entry);
        FD_SET(fd_, &mut family_entry.fds_set.error);

        #[cfg(not(target_os = "windows"))]
        {
            if fd_ > self._max_fd {
                self._max_fd = fd_;
            }
        }

        // todo
        // self._worker_poller_base._poller_base.adjust_load(family_entry);
        return fd_;
    }

    pub fn find_fd_entry_by_handle(
        &mut self,
        fd_entries_: &mut Vec<ZmqFdEntry>,
        handle_: &ZmqFd,
    ) -> Option<&mut ZmqFdEntry> {
        // let fd_entry_it = fd_entries_.iter();
        // for i in 0..fd_entries_.len() {
        //     if fd_entry_it[i].fd == handle_ {
        //         return fd_entry_it[i];
        //     }
        // }
        for fde in fd_entries_.iter_mut() {
            if fde.fd == *handle_ {
                return Some(fde);
            }
        }

        return fd_entries_.last_mut();
    }

    pub fn trigger_events(
        &mut self,
        fd_entries_: &mut Vec<ZmqFdEntry>,
        local_fds_set_: &ZmqFdsSet,
        mut event_count_: i32,
    ) {
        for i in 0..fd_entries_.len() {
            if event_count_ <= 0 {
                break;
            }
            if is_retired_fd(fd_entries_[i].fd) {
                continue;
            }

            if FD_ISSET(fd_entries_[i].fd, &local_fds_set_.read) {
                // TODO
                // fd_entries_[i].events.on_read(&mut fd_entries_[i].events);
                event_count_ -= 1;
            }

            if is_retired_fd(fd_entries_[i].fd) || event_count_ == 0 {
                continue;
            }

            if FD_ISSET(fd_entries_[i].fd, &local_fds_set_.write) {
                // TODO
                // fd_entries_[i].events.on_write(&mut fd_entries_[i].events);
                event_count_ -= 1;
            }

            if is_retired_fd(fd_entries_[i].fd) || event_count_ == 0 {
                continue;
            }

            if FD_ISSET(fd_entries_[i].fd, &local_fds_set_.error) {
                // TODO
                //fd_entries_[i].events.on_error(&mut fd_entries_[i].events);
                event_count_ -= 1;
            }
        }
    }

    #[cfg(target_os = "windows")]
    pub fn try_retire_fd_entry(
        &mut self,
        family_entry_it_: &mut ZmqFamilyEntry,
        handle_: &ZmqFd,
    ) -> Result<(), ZmqError> {
        // let family_entry: &mut family_entry_t = family_entry_it_.;
        let mut fd_entry_it = self.find_fd_entry_by_handle(&mut family_entry_it_.fd_entries, handle_ as ZmqHandle).unwrap();
        if fd_entry_it == family_entry_it_.fd_entries.last() {
            return Ok(());
        }

        if family_entry_it_ != self._current_family_entry_it {
            // family_entry_it_.fd_entries.remove(fd_entry_it);
            remove_fd_entry(&mut family_entry_it_.fd_entries, &fd_entry_it);
        } else {
            fd_entry_it.fd = -1 as ZmqFd;
            family_entry_it_.has_retired = true;
        }
        family_entry_it_.fds_set.remove_fd(handle_);
        1
    }

    pub fn rm_fd(&mut self, desc: &mut ZmqFd) {
        self.check_thread();
        let mut retired = 0;

        #[cfg(target_os = "windows")]
        {
            let family = self._worker_poller_base.get_fd_family(desc);
            if family != AF_UNSPEC {
                let family_entry_it = self._family_entries.get_mut(family).unwrap();
                retired = unsafe { self.try_retire_fd_entry(family_entry_it, &desc) };
            } else {
                // let end = self._family_entries.iter().last().unwrap().1
                for it in self._family_entries.iter_mut() {
                    retired = unsafe { self.try_retire_fd_entry(it.1, &desc) };
                    if retired != 0 {
                        break;
                    }
                }
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            let mut fd_entry = self.find_fd_entry_by_handle(&mut self._family_entry.fd_entries, desc);
            if fd_entry.is_none() {
                return;
            }

            self._family_entry.fds_set.remove_fd(&fd_entry.unwrap().fd);
            fd_entry.unwrap().fd = -1 as ZmqFd;
            retired += 1;

            if *desc == self._max_fd {
                self._max_fd = RETIRED_FD;
                for fd_entry_it in self._family_entry.fd_entries.iter() {
                    if fd_entry_it.fd > self._max_fd {
                        self._max_fd = fd_entry_it.fd;
                    }
                }
            }

            self._family_entry.has_retired = true;
        }
        self.adjust_load(-1);
    }

    pub fn set_pollin(&mut self, handle_: &mut ZmqHandle) {
        self.check_thread();
        let mut family_entry: &mut ZmqFamilyEntry;
        #[cfg(target_os = "windows")]
        {
            let family = self._worker_poller_base.get_fd_family(handle_);
            family_entry = self._family_entries.get_mut(family).unwrap();
        }
        #[cfg(not(target_os = "windows"))]
        {
            family_entry = &mut self._family_entry;
        }
        // TODO: get FD for handle or set argument to be a ZmqFd
        // FD_SET(handle_, &mut family_entry.fds_set.read);
    }

    pub fn reset_pollin(&mut self, handle_: ZmqHandle) {
        self.check_thread();
        let mut family_entry: &mut ZmqFamilyEntry;
        #[cfg(target_os = "windows")]
        {
            let family = self._worker_poller_base.get_fd_family(handle_);
            family_entry = self._family_entries.get_mut(family).unwrap();
        }
        #[cfg(not(target_os = "windows"))]
        {
            family_entry = &mut self._family_entry;
        }
        // TODO: get FD for handle or set argument to be a ZmqFd
        // FD_CLR(handle_, &mut family_entry.fds_set.read);
    }

    pub fn set_pollout(&mut self, handle_: ZmqHandle) {
        self.check_thread();
        let mut family_entry: &mut ZmqFamilyEntry;
        #[cfg(target_os = "windows")]
        {
            let family = self._worker_poller_base.get_fd_family(handle_);
            family_entry = self._family_entries.get_mut(family).unwrap();
        }
        #[cfg(not(target_os = "windows"))]
        {
            family_entry = &mut self._family_entry;
        }
        // TODO: get FD for handle or set argument to be a ZmqFd
        // FD_SET(handle_, &mut family_entry.fds_set.write);
    }

    pub fn reset_pollout(&mut self, handle_: ZmqHandle) {
        self.check_thread();
        let mut family_entry: &mut ZmqFamilyEntry;
        #[cfg(target_os = "windows")]
        {
            let family = self._worker_poller_base.get_fd_family(handle_);
            family_entry = self._family_entries.get_mut(family).unwrap();
        }
        #[cfg(not(target_os = "windows"))]
        {
            family_entry = &mut self._family_entry;
        }
        // TODO: get FD for handle or set argument to be a ZmqFd
        // FD_CLR(handle_, &mut family_entry.fds_set.write);
    }

    pub fn stop(&mut self) {
        unimplemented!()
    }

    pub fn loop_op(&mut self) {
        loop {
            //  Execute any due timers.
            // int timeout = static_cast<int> (execute_timers ());
            let timeout = self.execute_timers();

            self.cleanup_retired_2();

            let mut entries_condition: bool = false;
            #[cfg(target_os = "windows")]
            {
                entries_condition = self._family_entries.len() > 0;
            }
            #[cfg(not_target_os = "windows")]
            {
                entries_condition = self._family_entry.fd_entries.len() > 0;
            }
            if !entries_condition {
                // zmq_assert (get_load () == 0);

                if (timeout == 0) {
                    break;
                }

                // TODO sleep for timeout
                continue;
            }

            // #if defined ZMQ_HAVE_OSX
            //         struct timeval tv = {(long) (timeout / 1000), timeout % 1000 * 1000};
            // #else
            //         struct timeval tv = {static_cast<long> (timeout / 1000),
            //                              static_cast<long> (timeout % 1000 * 1000)};
            // #endif

            // #if defined ZMQ_HAVE_WINDOWS
            #[cfg(target_os = "windows")]
            {
                /*
                    On Windows select does not allow to mix descriptors from different
                    service providers. It seems to work for AF_INET and AF_INET6,
                    but fails for AF_INET and VMCI. The workaround is to use
                    WSAEventSelect and WSAWaitForMultipleEvents to wait, then use
                    select to find out what actually changed. WSAWaitForMultipleEvents
                    cannot be used alone, because it does not support more than 64 events
                    which is not enough.

                    To reduce unnecessary overhead, WSA is only used when there are more
                    than one family. Moreover, AF_INET and AF_INET6 are considered the same
                    family because Windows seems to handle them properly.
                    See get_fd_family for details.
                */

                //  If there is just one family, there is no reason to use WSA events.
                let mut rc = 0;
                let use_wsa_events = self._family_entries.size() > 1;
                if (use_wsa_events) {
                    // TODO: I don't really understand why we are doing this. If any of
                    // the events was signaled, we will call select for each fd_family
                    // afterwards. The only benefit is if none of the events was
                    // signaled, then we continue early.
                    // IMHO, either WSAEventSelect/WSAWaitForMultipleEvents or select
                    // should be used, but not both

                    // wsa_events_t
                    // wsa_events;
                    let wsa_events: crate::poll::select::ZmqWsaEvents = crate::poll::select::ZmqWsaEvents {
                        events: [0 as crate::defines::WSAEVENT; 4],
                    };

                    // for (family_entries_t::iterator family_entry_it = _family_entries.begin();
                    // family_entry_it != _family_entries.end();
                    // + +family_entry_it)
                    for family_entry_it in self._family_entries.iter() {
                        // family_entry_t& family_entry = family_entry_it -> second;
                        let family_entry = family_entry_it.1;

                        // for (fd_entries_t::iterator fd_entry_it = family_entry.fd_entries.begin();
                        // fd_entry_it != family_entry.fd_entries.end();
                        // + +fd_entry_it)
                        for fd_entry_it in family_entry.fd_entries.iter() {
                            let fd = fd_entry_it.fd;

                            //  http://stackoverflow.com/q/35043420/188530
                            if (FD_ISSET(fd, &family_entry.fds_set.read) && FD_ISSET(fd, &family_entry.fds_set.write)) {
                                rc = unsafe{WSAEventSelect(
                                    fd,
                                    wsa_events.events[3],
                                    (FD_READ | FD_ACCEPT | FD_CLOSE | FD_WRITE | FD_CONNECT) as i32,
                                )};
                            } else if (FD_ISSET(fd, &family_entry.fds_set.read)) {
                                rc = unsafe{WSAEventSelect(
                                    fd,
                                    wsa_events.events[0],
                                    (FD_READ | FD_ACCEPT | FD_CLOSE) as i32,
                                )};
                            } else if (FD_ISSET(fd, &family_entry.fds_set.write)) {
                                rc = unsafe{WSAEventSelect(
                                    fd,
                                    wsa_events.events[1],
                                    (FD_WRITE | FD_CONNECT) as i32,
                                )};
                            } else {
                                rc = 0;
                            }

                            // wsa_assert(rc != SOCKET_ERROR);
                        }
                    }

                    rc = unsafe{WSAWaitForMultipleEvents(
                        &wsa_events.events as &[HANDLE],
                        FALSE,
                        if timeout { timeout } else { INFINITE },
                        FALSE,
                    ) as i32};
                    // wsa_assert(rc != (int) WSA_WAIT_FAILED);
                    // zmq_assert(rc != WSA_WAIT_IO_COMPLETION);

                    if (rc == WSA_WAIT_TIMEOUT) {
                        continue;
                    }
                }

                // for (_current_family_entry_it = _family_entries.begin();
                // _current_family_entry_it != _family_entries.end();
                // + +_current_family_entry_it)
                for entry in self._family_entries.iter_mut() {
                    self._current_family_entry_it = entry.1;
                    let family_entry = self._current_family_entry_it;

                    if (use_wsa_events) {
                        //  There is no reason to wait again after WSAWaitForMultipleEvents.
                        //  Simply collect what is Ready. struct timeval
                        let mut tv_nodelay = ZmqTimeval::default();
                        self.select_family_entry(
                            family_entry,
                            0,
                            true,
                            &mut tv_nodelay,
                        );
                    } else {
                        // self.select_family_entry(family_entry, 0, timeout > 0, tv);
                    }
                }
            }
            // #else
            #[cfg(not_target_os = "windows")]
            {
                self.select_family_entry(_family_entry, _max_fd + 1, timeout > 0, tv);
            }
            // #endif
        }
    }

    pub fn select_family_entry(
        &mut self,
        family_entry_: &mut ZmqFamilyEntry,
        max_fd_: i32,
        use_timeout_: bool,
        tv_: &mut ZmqTimeval,
    ) -> Result<(), ZmqError> {
        //  select will fail when run with empty sets.
        let fd_entries = &mut family_entry_.fd_entries;
        if fd_entries.is_empty() {
            return Ok(());
        }

        let local_fds_set = &mut family_entry_.fds_set;
        // let rc = select(
        //     max_fd_,
        //     Some(&mut local_fds_set.read as *mut FD_SET),
        //     Some(&mut local_fds_set.write as *mut FD_SET),
        //     Some(&mut local_fds_set.error as *mut FD_SET),
        //     if use_timeout_ {
        //         Some(tv_ as *const TIMEVAL)
        //     } else {
        //         None
        //     },
        // );
        let event_count = platform_select(
            max_fd_,
            Some(&mut local_fds_set.read),
            Some(&mut local_fds_set.write),
            Some(&mut local_fds_set.error),
            if use_timeout_ {
                Some(tv_)
            } else {
                None
            })?;

        // #if defined ZMQ_HAVE_WINDOWS
        //     wsa_assert (rc != SOCKET_ERROR);
        // #else
        //     if (rc == -1) {
        //         errno_assert (errno == EINTR);
        //         return;
        //     }
        // #endif

        self.trigger_events(fd_entries, local_fds_set, event_count);

        self.cleanup_retired(family_entry_);

        return Ok(());
    }

    pub fn cleanup_retired(&mut self, family_entry_: &mut ZmqFamilyEntry) -> bool {
        if family_entry_.has_retired {
            family_entry_.has_retired = false;
            for i in 0..family_entry_.fd_entries.len() {
                if family_entry_.fd_entries[i].fd == -1 as ZmqFd {
                    family_entry_.fd_entries.remove(i);
                }
            }
        }
        family_entry_.fd_entries.is_empty()
    }

    pub fn cleanup_retired_2(&mut self) {
        #[cfg(target_os = "windows")]
        {
            for it in self._family_entries.iter_mut() {
                self.cleanup_retired(it.1);
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            self.cleanup_retired(&mut self._family_entry);
        }
    }

    pub fn is_retired_fd(&mut self, entry_: &ZmqFdEntry) -> bool {
        entry_.fd == -1 as ZmqFd
    }

    #[cfg(target_os = "windows")]
    pub fn get_fd_family(&mut self, fd_: ZmqFd) -> u16 {
        // cache the results of determine_fd_family, as this is frequently called
        // for the same sockets, and determine_fd_family is expensive
        // size_t i;
        // for (i = 0; i < FD_FAMILY_CACHE_SIZE; ++i) {
        let mut i = 0;
        for i in 0..FD_FAMILY_CACHE_SIZE {
            let entry = self._fd_family_cache[i];
            if (entry.0 == fd_) {
                return entry.1;
            }
            if entry.0 == RETIRED_FD as ZmqFd {
                break;
            }
        }

        // std::pair<fd_t, u_short> res =
        //   std::make_pair (fd_, determine_fd_family (fd_));
        let res = (fd_, self.determine_fd_family(fd_));
        if (i < FD_FAMILY_CACHE_SIZE) {
            self._fd_family_cache[i] = res;
        } else {
            // just overwrite a random entry
            // could be optimized by some LRU strategy
            self._fd_family_cache[platform_random() % FD_FAMILY_CACHE_SIZE] = res;
        }

        return res.1;
    }

    #[cfg(target_os = "windows")]
    pub fn determine_fd_family(&mut self, fd_: ZmqFd) -> u16 {
        //  Use sockaddr_storage instead of sockaddr to accommodate different structure sizes
        // sockaddr_storage addr = {0};
        let mut addr = SOCKADDR_STORAGE::default();
        // int addr_size = sizeof addr;
        let mut addr_size = size_of_val(&addr);

        let mut type_ = 0;
        let mut type_length = 4;

        let mut rc = getsockopt(
            fd_ as SOCKET,
            SOL_SOCKET,
            SO_TYPE,
            &mut type_ as *mut libc::c_char,
            &mut type_length,
        );

        if (rc == 0) {
            if (type_ == SOCK_DGRAM) {
                return AF_INET as u16;
            }

            rc = getsockname(
                fd_ as SOCKET,
                &mut addr as *mut SOCKADDR,
                &mut addr_size as *mut libc::c_int,
            );

            //  AF_INET and AF_INET6 can be mixed in select
            //  TODO: If proven otherwise, should simply return addr.sa_family
            if (rc != SOCKET_ERROR) {
                return if addr.ss_family == AF_INET6 {
                    AF_INET
                } else {
                    addr.ss_family
                } as u16;
            }
        }

        return AF_UNSPEC.0;
    }
}
