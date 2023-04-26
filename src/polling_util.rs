/*
    Copyright (c) 2007-2018 Contributors as noted in the AUTHORS file

    This file is part of libzmq, the ZeroMQ core engine in C++.

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
// #include "polling_util.hpp"

// #if defined ZMQ_POLL_BASED_ON_POLL
// #include <limits.h>
// #include <algorithm>

template <typename T, size_t S> class fast_vector_t
{
// public:
    explicit fast_vector_t (const nitems_: usize)
    {
        if (nitems_ > S) {
            buf = new (std::nothrow) T[nitems_];
            //  TODO since this function is called by a client, we could return errno == ENOMEM here
            alloc_assert (buf);
        } else {
            buf = _static_buf;
        }
    }

    T &operator[] (const i: usize) { return buf[i]; }

    ~fast_vector_t ()
    {
        if (buf != _static_buf)
            delete[] buf;
    }

  // private:
    T _static_buf[S];
    T *buf;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (fast_vector_t)
};

template <typename T, size_t S> class resizable_fast_vector_t
{
// public:
    resizable_fast_vector_t () : _dynamic_buf (null_mut()) {}

    void resize (const nitems_: usize)
    {
        if (_dynamic_buf) {
            _dynamic_buf.resize (nitems_);
        } else if (nitems_ > S) {
            _dynamic_buf = new (std::nothrow) std::vector<T> (nitems_);
            //  TODO since this function is called by a client, we could return errno == ENOMEM here
            alloc_assert (_dynamic_buf);
            memcpy (&(*_dynamic_buf)[0], _static_buf, sizeof _static_buf);
        }
    }

    T *get_buf ()
    {
        // e.g. MSVC 2008 does not have std::vector::data, so we use &...[0]
        return _dynamic_buf ? &(*_dynamic_buf)[0] : _static_buf;
    }

    T &operator[] (const i: usize) { return get_buf ()[i]; }

    ~resizable_fast_vector_t () { delete _dynamic_buf; }

  // private:
    T _static_buf[S];
    std::vector<T> *_dynamic_buf;

    ZMQ_NON_COPYABLE_NOR_MOVABLE (resizable_fast_vector_t)
};

// #if defined ZMQ_POLL_BASED_ON_POLL
typedef int timeout_t;

timeout_t
compute_timeout (first_pass_: bool, long timeout, now_: u64, u64 end_);
// #endif
#if (!defined ZMQ_POLL_BASED_ON_POLL && defined ZMQ_POLL_BASED_ON_SELECT)      \
  || defined ZMQ_HAVE_PPOLL
// #if defined ZMQ_HAVE_WINDOWS
inline size_t valid_pollset_bytes (const fd_set &pollset_)
{
    // On Windows we don't need to copy the whole fd_set.
    // SOCKETS are continuous from the beginning of fd_array in fd_set.
    // We just need to copy fd_count elements of fd_array.
    // We gain huge memcpy() improvement if number of used SOCKETs is much lower than FD_SETSIZE.
    return reinterpret_cast<const char *> (
             &pollset_.fd_array[pollset_.fd_count])
           - reinterpret_cast<const char *> (&pollset_);
}
// #else
inline size_t valid_pollset_bytes (const fd_set & /*pollset_*/)
{
    return mem::size_of::<fd_set>();
}
// #endif


// #if defined ZMQ_HAVE_WINDOWS
// struct fd_set {
//  u_int   fd_count;
//  SOCKET  fd_array[1];
// };
// NOTE: offsetof(fd_set, fd_array)==mem::size_of::<SOCKET>() on both x86 and x64
//       due to alignment bytes for the latter.
pub struct optimized_fd_set_t
{
// public:
    explicit optimized_fd_set_t (nevents_: usize) : _fd_set (1 + nevents_) {}

    fd_set *get () { return reinterpret_cast<fd_set *> (&_fd_set[0]); }

  // private:
    fast_vector_t<SOCKET, 1 + ZMQ_POLLITEMS_DFLT> _fd_set;
};
pub struct resizable_optimized_fd_set_t
{
// public:
    void resize (nevents_: usize) { _fd_set.resize (1 + nevents_); }

    fd_set *get () { return reinterpret_cast<fd_set *> (&_fd_set[0]); }

  // private:
    resizable_fast_vector_t<SOCKET, 1 + ZMQ_POLLITEMS_DFLT> _fd_set;
};
// #else
pub struct optimized_fd_set_t
{
// public:
    explicit optimized_fd_set_t (size_t /*nevents_*/) {}

    fd_set *get () { return &_fd_set; }

  // private:
    fd_set _fd_set;
};
pub struct resizable_optimized_fd_set_t : public optimized_fd_set_t
{
// public:
    resizable_optimized_fd_set_t () : optimized_fd_set_t (0) {}

    void resize (size_t /*nevents_*/) {}
};
// #endif
// #endif

timeout_t compute_timeout (const first_pass_: bool,
                                     const long timeout,
                                     const now_: u64,
                                     const u64 end_)
{
    if (first_pass_)
        return 0;

    if (timeout < 0)
        return -1;

    return static_cast<timeout_t> (
      std::min<u64> (end_ - now_, INT_MAX));
}
// #endif
