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
// #include "socket_poller.hpp"
// #include "err.hpp"
// #include "polling_util.hpp"
// #include "macros.hpp"

// #include <limits.h>
pub struct socket_poller_t
{
//
    socket_poller_t ();
    ~socket_poller_t ();

    typedef ZmqPollerEvent event_t;

    int add (ZmqSocketBase *socket, user_data_: &mut [u8], short events_);
    int modify (const ZmqSocketBase *socket, short events_);
    int remove (ZmqSocketBase *socket);

    int add_fd (fd: ZmqFileDesc, user_data_: &mut [u8], short events_);
    int modify_fd (fd: ZmqFileDesc, short events_);
    int remove_fd (ZmqFileDesc fd);
    // Returns the signaler's fd if there is one, otherwise errors.
    int signaler_fd (ZmqFileDesc *fd) const;

    int wait (event_t *events_, n_events_: i32, long timeout);

    int size () const { return static_cast<int> (_items.size ()); };

    //  Return false if object is not a socket.
    bool check_tag () const;

  //
    typedef struct item_t
    {
        ZmqSocketBase *socket;
        ZmqFileDesc fd;
        user_data: *mut c_void;
        short events;
// #if defined ZMQ_POLL_BASED_ON_POLL
        pollfd_index: i32;
// #endif
    } item_t;

    static void zero_trail_events (socket_poller_t::event_t *events_,
                                   n_events_: i32,
                                   found_: i32);
// #if defined ZMQ_POLL_BASED_ON_POLL
    int check_events (socket_poller_t::event_t *events_, n_events_: i32);
#elif defined ZMQ_POLL_BASED_ON_SELECT
    int check_events (socket_poller_t::event_t *events_,
                      n_events_: i32,
                      fd_set &inset_,
                      fd_set &outset_,
                      fd_set &errset_);
// #endif
    static int adjust_timeout (clock_t &clock_,
                               long timeout,
                               u64 &now_,
                               u64 &end_,
                               bool &first_pass_);
    static bool is_socket (const item_t &item, const ZmqSocketBase *socket)
    {
        return item.socket == socket;
    }
    static bool is_fd (const item_t &item, ZmqFileDesc fd)
    {
        return !item.socket && item.fd == fd;
    }

    int rebuild ();

    //  Used to check whether the object is a socket_poller.
    u32 _tag;

    //  Signaler used for thread safe sockets polling
    ZmqSignaler *signaler;

    //  List of sockets
    typedef std::vector<item_t> items_t;
    items_t _items;

    //  Does the pollset needs rebuilding?
    _need_rebuild: bool

    //  Should the signaler be used for the thread safe polling?
    _use_signaler: bool

    //  Size of the pollset
    _pollset_size: i32;

// #if defined ZMQ_POLL_BASED_ON_POLL
    pollfd *_pollfds;
#elif defined ZMQ_POLL_BASED_ON_SELECT
    resizable_optimized_fd_set_t _pollset_in;
    resizable_optimized_fd_set_t _pollset_out;
    resizable_optimized_fd_set_t _pollset_err;
    ZmqFileDesc _max_fd;
// #endif

    // ZMQ_NON_COPYABLE_NOR_MOVABLE (socket_poller_t)
};

static bool is_thread_safe (const ZmqSocketBase &socket)
{
    // do not use getsockopt here, since that would fail during context termination
    return socket.is_thread_safe ();
}

// compare elements to value
template <class It, class T, class Pred>
static It find_if2 (It b_, It e_, const T &value, Pred pred)
{
    for (; b_ != e_; += 1b_) {
        if (pred (*b_, value)) {
            break;
        }
    }
    return b_;
}

socket_poller_t::socket_poller_t () :
    _tag (0xCAFEBABE),
    signaler (null_mut())
// #if defined ZMQ_POLL_BASED_ON_POLL
    ,
    _pollfds (null_mut())
#elif defined ZMQ_POLL_BASED_ON_SELECT
    ,
    _max_fd (0)
// #endif
{
    rebuild ();
}

socket_poller_t::~socket_poller_t ()
{
    //  Mark the socket_poller as dead
    _tag = 0xdeadbeef;

    for (items_t::iterator it = _items.begin (), end = _items.end (); it != end;
         += 1it) {
        // TODO shouldn't this zmq_assert (it->socket->check_tag ()) instead?
        if (it.socket && it.socket.check_tag ()
            && is_thread_safe (*it.socket)) {
            it.socket.remove_signaler (signaler);
        }
    }

    if (signaler != null_mut()) {
        LIBZMQ_DELETE (signaler);
    }

// #if defined ZMQ_POLL_BASED_ON_POLL
    if (_pollfds) {
        free (_pollfds);
        _pollfds = null_mut();
    }
// #endif
}

bool socket_poller_t::check_tag () const
{
    return _tag == 0xCAFEBABE;
}

int socket_poller_t::signaler_fd (ZmqFileDesc *fd) const
{
    if (signaler) {
        *fd = signaler.get_fd ();
        return 0;
    }
    // Only thread-safe socket types are guaranteed to have a signaler.
    errno = EINVAL;
    return -1;
}

int socket_poller_t::add (ZmqSocketBase *socket,
                               user_data_: &mut [u8],
                               short events_)
{
    if (find_if2 (_items.begin (), _items.end (), socket, &is_socket)
        != _items.end ()) {
        errno = EINVAL;
        return -1;
    }

    if (is_thread_safe (*socket)) {
        if (signaler == null_mut()) {
            signaler =  ZmqSignaler ();
            if (!signaler) {
                errno = ENOMEM;
                return -1;
            }
            if (!signaler.valid ()) {
                delete signaler;
                signaler = null_mut();
                errno = EMFILE;
                return -1;
            }
        }

        socket.add_signaler (signaler);
    }

    const item_t item = {
        socket,
        0,
        user_data_,
        events_
// #if defined ZMQ_POLL_BASED_ON_POLL
        ,
        -1
// #endif
    };
    try {
        _items.push_back (item);
    }
    catch (const std::bad_alloc &) {
        errno = ENOMEM;
        return -1;
    }
    _need_rebuild = true;

    return 0;
}

int socket_poller_t::add_fd (fd: ZmqFileDesc, user_data_: &mut [u8], short events_)
{
    if (find_if2 (_items.begin (), _items.end (), fd, &is_fd)
        != _items.end ()) {
        errno = EINVAL;
        return -1;
    }

    const item_t item = {
        null_mut(),
        fd,
        user_data_,
        events_
// #if defined ZMQ_POLL_BASED_ON_POLL
        ,
        -1
// #endif
    };
    try {
        _items.push_back (item);
    }
    catch (const std::bad_alloc &) {
        errno = ENOMEM;
        return -1;
    }
    _need_rebuild = true;

    return 0;
}

int socket_poller_t::modify (const ZmqSocketBase *socket, short events_)
{
    const items_t::iterator it =
      find_if2 (_items.begin (), _items.end (), socket, &is_socket);

    if (it == _items.end ()) {
        errno = EINVAL;
        return -1;
    }

    it.events = events_;
    _need_rebuild = true;

    return 0;
}


int socket_poller_t::modify_fd (fd: ZmqFileDesc, short events_)
{
    const items_t::iterator it =
      find_if2 (_items.begin (), _items.end (), fd, &is_fd);

    if (it == _items.end ()) {
        errno = EINVAL;
        return -1;
    }

    it.events = events_;
    _need_rebuild = true;

    return 0;
}


int socket_poller_t::remove (ZmqSocketBase *socket)
{
    const items_t::iterator it =
      find_if2 (_items.begin (), _items.end (), socket, &is_socket);

    if (it == _items.end ()) {
        errno = EINVAL;
        return -1;
    }

    _items.erase (it);
    _need_rebuild = true;

    if (is_thread_safe (*socket)) {
        socket.remove_signaler (signaler);
    }

    return 0;
}

int socket_poller_t::remove_fd (ZmqFileDesc fd)
{
    const items_t::iterator it =
      find_if2 (_items.begin (), _items.end (), fd, &is_fd);

    if (it == _items.end ()) {
        errno = EINVAL;
        return -1;
    }

    _items.erase (it);
    _need_rebuild = true;

    return 0;
}

int socket_poller_t::rebuild ()
{
    _use_signaler = false;
    _pollset_size = 0;
    _need_rebuild = false;

// #if defined ZMQ_POLL_BASED_ON_POLL

    if (_pollfds) {
        free (_pollfds);
        _pollfds = null_mut();
    }

    for (items_t::iterator it = _items.begin (), end = _items.end (); it != end;
         += 1it) {
        if (it.events) {
            if (it.socket && is_thread_safe (*it.socket)) {
                if (!_use_signaler) {
                    _use_signaler = true;
                    _pollset_size+= 1;
                }
            } else
                _pollset_size+= 1;
        }
    }

    if (_pollset_size == 0)
        return 0;

    _pollfds = static_cast<pollfd *> (malloc (_pollset_size * mem::size_of::<pollfd>()));

    if (!_pollfds) {
        errno = ENOMEM;
        _need_rebuild = true;
        return -1;
    }

    int item_nbr = 0;

    if (_use_signaler) {
        item_nbr = 1;
        _pollfds[0].fd = signaler.get_fd ();
        _pollfds[0].events = POLLIN;
    }

    for (items_t::iterator it = _items.begin (), end = _items.end (); it != end;
         += 1it) {
        if (it.events) {
            if (it.socket) {
                if (!is_thread_safe (*it.socket)) {
                    size_t fd_size = sizeof (ZmqFileDesc);
                    let rc: i32 = it.socket.getsockopt (
                      ZMQ_FD, &_pollfds[item_nbr].fd, &fd_size);
                    // zmq_assert (rc == 0);

                    _pollfds[item_nbr].events = POLLIN;
                    item_nbr+= 1;
                }
            } else {
                _pollfds[item_nbr].fd = it.fd;
                _pollfds[item_nbr].events =
                  (it.events & ZMQ_POLLIN ? POLLIN : 0)
                  | (it.events & ZMQ_POLLOUT ? POLLOUT : 0)
                  | (it.events & ZMQ_POLLPRI ? POLLPRI : 0);
                it.pollfd_index = item_nbr;
                item_nbr+= 1;
            }
        }
    }

#elif defined ZMQ_POLL_BASED_ON_SELECT

    //  Ensure we do not attempt to select () on more than FD_SETSIZE
    //  file descriptors.
    // zmq_assert (_items.size () <= FD_SETSIZE);

    _pollset_in.resize (_items.size ());
    _pollset_out.resize (_items.size ());
    _pollset_err.resize (_items.size ());

    FD_ZERO (_pollset_in.get ());
    FD_ZERO (_pollset_out.get ());
    FD_ZERO (_pollset_err.get ());

    for (items_t::iterator it = _items.begin (), end = _items.end (); it != end;
         += 1it) {
        if (it.socket && is_thread_safe (*it.socket) && it.events) {
            _use_signaler = true;
            FD_SET (signaler.get_fd (), _pollset_in.get ());
            _pollset_size = 1;
            break;
        }
    }

    _max_fd = 0;

    //  Build the fd_sets for passing to select ().
    for (items_t::iterator it = _items.begin (), end = _items.end (); it != end;
         += 1it) {
        if (it.events) {
            //  If the poll item is a 0MQ socket we are interested in input on the
            //  notification file descriptor retrieved by the ZMQ_FD socket option.
            if (it.socket) {
                if (!is_thread_safe (*it.socket)) {
                    ZmqFileDesc notify_fd;
                    size_t fd_size = sizeof (ZmqFileDesc);
                    int rc =
                      it.socket.getsockopt (ZMQ_FD, &notify_fd, &fd_size);
                    // zmq_assert (rc == 0);

                    FD_SET (notify_fd, _pollset_in.get ());
                    if (_max_fd < notify_fd)
                        _max_fd = notify_fd;

                    _pollset_size+= 1;
                }
            }
            //  Else, the poll item is a raw file descriptor. Convert the poll item
            //  events to the appropriate fd_sets.
            else {
                if (it.events & ZMQ_POLLIN)
                    FD_SET (it.fd, _pollset_in.get ());
                if (it.events & ZMQ_POLLOUT)
                    FD_SET (it.fd, _pollset_out.get ());
                if (it.events & ZMQ_POLLERR)
                    FD_SET (it.fd, _pollset_err.get ());
                if (_max_fd < it.fd)
                    _max_fd = it.fd;

                _pollset_size+= 1;
            }
        }
    }

// #endif

    return 0;
}

void socket_poller_t::zero_trail_events (
  socket_poller_t::event_t *events_, n_events_: i32, found_: i32)
{
    for (int i = found_; i < n_events_; += 1i) {
        events_[i].socket = null_mut();
        events_[i].fd = retired_fd;
        events_[i].user_data = null_mut();
        events_[i].events = 0;
    }
}

// #if defined ZMQ_POLL_BASED_ON_POLL
int socket_poller_t::check_events (socket_poller_t::event_t *events_,
                                        n_events_: i32)
#elif defined ZMQ_POLL_BASED_ON_SELECT
int socket_poller_t::check_events (socket_poller_t::event_t *events_,
                                        n_events_: i32,
                                        fd_set &inset_,
                                        fd_set &outset_,
                                        fd_set &errset_)
// #endif
{
    int found = 0;
    for (items_t::iterator it = _items.begin (), end = _items.end ();
         it != end && found < n_events_; += 1it) {
        //  The poll item is a 0MQ socket. Retrieve pending events
        //  using the ZMQ_EVENTS socket option.
        if (it.socket) {
            size_t events_size = mem::size_of::<u32>();
            u32 events;
            if (it.socket.getsockopt (ZMQ_EVENTS, &events, &events_size)
                == -1) {
                return -1;
            }

            if (it.events & events) {
                events_[found].socket = it.socket;
                events_[found].fd = retired_fd;
                events_[found].user_data = it.user_data;
                events_[found].events = it.events & events;
                += 1found;
            }
        }
        //  Else, the poll item is a raw file descriptor, simply convert
        //  the events to ZmqPollItem-style format.
        else if (it.events) {
// #if defined ZMQ_POLL_BASED_ON_POLL
            // zmq_assert (it.pollfd_index >= 0);
            const short revents = _pollfds[it.pollfd_index].revents;
            short events = 0;

            if (revents & POLLIN)
                events |= ZMQ_POLLIN;
            if (revents & POLLOUT)
                events |= ZMQ_POLLOUT;
            if (revents & POLLPRI)
                events |= ZMQ_POLLPRI;
            if (revents & ~(POLLIN | POLLOUT | POLLPRI))
                events |= ZMQ_POLLERR;

#elif defined ZMQ_POLL_BASED_ON_SELECT

            short events = 0;

            if (FD_ISSET (it.fd, &inset_))
                events |= ZMQ_POLLIN;
            if (FD_ISSET (it.fd, &outset_))
                events |= ZMQ_POLLOUT;
            if (FD_ISSET (it.fd, &errset_))
                events |= ZMQ_POLLERR;
// #endif //POLL_SELECT

            if (events) {
                events_[found].socket = null_mut();
                events_[found].fd = it.fd;
                events_[found].user_data = it.user_data;
                events_[found].events = events;
                += 1found;
            }
        }
    }

    return found;
}

//Return 0 if timeout is expired otherwise 1
int socket_poller_t::adjust_timeout (clock_t &clock_,
                                          long timeout,
                                          u64 &now_,
                                          u64 &end_,
                                          bool &first_pass_)
{
    //  If socket_poller_t::timeout is zero, exit immediately whether there
    //  are events or not.
    if (timeout == 0)
        return 0;

    //  At this point we are meant to wait for events but there are none.
    //  If timeout is infinite we can just loop until we get some events.
    if (timeout < 0) {
        if (first_pass_)
            first_pass_ = false;
        return 1;
    }

    //  The timeout is finite and there are no events. In the first pass
    //  we get a timestamp of when the polling have begun. (We assume that
    //  first pass have taken negligible time). We also compute the time
    //  when the polling should time out.
    now_ = clock_.now_ms ();
    if (first_pass_) {
        end_ = now_ + timeout;
        first_pass_ = false;
        return 1;
    }

    //  Find out whether timeout have expired.
    if (now_ >= end_)
        return 0;

    return 1;
}

int socket_poller_t::wait (socket_poller_t::event_t *events_,
                                n_events_: i32,
                                long timeout)
{
    if (_items.is_empty() && timeout < 0) {
        errno = EFAULT;
        return -1;
    }

    if (_need_rebuild) {
        let rc: i32 = rebuild ();
        if (rc == -1)
            return -1;
    }

    if ( (_pollset_size == 0)) {
        if (timeout < 0) {
            // Fail instead of trying to sleep forever
            errno = EFAULT;
            return -1;
        }
        // We'll report an error (timed out) as if the list was non-empty and
        // no event occurred within the specified timeout. Otherwise the caller
        // needs to check the return value AND the event to avoid using the
        // nullified event data.
        errno = EAGAIN;
        if (timeout == 0)
            return -1;
// #if defined ZMQ_HAVE_WINDOWS
        Sleep (timeout > 0 ? timeout : INFINITE);
        return -1;
#elif defined ZMQ_HAVE_ANDROID
        usleep (timeout * 1000);
        return -1;
#elif defined ZMQ_HAVE_OSX
        usleep (timeout * 1000);
        errno = EAGAIN;
        return -1;
#elif defined ZMQ_HAVE_VXWORKS
        struct timespec ns_;
        ns_.tv_sec = timeout / 1000;
        ns_.tv_nsec = timeout % 1000 * 1000000;
        nanosleep (&ns_, 0);
        return -1;
// #else
        usleep (timeout * 1000);
        return -1;
// #endif
    }

// #if defined ZMQ_POLL_BASED_ON_POLL
    clock_t clock;
    u64 now = 0;
    u64 end = 0;

    bool first_pass = true;

    while (true) {
        //  Compute the timeout for the subsequent poll.
        timeout: i32;
        if (first_pass)
            timeout = 0;
        else if (timeout < 0)
            timeout = -1;
        else
            timeout =
              static_cast<int> (std::min<u64> (end - now, INT_MAX));

        //  Wait for events.
        let rc: i32 = poll (_pollfds, _pollset_size, timeout);
        if (rc == -1 && errno == EINTR) {
            return -1;
        }
        // errno_assert (rc >= 0);

        //  Receive the signal from pollfd
        if (_use_signaler && _pollfds[0].revents & POLLIN)
            signaler.recv ();

        //  Check for the events.
        let found: i32 = check_events (events_, n_events_);
        if (found) {
            if (found > 0)
                zero_trail_events (events_, n_events_, found);
            return found;
        }

        //  Adjust timeout or break
        if (adjust_timeout (clock, timeout, now, end, first_pass) == 0)
            break;
    }
    errno = EAGAIN;
    return -1;

#elif defined ZMQ_POLL_BASED_ON_SELECT

    clock_t clock;
    u64 now = 0;
    u64 end = 0;

    bool first_pass = true;

    optimized_fd_set_t inset (_pollset_size);
    optimized_fd_set_t outset (_pollset_size);
    optimized_fd_set_t errset (_pollset_size);

    while (true) {
        //  Compute the timeout for the subsequent poll.
        timeval timeout;
        timeval *ptimeout;
        if (first_pass) {
            timeout.tv_sec = 0;
            timeout.tv_usec = 0;
            ptimeout = &timeout;
        } else if (timeout < 0)
            ptimeout = null_mut();
        else {
            timeout.tv_sec = static_cast<long> ((end - now) / 1000);
            timeout.tv_usec = static_cast<long> ((end - now) % 1000 * 1000);
            ptimeout = &timeout;
        }

        //  Wait for events. Ignore interrupts if there's infinite timeout.
        memcpy (inset.get (), _pollset_in.get (),
                valid_pollset_bytes (*_pollset_in.get ()));
        memcpy (outset.get (), _pollset_out.get (),
                valid_pollset_bytes (*_pollset_out.get ()));
        memcpy (errset.get (), _pollset_err.get (),
                valid_pollset_bytes (*_pollset_err.get ()));
        let rc: i32 = select (static_cast<int> (_max_fd + 1), inset.get (),
                               outset.get (), errset.get (), ptimeout);
// #if defined ZMQ_HAVE_WINDOWS
        if ( (rc == SOCKET_ERROR)) {
            errno = wsa_error_to_errno (WSAGetLastError ());
            wsa_assert (errno == ENOTSOCK);
            return -1;
        }
// #else
        if ( (rc == -1)) {
            // errno_assert (errno == EINTR || errno == EBADF);
            return -1;
        }
// #endif

        if (_use_signaler && FD_ISSET (signaler.get_fd (), inset.get ()))
            signaler.recv ();

        //  Check for the events.
        let found: i32 = check_events (events_, n_events_, *inset.get (),
                                        *outset.get (), *errset.get ());
        if (found) {
            if (found > 0)
                zero_trail_events (events_, n_events_, found);
            return found;
        }

        //  Adjust timeout or break
        if (adjust_timeout (clock, timeout, now, end, first_pass) == 0)
            break;
    }

    errno = EAGAIN;
    return -1;

// #else

    //  Exotic platforms that support neither poll() nor select().
    errno = ENOTSUP;
    return -1;

// #endif
}
