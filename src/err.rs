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
// #include "err.hpp"
// #include "macros.hpp"

// const char *errno_to_string (errno_: i32);
// // #if defined __clang__
// #if __has_feature(attribute_analyzer_noreturn)
// void zmq_abort (errmsg_: &str) __attribute__ ((analyzer_noreturn));
// // #else
// void zmq_abort (errmsg_: &str);
// // #endif
// #elif defined __MSCVER__
// __declspec(noreturn) void zmq_abort (errmsg_: &str);
// // #else
// void zmq_abort (errmsg_: &str);
// // #endif
// void print_backtrace ();
// }

// #ifdef ZMQ_HAVE_WINDOWS

// namespace zmq
// {
// const char *wsa_error ();
// const char *
// wsa_error_no (no_: i32,
//               const char *wsae_wouldblock_string_ = "Operation would block");
// void win_error (char *buffer_, buffer_size_: usize);
// int wsa_error_to_errno (errcode_: i32);
// }

//  Provides convenient way to check WSA-style errors on Windows.
// #define wsa_assert(x)                                                          \
// do {                                                                       \
//     if (unlikely (!(x))) {                                                 \
//         const char *errstr = wsa_error ();                            \
//         if (errstr != null_mut()) {                                              \
//             fprintf (stderr, "Assertion failed: %s [%i] (%s:%d)\n",        \
//                      errstr, WSAGetLastError (), __FILE__, __LINE__);      \
//             fflush (stderr);                                               \
//             zmq_abort (errstr);                                       \
//         }                                                                  \
//     }                                                                      \
// } while (false)

//  Provides convenient way to assert on WSA-style errors on Windows.
// #define wsa_assert_no(no)                                                      \
// do {                                                                       \
//     const char *errstr = wsa_error_no (no);                           \
//     if (errstr != null_mut()) {                                                  \
//         fprintf (stderr, "Assertion failed: %s (%s:%d)\n", errstr,         \
//                  __FILE__, __LINE__);                                      \
//         fflush (stderr);                                                   \
//         zmq_abort (errstr);                                           \
//     }                                                                      \
// } while (false)

// Provides convenient way to check GetLastError-style errors on Windows.
// #define win_assert(x)                                                          \
// do {                                                                       \
//     if (unlikely (!(x))) {                                                 \
//         char errstr[256];                                                  \
//         win_error (errstr, 256);                                      \
//         fprintf (stderr, "Assertion failed: %s (%s:%d)\n", errstr,         \
//                  __FILE__, __LINE__);                                      \
//         fflush (stderr);                                                   \
//         zmq_abort (errstr);                                           \
//     }                                                                      \
// } while (false)

// #endif

//  This macro works in exactly the same way as the normal assert. It is used
//  in its stead because standard assert on Win32 in broken - it prints nothing
//  when used within the scope of JNI library.
// #define zmq_assert(x)                                                          \
// do {                                                                       \
//     if (unlikely (!(x))) {                                                 \
//         fprintf (stderr, "Assertion failed: %s (%s:%d)\n", #x, __FILE__,   \
//                  __LINE__);                                                \
//         fflush (stderr);                                                   \
//         zmq_abort (#x);                                               \
//     }                                                                      \
// } while (false)

//  Provides convenient way to check for errno-style errors.
// #define errno_assert(x)                                                        \
// do {                                                                       \
//     if (unlikely (!(x))) {                                                 \
//         const char *errstr = strerror (errno);                             \
//         fprintf (stderr, "%s (%s:%d)\n", errstr, __FILE__, __LINE__);      \
//         fflush (stderr);                                                   \
//         zmq_abort (errstr);                                           \
//     }                                                                      \
// } while (false)

//  Provides convenient way to check for POSIX errors.
// #define posix_assert(x)                                                        \
// do {                                                                       \
//     if (unlikely (x)) {                                                    \
//         const char *errstr = strerror (x);                                 \
//         fprintf (stderr, "%s (%s:%d)\n", errstr, __FILE__, __LINE__);      \
//         fflush (stderr);                                                   \
//         zmq_abort (errstr);                                           \
//     }                                                                      \
// } while (false)

//  Provides convenient way to check for errors from getaddrinfo.
// #define gai_assert(x)                                                          \
// do {                                                                       \
//     if (unlikely (x)) {                                                    \
//         const char *errstr = gai_strerror (x);                             \
//         fprintf (stderr, "%s (%s:%d)\n", errstr, __FILE__, __LINE__);      \
//         fflush (stderr);                                                   \
//         zmq_abort (errstr);                                           \
//     }                                                                      \
// } while (false)

//  Provides convenient way to check whether memory allocation have succeeded.
// #define alloc_assert(x)                                                        \
// do {                                                                       \
//     if (unlikely (!x)) {                                                   \
//         fprintf (stderr, "FATAL ERROR: OUT OF MEMORY (%s:%d)\n", __FILE__, \
//                  __LINE__);                                                \
//         fflush (stderr);                                                   \
//         zmq_abort ("FATAL ERROR: OUT OF MEMORY");                     \
//     }                                                                      \
// } while (false)

// #endif

use libc::{
    EACCES, EADDRINUSE, EADDRNOTAVAIL, EAFNOSUPPORT, EAGAIN, EBADF, EBUSY, ECONNABORTED,
    ECONNREFUSED, ECONNRESET, EFAULT, EHOSTUNREACH, EINTR, EINVAL, EMFILE, EMSGSIZE, ENETDOWN,
    ENETRESET, ENETUNREACH, ENOBUFS, ENOTCONN, ENOTSOCK, EPROTONOSUPPORT, ETIMEDOUT,
};
use windows::Win32::Networking::WinSock::WSA_ERROR;

pub fn errno_to_string(errno_: i32) -> &'static str {
    match (errno_) {
        // #if defined ZMQ_HAVE_WINDOWS
        ENOTSUP => return "Not supported",
        EPROTONOSUPPORT => return "Protocol not supported",
        ENOBUFS => return "No buffer space available",
        ENETDOWN => return "Network is down",
        EADDRINUSE => return "Address in use",
        EADDRNOTAVAIL => return "Address not available",
        ECONNREFUSED => return "Connection refused",
        EINPROGRESS => return "Operation in progress",
        // #endif
        EFSM => return "Operation cannot be accomplished in current state",
        ENOCOMPATPROTO => return "The protocol is not compatible with the socket type",
        ETERM => return "Context was terminated",
        EMTHREAD => return "No thread available",
        EHOSTUNREACH => return "Host unreachable",
        _ => return format!("Unknown error {}", errno_).as_str(),
        // // #if defined _MSC_VER
        // #pragma warning(push)
        // #pragma warning(disable : 4996)
        // // #endif
        //             return strerror (errno_);
        // // #if defined _MSC_VER
        // #pragma warning(pop)
        // // #endif
    }
}

// void zmq_abort (errmsg_: &str)
// {
// // #if defined ZMQ_HAVE_WINDOWS

//     //  Raise STATUS_FATAL_APP_EXIT.
//     ULONG_PTR extra_info[1];
//     extra_info[0] = (ULONG_PTR) errmsg_;
//     RaiseException (0x40000015, EXCEPTION_NONCONTINUABLE, 1, extra_info);
// // #else
//     LIBZMQ_UNUSED (errmsg_);
//     print_backtrace ();
//     abort ();
// // #endif
// }

// #ifdef ZMQ_HAVE_WINDOWS

// const char *wsa_error ()
// {
//     return wsa_error_no (WSAGetLastError (), null_mut());
// }

pub fn wsa_error_no(no_: i32, wsae_wouldblock_string_: &str) -> &str {
    //  TODO:  It seems that list of Windows socket errors is longer than this.
    //         Investigate whether there's a way to convert it into the string
    //         automatically (wsaError->HRESULT->string?).
    match (no_) {
        WSABASEERR => {
            return "No Error";
        }
        WSAEINTR => {
            return "Interrupted system call";
        }
        WSAEBADF => {
            return "Bad file number";
        }
        WSAEACCES => {
            return "Permission denied";
        }
        WSAEFAULT => {
            return "Bad address";
        }
        WSAEINVAL => {
            return "Invalid argument";
        }
        WSAEMFILE => {
            return "Too many open files";
        }
        WSAEWOULDBLOCK => {
            return wsae_wouldblock_string_;
        }
        WSAEINPROGRESS => {
            return "Operation now in progress";
        }
        WSAEALREADY => {
            return "Operation already in progress";
        }
        WSAENOTSOCK => {
            return "Socket operation on non-socket";
        }
        WSAEDESTADDRREQ => {
            return "Destination address required";
        }
        WSAEMSGSIZE => {
            return "Message too long";
        }
        WSAEPROTOTYPE => {
            return "Protocol wrong type for socket";
        }
        WSAENOPROTOOPT => {
            return "Bas protocol option";
        }
        WSAEPROTONOSUPPORT => {
            return "Protocol not supported";
        }
        WSAESOCKTNOSUPPORT => {
            return "Socket type not supported";
        }
        WSAEOPNOTSUPP => {
            return "Operation not supported on socket";
        }
        WSAEPFNOSUPPORT => {
            return "Protocol family not supported";
        }
        WSAEAFNOSUPPORT => {
            return "Address family not supported by protocol family";
        }
        WSAEADDRINUSE => {
            return "Address already in use";
        }
        WSAEADDRNOTAVAIL => {
            return "Can't assign requested address";
        }
        WSAENETDOWN => {
            return "Network is down";
        }
        WSAENETUNREACH => {
            return "Network is unreachable";
        }
        WSAENETRESET => {
            return "Net dropped connection or reset";
        }
        WSAECONNABORTED => {
            return "Software caused connection abort";
        }
        WSAECONNRESET => {
            return "Connection reset by peer";
        }
        WSAENOBUFS => {
            return "No buffer space available";
        }
        WSAEISCONN => {
            return "Socket is already connected";
        }
        WSAENOTCONN => {
            return "Socket is not connected";
        }
        WSAESHUTDOWN => {
            return "Can't send after socket shutdown";
        }
        WSAETOOMANYREFS => {
            return "Too many references can't splice";
        }
        WSAETIMEDOUT => {
            return "Connection timed out";
        }
        WSAECONNREFUSED => {
            return "Connection refused";
        }
        WSAELOOP => {
            return "Too many levels of symbolic links";
        }
        WSAENAMETOOLONG => {
            return "File name too long";
        }
        WSAEHOSTDOWN => {
            return "Host is down";
        }
        WSAEHOSTUNREACH => {
            return "No Route to Host";
        }
        WSAENOTEMPTY => {
            return "Directory not empty";
        }
        WSAEPROCLIM => {
            return "Too many processes";
        }
        WSAEUSERS => {
            return "Too many users";
        }
        WSAEDQUOT => {
            return "Disc Quota Exceeded";
        }
        WSAESTALE => {
            return "Stale NFS file handle";
        }
        WSAEREMOTE => {
            return "Too many levels of remote in path";
        }
        WSASYSNOTREADY => {
            return "Network SubSystem is unavailable";
        }
        WSAVERNOTSUPPORTED => {
            return "WINSOCK DLL Version out of range";
        }
        WSANOTINITIALISED => {
            return "Successful WSASTARTUP not yet performed";
        }
        WSAHOST_NOT_FOUND => {
            return "Host not found";
        }
        WSATRY_AGAIN => {
            return "Non-Authoritative Host not found";
        }
        WSANO_RECOVERY => {
            return "Non-Recoverable errors: FORMERR REFUSED NOTIMP";
        }
        WSANO_DATA => {
            return "Valid name no data record of requested";
        }
        _ => {
            return "error not defined";
        }
    }
}

// void win_error (char *buffer_, buffer_size_: usize)
// {
//     const DWORD errcode = GetLastError ();
// // #if defined _WIN32_WCE
//     DWORD rc = FormatMessageW (
//       FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS, null_mut(), errcode,
//       MAKELANGID (LANG_NEUTRAL, SUBLANG_DEFAULT), (LPWSTR) buffer_,
//       buffer_size_ / mem::size_of::<wchar_t>(), null_mut());
// // #else
//     const DWORD rc = FormatMessageA (
//       FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS, null_mut(), errcode,
//       MAKELANGID (LANG_NEUTRAL, SUBLANG_DEFAULT), buffer_,
//       static_cast<DWORD> (buffer_size_), null_mut());
// // #endif
//     zmq_assert (rc);
// }

pub fn wsa_error_to_errno(errcode_: WSA_ERROR) -> i32 {
    match (errcode_) {
        //  10004 - Interrupted system call.
        WSAEINTR => {
            return EINTR;
        }
        //  10009 - File handle is not valid.
        WSAEBADF => {
            return EBADF;
        }
        //  10013 - Permission denied.
        WSAEACCES => {
            return EACCES;
        }
        //  10014 - Bad address.
        WSAEFAULT => {
            return EFAULT;
        }
        //  10022 - Invalid argument.
        WSAEINVAL => {
            return EINVAL;
        }
        //  10024 - Too many open files.
        WSAEMFILE => {
            return EMFILE;
        }
        //  10035 - Operation would block.
        WSAEWOULDBLOCK => {
            return EBUSY;
        }
        //  10036 - Operation now in progress.
        WSAEINPROGRESS => {
            return EAGAIN;
        }
        //  10037 - Operation already in progress.
        WSAEALREADY => {
            return EAGAIN;
        }
        //  10038 - Socket operation on non-socket.
        WSAENOTSOCK => {
            return ENOTSOCK;
        }
        //  10039 - Destination address required.
        WSAEDESTADDRREQ => {
            return EFAULT;
        }
        //  10040 - Message too long.
        WSAEMSGSIZE => {
            return EMSGSIZE;
        }
        //  10041 - Protocol wrong type for socket.
        WSAEPROTOTYPE => {
            return EFAULT;
        }
        //  10042 - Bad protocol option.
        WSAENOPROTOOPT => {
            return EINVAL;
        }
        //  10043 - Protocol not supported.
        WSAEPROTONOSUPPORT => {
            return EPROTONOSUPPORT;
        }
        //  10044 - Socket type not supported.
        WSAESOCKTNOSUPPORT => {
            return EFAULT;
        }
        //  10045 - Operation not supported on socket.
        WSAEOPNOTSUPP => {
            return EFAULT;
        }
        //  10046 - Protocol family not supported.
        WSAEPFNOSUPPORT => {
            return EPROTONOSUPPORT;
        }
        //  10047 - Address family not supported by protocol family.
        WSAEAFNOSUPPORT => {
            return EAFNOSUPPORT;
        }
        //  10048 - Address already in use.
        WSAEADDRINUSE => {
            return EADDRINUSE;
        }
        //  10049 - Cannot assign requested address.
        WSAEADDRNOTAVAIL => {
            return EADDRNOTAVAIL;
        }
        //  10050 - Network is down.
        WSAENETDOWN => {
            return ENETDOWN;
        }
        //  10051 - Network is unreachable.
        WSAENETUNREACH => {
            return ENETUNREACH;
        }
        //  10052 - Network dropped connection on reset.
        WSAENETRESET => {
            return ENETRESET;
        }
        //  10053 - Software caused connection abort.
        WSAECONNABORTED => {
            return ECONNABORTED;
        }
        //  10054 - Connection reset by peer.
        WSAECONNRESET => {
            return ECONNRESET;
        }
        //  10055 - No buffer space available.
        WSAENOBUFS => {
            return ENOBUFS;
        }
        //  10056 - Socket is already connected.
        WSAEISCONN => {
            return EFAULT;
        }
        //  10057 - Socket is not connected.
        WSAENOTCONN => {
            return ENOTCONN;
        }
        //  10058 - Can't send after socket shutdown.
        WSAESHUTDOWN => {
            return EFAULT;
        }
        //  10059 - Too many references can't splice.
        WSAETOOMANYREFS => {
            return EFAULT;
        }
        //  10060 - Connection timed out.
        WSAETIMEDOUT => {
            return ETIMEDOUT;
        }
        //  10061 - Connection refused.
        WSAECONNREFUSED => {
            return ECONNREFUSED;
        }
        //  10062 - Too many levels of symbolic links.
        WSAELOOP => {
            return EFAULT;
        }
        //  10063 - File name too long.
        WSAENAMETOOLONG => {
            return EFAULT;
        }
        //  10064 - Host is down.
        WSAEHOSTDOWN => {
            return EAGAIN;
        }
        //  10065 - No route to host.
        WSAEHOSTUNREACH => {
            return EHOSTUNREACH;
        }
        //  10066 - Directory not empty.
        WSAENOTEMPTY => {
            return EFAULT;
        }
        //  10067 - Too many processes.
        WSAEPROCLIM => {
            return EFAULT;
        }
        //  10068 - Too many users.
        WSAEUSERS => {
            return EFAULT;
        }
        //  10069 - Disc Quota Exceeded.
        WSAEDQUOT => {
            return EFAULT;
        }
        //  10070 - Stale NFS file handle.
        WSAESTALE => {
            return EFAULT;
        }
        //  10071 - Too many levels of remote in path.
        WSAEREMOTE => {
            return EFAULT;
        }
        //  10091 - Network SubSystem is unavailable.
        WSASYSNOTREADY => {
            return EFAULT;
        }
        //  10092 - WINSOCK DLL Version out of range.
        WSAVERNOTSUPPORTED => {
            return EFAULT;
        }
        //  10093 - Successful WSASTARTUP not yet performed.
        WSANOTINITIALISED => {
            return EFAULT;
        }
        //  11001 - Host not found.
        WSAHOST_NOT_FOUND => {
            return EFAULT;
        }
        //  11002 - Non-Authoritative Host not found.
        WSATRY_AGAIN => {
            return EFAULT;
        }
        //  11003 - Non-Recoverable errors: FORMERR REFUSED NOTIMP.
        WSANO_RECOVERY => {
            return EFAULT;
        }
        //  11004 - Valid name no data record of requested.
        WSANO_DATA => {
            return EFAULT;
        }
        _ => {
            // wsa_assert (false);
            return EFAULT;
        }
    }
    //  Not reachable
}

// #endif

// #if defined(HAVE_LIBUNWIND) && !defined(__SUNPRO_CC)

// #define UNW_LOCAL_ONLY
// #include <libunwind.h>
// #include <dlfcn.h>
// #include <cxxabi.h>
// #include "mutex.hpp"

// void print_backtrace (void)
// {
//     static mutex_t mtx;
//     mtx.lock ();
//     Dl_info dl_info;
//     unw_cursor_t cursor;
//     unw_context_t ctx;
//     unsigned frame_n = 0;

//     unw_getcontext (&ctx);
//     unw_init_local (&cursor, &ctx);

//     while (unw_step (&cursor) > 0) {
//         unw_word_t offset;
//         unw_proc_info_t p_info;
//         pub const unknown: String = String::from("?");
//         const char *file_name;
//         char *demangled_name;
//         char func_name[256] = "";
//         addr: *mut c_void;
//         rc: i32;

//         if (unw_get_proc_info (&cursor, &p_info))
//             break;

//         rc = unw_get_proc_name (&cursor, func_name, 256, &offset);
//         if (rc == -UNW_ENOINFO)
//             memcpy (func_name, unknown, sizeof unknown);

//         addr = (void *) (p_info.start_ip + offset);

//         if (dladdr (addr, &dl_info) && dl_info.dli_fname)
//             file_name = dl_info.dli_fname;
//         else
//             file_name = unknown;

//         demangled_name = abi::__cxa_demangle (func_name, null_mut(), null_mut(), &rc);

//         printf ("#%u  %p in %s (%s+0x%lx)\n", frame_n+= 1, addr, file_name,
//                 rc ? func_name : demangled_name, (unsigned long) offset);
//         free (demangled_name);
//     }
//     puts ("");

//     fflush (stdout);
//     mtx.unlock ();
// }

// #else

// void print_backtrace ()
// {
// }

// #endif
