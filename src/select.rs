/*
    Copyright (c) 2007-2016 Contributors as noted in the AUTHORS file

    This file is part of libzmq, the ZeroMQ core engine in C+= 1.

    libzmq is free software; you can redistribute it and/or modify it under
    the terms of the GNU Lesser General Public License (LGPL) as published
    by the Free Software Foundation; either version 3 of the License, or
    (at your option) any later version.

    As a special exception, the Contributors give you permission to link
    this library with independent modules to produce an executable,
    regardless of the license terms of these independent modules, and to
    copy and distribute the resulting executable under terms of your choice,
    provided that you also meet, for each linked independent module, the
    terms and conditions of the license of that module. An independent
    module is a module which is not derived from or based on this library.
    If you modify this library, you must extend this exception to your
    version of the library.

    libzmq is distributed in the hope that it will be useful, but WITHOUT
    ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
    FITNESS FOR A PARTICULAR PURPOSE. See the GNU Lesser General Public
    License for more details.

    You should have received a copy of the GNU Lesser General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

// #include "precompiled.hpp"
// #include "select.hpp"
// #if defined ZMQ_IOTHREAD_POLLER_USE_SELECT

// #if defined ZMQ_HAVE_WINDOWS
// #elif defined ZMQ_HAVE_HPUX
// // #include <sys/param.h>
// // #include <sys/types.h>
// // #include <sys/time.h>
// #elif defined ZMQ_HAVE_OPENVMS
// // #include <sys/types.h>
// // #include <sys/time.h>
// #elif defined ZMQ_HAVE_VXWORKS
// #include <sys/types.h>
// #include <sys/time.h>
// #include <strings.h>
// #else
// #include <sys/select.h>
// #endif

// #include "err.hpp"
// #include "config.hpp"
// #include "i_poll_events.hpp"

use std::collections::HashMap;
use std::mem;
use libc::{memcpy, timeval};
use windows::Win32::Foundation::FALSE;
use windows::Win32::Networking::WinSock::{FD_ACCEPT, FD_CLOSE, FD_CONNECT, FD_READ, FD_SET, FD_SETSIZE, FD_WRITE, select, SO_TYPE, SOCK_DGRAM, SOCKET_ERROR, SOL_SOCKET, WSA_WAIT_FAILED, WSA_WAIT_TIMEOUT, WSACloseEvent, WSACreateEvent, WSAEventSelect, WSAWaitForMultipleEvents};
use windows::Win32::System::Threading::INFINITE;
use crate::address_family::AF_UNSPEC;
use crate::decoder_allocators::size;
use crate::defines::ZmqHandle;
use crate::fd::ZmqFileDesc;
use crate::mechanism::ZmqMechanismStatus::error;
use crate::poll_events_interface::ZmqPollEventsInterface;
use crate::poller_base::WorkerPollerBase;
use crate::thread_ctx::ThreadCtx;
use crate::utils::copy_bytes;

//  Internal state.
#[derive(Default, Debug, Clone)]
struct fds_set_t {
    // fds_set_t ();
    // fds_set_t (const fds_set_t &other_);
    // fds_set_t &operator= (const fds_set_t &other_);
    //  Convenience method to descriptor from all sets.
    // void remove_fd (const ZmqFileDesc &fd);

    // fd_set read;
    pub read: fd_set,
    // fd_set write;
    pub write: fd_set,
    // fd_set error;
    pub error: fd_set,
}

#[derive(Default, Debug, Clone)]
struct FdEntry {
    // ZmqFileDesc fd;
    fd: ZmqFileDesc,
    // ZmqPollEventsInterface *events;
}

#[derive(Default, Debug, Clone)]
struct family_entry_t {
    // family_entry_t ();
    // fd_entries_t fd_entries;
    fd_entries: Vec<FdEntry>,
    // fds_set_t fds_set;
    fds_set: fds_set_t,
    has_retired: bool,
}

#[derive(Default, Debug, Clone)]
struct wsa_events_t {
    // wsa_events_t ();
    // ~wsa_events_t ();

    //  read, write, error and readwrite
    pub events: [WSAEVENT; 4],
}

// static const size_t fd_family_cache_size = 8; const fd_family_cache_size: usize = 8;

// #include <algorithm>
// #include <limits>
// #include <climits>
pub struct ZmqSelect<'a> {
    // : public WorkerPollerBase
    pub base: WorkerPollerBase<'a>,
//     typedef ZmqFileDesc handle_t;

    // ZmqSelect (const ThreadCtx &ctx);
    // ~ZmqSelect () ;
    //
    //  "poller" concept.
    // handle_t add_fd (fd: ZmqFileDesc, ZmqPollEventsInterface *events_);
    // void rm_fd (handle_t handle_);
    // void set_pollin (handle_t handle_);
    // void reset_pollin (handle_t handle_);
    // void set_pollout (handle_t handle_);
    // void reset_pollout (handle_t handle_);
    // void stop ();
    // static int max_fds ();
    //  Main event loop.
    // void loop () ;
    // void trigger_events (const fd_entries_t &fd_entries_,
    //                          const fds_set_t &local_fds_set_,
    //                          event_count_: i32);
    // typedef std::vector<FdEntry> fd_entries_t;
    // void select_family_entry (family_entry_t &family_entry_,
    //                           max_fd_: i32,
    //                           use_timeout_: bool,
    //                           struct timeval &tv_);
    // int try_retire_fd_entry (family_entries_t::iterator family_entry_it_,
    //                              ZmqFileDesc &handle_);
    // u_short get_fd_family (ZmqFileDesc fd);
    // static u_short determine_fd_family (ZmqFileDesc fd);
    // void cleanup_retired ();
    //     bool cleanup_retired (family_entry_t &family_entry_);
    //  Checks if an FdEntry is retired.
    // static bool is_retired_fd (const FdEntry &entry_);
    // static fd_entries_t::iterator
    //     find_fd_entry_by_handle (fd_entries_t &fd_entries_, handle_t handle_);
// #if defined ZMQ_HAVE_WINDOWS
//     typedef std::map<u_short, family_entry_t> family_entries_t;
    // family_entries_t _family_entries;
    pub _family_entires: HashMap<u16, family_entry_t>,
    // See loop for details.
    // family_entries_t::iterator _current_family_entry_it;
    // std::pair<ZmqFileDesc, u_short> _fd_family_cache[fd_family_cache_size];
    pub _fd_family_cache: [(ZmqFileDesc, u16); fd_family_cache_size],
    //  Socket's family or AF_UNSPEC on error.
// #else
    //  on non-Windows, we can treat all fds as one family
    // family_entry_t _family_entry;
    pub _family_entry: family_entry_t,
    // ZmqFileDesc _max_fd;
    pub _max_fd: ZmqFileDesc,
// #endif
}

// typedef ZmqSelect Poller;

impl ZmqSelect {
    pub fn new(ctx: &ThreadCtx) -> Self {
        //     WorkerPollerBase (ctx),
        // // #if defined ZMQ_HAVE_WINDOWS
        //     //  Fine as long as map is not cleared.
        //     _current_family_entry_it (_family_entries.end ())
        // // #else
        //     _max_fd (retired_fd)
        // // #endif
// #if defined ZMQ_HAVE_WINDOWS
//         for (size_t i = 0; i < fd_family_cache_size; += 1i)
        for i in 0..fd_family_cache_size {
            _fd_family_cache[i] = std::make_pair(retired_fd, 0);
        }
// #endif
        Self {
            base: WorkerPollerBase::new(ctx),
            _family_entires: Default::default(),
            _fd_family_cache: [(retired_fd, 0); fd_family_cache_size],
            _family_entry: Default::default(),
            _max_fd: 0,
        }
    }

    pub fn add_fd(&mut self, fd: ZmqFileDesc, events_: &mut ZmqPollEventsInterface) -> ZmqFileDesc {
        check_thread();
        // zmq_assert (fd != retired_fd);

        let mut fd_entry = FdEntry::new(fd, events_);
        // fd_entry.fd = fd;
        // fd_entry.events = events_;

// #if defined ZMQ_HAVE_WINDOWS
        let family = get_fd_family(fd);
        // wsa_assert (family != AF_UNSPEC);
        family_entry_t & family_entry = _family_entries[family];
// #else
        family_entry_t & family_entry = _family_entry;
// #endif
        family_entry.fd_entries.push_back(fd_entry);
        FD_SET(fd, &family_entry.fds_set.error);

// #if !defined ZMQ_HAVE_WINDOWS
        if (fd > _max_fd) {
            _max_fd = fd;
        }
// #endif

        adjust_load(1);

        return fd;
    }

    pub fn find_fd_entry_by_handle(&mut self,
                                   fd_entries_: &mut Vec<FdEntry>,
                                   handle_: ZmqHandle) -> Option<&mut FdEntry> {
        // fd_entries_t::iterator fd_entry_it;
        // for (fd_entry_it = fd_entries_.begin (); fd_entry_it != fd_entries_.end ();
        //      += 1fd_entry_it)
        let mut out_entry: Option<&mut FdEntry> = None;
        for entry in fd_entries_.iter_mut() {
            if fd_entry_it.fd == handle_ {
                out_entry = Some(entry);
                break;
            }
        }

        return out_entry;
    }


    pub fn trigger_events(&mut self,
                          fd_entries: &mut Vec<FdEntry>,
                          local_fds_set_: &mut fds_set_t,
                          mut event_count_: i32) {
        //  Size is cached to avoid iteration through recently added descriptors.
        // for (fd_entries_t::size_type i = 0, size = fd_entries_.size ();
        //     i < size && event_count_ > 0; += 1i)
        let mut size = fd_entries.len();
        for i in 0..fd_entries.len() {
            if i >= size || event_count_ <= 0 {
                break;
            }
            //  fd_entries_[i] may not be stored, since calls to
            //  in_event/out_event may reallocate the vector

            if (is_retired_fd(fd_entries_[i])) {
                continue;
            }

            if (FD_ISSET(fd_entries_[i].fd, &local_fds_set_.read)) {
                fd_entries_[i].events.in_event();
                event_count_ -= 1;
            }

            //  TODO: can the is_retired_fd be true at this point? if it
            //  was retired before, we would already have continued, and I
            //  don't see where it might have been modified
            //  And if rc == 0, we can break instead of continuing
            if (is_retired_fd(fd_entries_[i]) || event_count_ == 0) {
                continue;
            }

            if (FD_ISSET(fd_entries_[i].fd, &local_fds_set_.write)) {
                fd_entries_[i].events.out_event();
                event_count_ -= 1;
            }

            //  TODO: same as above
            if (is_retired_fd(fd_entries_[i]) || event_count_ == 0) {
                continue;
            }

            if (FD_ISSET(fd_entries_[i].fd, &local_fds_set_.error)) {
                fd_entries_[i].events.in_event();
                event_count_ -= 1;
            }
            size += 1;
        }
    }

    pub fn try_retire_fd_entry(
        family_entry_it: &mut [family_entry_t],
        handle_: &mut ZmqFileDesc) -> i32 {
        let mut family_entry = &mut family_entry_it[0];

        fd_entry_it = find_fd_entry_by_handle(&family_entry.fd_entries, handle_);

        if (fd_entry_it == family_entry.fd_entries.end()) {
            return 0;
        }

        FdEntry & fd_entry = *fd_entry_it;
        // zmq_assert (fd_entry.fd != retired_fd);

        if (family_entry_it_ != _current_family_entry_it) {
            //  Family is not currently being iterated and can be safely
            //  modified in-place. So later it can be skipped without
            //  re-verifying its content.
            family_entry.fd_entries.erase(fd_entry_it);
        } else {
            //  Otherwise mark removed entries as retired. It will be cleaned up
            //  at the end of the iteration. See ZmqSelect::loop
            fd_entry.fd = retired_fd;
            family_entry.has_retired = true;
        }
        family_entry.fds_set.remove_fd(handle_);
        return 1;
    }


    pub fn rm_fd(&mut self, handle_: ZmqHandle) {
        check_thread();
        let mut retired = 0;
// #if defined ZMQ_HAVE_WINDOWS
        let family = get_fd_family(handle_);
        if (family != AF_UNSPEC) {
            let family_entry_it = _family_entries.find(family);

            retired += try_retire_fd_entry(family_entry_it, handle_);
        } else {
            //  get_fd_family may fail and return AF_UNSPEC if the socket was not
            //  successfully connected. In that case, we need to look for the
            //  socket in all family_entries.
            let end = _family_entries.end();
            // for (family_entries_t::iterator family_entry_it =
            //     _family_entries.begin ();
            //     family_entry_it != end; += 1family_entry_it)
            for family_entry_it in _family_entries.iter_mut() {
                if (retired += try_retire_fd_entry(family_entry_it, handle_)) {
                    break;
                }
            }
        }
// #else
        let fd_entry_it = find_fd_entry_by_handle(_family_entry.fd_entries, handle_);
        assert(fd_entry_it != _family_entry.fd_entries.end());

        // zmq_assert (fd_entry_it.fd != retired_fd);
        fd_entry_it.fd = retired_fd;
        _family_entry.fds_set.remove_fd(handle_);

        retired += 1;

        if (handle_ == _max_fd) {
            _max_fd = retired_fd;
            // for (fd_entry_it = _family_entry.fd_entries.begin ();
            //     fd_entry_it != _family_entry.fd_entries.end (); += 1fd_entry_it)
            for fd_entry_it in _family_entry.fd_entries.iter_mut() {
                if (fd_entry_it.fd > _max_fd) {
                    _max_fd = fd_entry_it.fd;
                }
            }
        }

        _family_entry.has_retired = true;
// #endif
        // zmq_assert (retired == 1);
        adjust_load(-1);
    }

    pub fn set_pollin(&mut self, handle_: ZmqHandle) {
        check_thread();
// #if defined ZMQ_HAVE_WINDOWS
        let family = get_fd_family(handle_);
        wsa_assert(family != AF_UNSPEC);
        family_entry_t & family_entry = _family_entries[family];
// #else
        family_entry_t & family_entry = _family_entry;
// #endif
        FD_SET(handle_, &family_entry.fds_set.read);
    }

    pub fn reset_pollin(&mut self, handle_: ZmqHandle) {
        check_thread();
// #if defined ZMQ_HAVE_WINDOWS
        let family = get_fd_family(handle_);
        // wsa_assert (family != AF_UNSPEC);
        family_entry_t & family_entry = _family_entries[family];
// #else
        family_entry_t & family_entry = _family_entry;
// #endif
        FD_CLR(handle_, &family_entry.fds_set.read);
    }

    pub fn set_pollout(&mut self, handle_: ZmqHandle) {
        check_thread();
// #if defined ZMQ_HAVE_WINDOWS
        let family = get_fd_family(handle_);
        // wsa_assert (family != AF_UNSPEC);
        family_entry_t & family_entry = _family_entries[family];
// #else
        family_entry_t & family_entry = _family_entry;
// #endif
        FD_SET(handle_, &family_entry.fds_set.write);
    }

    pub fn reset_pollout(&mut self, handle_: ZmqHandle) {
        check_thread();
// #if defined ZMQ_HAVE_WINDOWS
        let family = get_fd_family(handle_);
        wsa_assert(family != AF_UNSPEC);
        family_entry_t & family_entry = _family_entries[family];
// #else
        family_entry_t & family_entry = _family_entry;
// #endif
        FD_CLR(handle_, &family_entry.fds_set.write);
    }

    pub fn stop(&mut self) {
        check_thread();
        //  no-op... thread is stopped when no more fds or timers are registered
    }

    pub fn max_fds(&mut self) -> i32 {
        return FD_SETSIZE as i32;
    }


    pub fn loop_(&mut self) {
        loop {
            //  Execute any due timers.
            let timeout = (execute_timers());

            cleanup_retired();

// #ifdef _WIN32
            if (_family_entries.empty()) {
// #else
                if (_family_entry.fd_entries.empty()) {
// #endif
                    // zmq_assert (get_load () == 0);

                    if (timeout == 0) {
                        break;
                    }

                    // TODO sleep for timeout
                    continue;
                }

                // #if defined ZMQ_HAVE_OSX
                let tv = timeval { tv_sec: timeout / 1000, tv_usec: timeout % 1000 * 1000 };
                // #else
                // let tv = timeval{static_cast<long> (timeout / 1000),
                //     static_cast<long> (timeout % 1000 * 1000)};
// #endif

// #if defined ZMQ_HAVE_WINDOWS
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
                let use_wsa_events = _family_entries.size() > 1;
                if (use_wsa_events) {
                    // TODO: I don't really understand why we are doing this. If any of
                    // the events was signaled, we will call select for each fd_family
                    // afterwards. The only benefit is if none of the events was
                    // signaled, then we continue early.
                    // IMHO, either WSAEventSelect/WSAWaitForMultipleEvents or select
                    // should be used, but not both

                    let mut wsa_events = wsa_events_t { events: [0; 4] };

                    // for (family_entries_t::iterator family_entry_it =
                    //     _family_entries.begin ();
                    //     family_entry_it != _family_entries.end (); += 1family_entry_it)

                    for family_entry_it in _family_entries.iter_mut() {
                        let family_entry = family_entry_it.second;

                        // for (fd_entries_t::iterator fd_entry_it =
                        //     family_entry.fd_entries.begin ();
                        //     fd_entry_it != family_entry.fd_entries.end ();
                        //     += 1fd_entry_it)
                        for fd_entry_it in fd_entries.iter_mut()

                        {
                            let mut fd: ZmqFileDesc = fd_entry_it.fd;

                            //  http://stackoverflow.com/q/35043420/188530
                            if (FD_ISSET(fd, &family_entry.fds_set.read) && FD_ISSET(fd, &family_entry.fds_set.write)) {
                                unsafe {
                                    rc = WSAEventSelect(fd, wsa_events.events[3],
                                                        (FD_READ | FD_ACCEPT | FD_CLOSE | FD_WRITE | FD_CONNECT) as i32);
                                }
                            } else if (FD_ISSET(fd, &family_entry.fds_set.read)) {
                                unsafe {
                                    rc = WSAEventSelect(fd, wsa_events.events[0],
                                                        (FD_READ | FD_ACCEPT | FD_CLOSE) as i32);
                                }
                            } else if (FD_ISSET(fd, &family_entry.fds_set.write)) {
                                unsafe {
                                    rc = WSAEventSelect(fd, wsa_events.events[1],
                                                        (FD_WRITE | FD_CONNECT) as i32);
                                }
                            } else {
                                rc = 0;
                            }

                            // wsa_assert (rc != SOCKET_ERROR);
                        }
                    }

                    unsafe {
                        // rc = WSAWaitForMultipleEvents(4, wsa_events.events, FALSE,
                        //                               if timeout { timeout } else { INFINITE }, FALSE);
                    }
                    // wsa_assert (rc !=  WSA_WAIT_FAILED);
                    // zmq_assert (rc != WSA_WAIT_IO_COMPLETION);

                    if (rc == WSA_WAIT_TIMEOUT) {
                        continue;
                    }
                }

                // for (_current_family_entry_it = _family_entries.begin ();
                //     _current_family_entry_it != _family_entries.end ();
                //     += 1_current_family_entry_it)
                for _current_family_entry_it in _family_entries.iter_mut() {
                    family_entry_t & family_entry = _current_family_entry_it.second;


                    if (use_wsa_events) {
                        //  There is no reason to wait again after WSAWaitForMultipleEvents.
                        //  Simply collect what is ready.
                        // struct timeval tv_nodelay = {0, 0};
                        let mut tv_nodelay = timeval { tv_sec: 0, tv_usec: 0 };
                        // select_family_entry(family_entry, 0, true, tv_nodelay);
                    } else {
                        // select_family_entry(family_entry, 0, timeout > 0, tv);
                    }
                }
// #else
                select_family_entry(_family_entry, _max_fd + 1, timeout > 0, tv);
// #endif
            }
        }
    }

    pub fn select_family_entry(&mut self, family_entry_: &mut family_entry_t,
                               max_fd_: i32,
                               use_timeout_: bool,
                               tv_: &timeval) {
        //  select will fail when run with empty sets.
        let fd_entries = &mut family_entry_.fd_entries;
        if (fd_entries.empty()) {
            return;
        }

        let local_fds_set = &mut family_entry_.fds_set;
        let rc = unsafe {
            select(max_fd_, Some(&mut local_fds_set.read), Some(&mut local_fds_set.write),
                   Some(&mut local_fds_set.error), use_timeout_? & tv_: null_mut())
        };

// #if defined ZMQ_HAVE_WINDOWS
        wsa_assert(rc != SOCKET_ERROR);
// #else
        if (rc == -1) {
            // errno_assert (errno == EINTR);
            return;
        }
// #endif

        trigger_events(fd_entries, local_fds_set, rc);

        cleanup_retired(family_entry_);
    }

    pub fn cleanup_retired(&mut self, family_entry_: &mut family_entry_t) -> bool {
        if (family_entry_.has_retired) {
            family_entry_.has_retired = false;
            family_entry_.fd_entries.erase(
                std::remove_if(family_entry_.fd_entries.begin(),
                               family_entry_.fd_entries.end(), is_retired_fd),
                family_entry_.fd_entries.end());
        }
        return family_entry_.fd_entries.is_empty();
    }

//     pub fn cleanup_retired (&mut self)
//     {
// // #ifdef _WIN32
//         for (family_entries_t::iterator it = _family_entries.begin ();
//             it != _family_entries.end ();) {
//             if (cleanup_retired (it.second))
//             it = _family_entries.erase (it);
//             else
//             += 1it;
//         }
// // #else
//         cleanup_retired (_family_entry);
// // #endif
//     }

    pub fn is_retired_fd(&mut self, entry_: &FdEntry) -> bool {
        return entry_.fd == retired_fd;
    }

    // #if defined ZMQ_HAVE_WINDOWS
    pub fn get_fd_family(&mut self, fd: ZmqFileDesc) -> u16 {
        // cache the results of determine_fd_family, as this is frequently called
        // for the same sockets, and determine_fd_family is expensive
        let mut i = 0usize;
        // for (i = 0; i < fd_family_cache_size; += 1i)
        for i in 0..fd_family_cache_size {
            let entry = _fd_family_cache[i];
            if (entry.first == fd) {
                return entry.second;
            }
            if (entry.first == retired_fd) {
                break;
            }
        }

        let res = (fd, determine_fd_family(fd));
        if (i < fd_family_cache_size) {
            _fd_family_cache[i] = res;
        } else {
            // just overwrite a random entry
            // could be optimized by some LRU strategy
            // TODO
            // _fd_family_cache[rand () % fd_family_cache_size] = res;
        }

        return res.second;
    }

    pub fn determine_fd_family(&mut self, fd: ZmqFileDesc) -> u16 {
        //  Use sockaddr_storage instead of sockaddr to accommodate different structure sizes
        let addr = sockaddr_storage { 0 };
        // let addr_size = sizeof addr;

        // TODO
        // type: i32;
        // let type_length = 4;
        //
        // int rc = getsockopt (fd, SOL_SOCKET, SO_TYPE,
        //                       (&type), &type_length);

        // if (rc == 0) {
        //     if (type == SOCK_DGRAM){
        //         return AF_INET;
        //     }
        //
        //     rc =
        //       getsockname (fd, reinterpret_cast<sockaddr *> (&addr), &addr_size);
        //
        //     //  AF_INET and AF_INET6 can be mixed in select
        //     //  TODO: If proven otherwise, should simply return addr.sa_family
        //     if (rc != SOCKET_ERROR)
        //         return addr.ss_family == AF_INET6 ? AF_INET : addr.ss_family;
        // }

        return AF_UNSPEC;
    }
}

impl fds_set_t {
    pub fn new() -> Self {
        // FD_ZERO (&read);
        // FD_ZERO (&write);
        // FD_ZERO (&error);
        Self {
            read: (),
            write: (),
            error: (),
        }
    }

    pub fn new2(other_: &mut Self) -> Self {
// #if defined ZMQ_HAVE_WINDOWS
        // On Windows we don't need to copy the whole fd_set.
        // SOCKETS are continuous from the beginning of fd_array in fd_set.
        // We just need to copy fd_count elements of fd_array.
        // We gain huge memcpy() improvement if number of used SOCKETs is much lower than FD_SETSIZE.
//         copy_bytes (&read, &other_.read,
//                 (other_.read.fd_array + other_.read.fd_count)
//                     -  &other_.read);
//         copy_bytes (&write, &other_.write,
//                 (other_.write.fd_array + other_.write.fd_count)
//                     -  &other_.write);
//         copy_bytes (&error, &other_.error,
//                 (other_.error.fd_array + other_.error.fd_count)
//                     -  &other_.error);
// // #else
//         copy_bytes (&read, &other_.read, sizeof other_.read);
//         copy_bytes (&write, &other_.write, sizeof other_.write);
//         copy_bytes (&error, &other_.error, sizeof other_.error);
// #endif
        unimplemented!()
    }

    pub fn remove_fd(&mut self, fd: &mut ZmqFileDesc) {
        // FD_CLR (fd, &read);
        // FD_CLR (fd, &write);
        // FD_CLR (fd, &error);
    }
}


// ZmqSelect::fds_set_t &
// ZmqSelect::fds_set_t::operator= (const fds_set_t &other_)
// {
// // #if defined ZMQ_HAVE_WINDOWS
//     // On Windows we don't need to copy the whole fd_set.
//     // SOCKETS are continuous from the beginning of fd_array in fd_set.
//     // We just need to copy fd_count elements of fd_array.
//     // We gain huge memcpy() improvement if number of used SOCKETs is much lower than FD_SETSIZE.
//     memcpy (&read, &other_.read,
//              (other_.read.fd_array + other_.read.fd_count)
//               -  &other_.read);
//     memcpy (&write, &other_.write,
//              (other_.write.fd_array + other_.write.fd_count)
//               -  &other_.write);
//     memcpy (&error, &other_.error,
//              (other_.error.fd_array + other_.error.fd_count)
//               -  &other_.error);
// // #else
//     memcpy (&read, &other_.read, sizeof other_.read);
//     memcpy (&write, &other_.write, sizeof other_.write);
//     memcpy (&error, &other_.error, sizeof other_.error);
// // #endif
//     return *this;
// }

impl family_entry_t {
    pub fn new() -> Self {
        Self {
            fds_set: fds_set_t::new(),
            fd_entries: Vec::new(),
            has_retired: false,
        }
    }
}


impl wsa_events_t {
    pub fn new() -> Self {
        Self {
            events: [0; 4],
        }
    }
}


// ZmqSelect::wsa_events_t::wsa_events_t ()
// {
//     events[0] = WSACreateEvent ();
//     wsa_assert (events[0] != WSA_INVALID_EVENT);
//     events[1] = WSACreateEvent ();
//     wsa_assert (events[1] != WSA_INVALID_EVENT);
//     events[2] = WSACreateEvent ();
//     wsa_assert (events[2] != WSA_INVALID_EVENT);
//     events[3] = WSACreateEvent ();
//     wsa_assert (events[3] != WSA_INVALID_EVENT);
// }
//
// ZmqSelect::wsa_events_t::~wsa_events_t ()
// {
//     wsa_assert (WSACloseEvent (events[0]));
//     wsa_assert (WSACloseEvent (events[1]));
//     wsa_assert (WSACloseEvent (events[2]));
//     wsa_assert (WSACloseEvent (events[3]));
// }
// #endif

// #endif
