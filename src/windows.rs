/*
    Copyright (c) 2007-2016 Contributors as noted in the AUTHORS file

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

// #ifndef __ZMQ_WINDOWS_HPP_INCLUDED__
// #define __ZMQ_WINDOWS_HPP_INCLUDED__

// #ifndef _CRT_SECURE_NO_WARNINGS
// #define _CRT_SECURE_NO_WARNINGS
// #endif
// #ifndef NOMINMAX
// #define NOMINMAX // Macros min(a,b) and max(a,b)
// #endif

//  Set target version to Windows Server 2008, Windows Vista or higher.
//  Windows XP (0x0501) is supported but without client & server socket types.
// #if !defined _WIN32_WINNT && !defined ZMQ_HAVE_WINDOWS_UWP
// #define _WIN32_WINNT 0x0600
// #endif

// #ifdef __MINGW32__
//  Require Windows XP or higher with MinGW for getaddrinfo().
#if (_WIN32_WINNT >= 0x0501)
// #else
#error You need at least Windows XP target
// #endif
// #endif

// #include <winsock2.h>
// #include <windows.h>
// #include <mswsock.h>
// #include <iphlpapi.h>

// #if !defined __MINGW32__
// #include <mstcpip.h>
// #endif

//  Workaround missing mstcpip.h in mingw32 (MinGW64 provides this)
//  __MINGW64_VERSION_MAJOR is only defined when using in mingw-w64
// #if defined __MINGW32__ && !defined SIO_KEEPALIVE_VALS                         \
  && !defined __MINGW64_VERSION_MAJOR
struct tcp_keepalive
{
    u_long onoff;
    u_long keepalivetime;
    u_long keepaliveinterval;
};
// #define SIO_KEEPALIVE_VALS _WSAIOW (IOC_VENDOR, 4)
// #endif

// #include <ws2tcpip.h>
// #include <ipexport.h>
// #if !defined _WIN32_WCE
// #include <process.h>
// #endif

// #if defined ZMQ_IOTHREAD_POLLER_USE_POLL || defined ZMQ_POLL_BASED_ON_POLL
static inline int poll (struct pollfd *pfd, unsigned long nfds, timeout: i32)
{
    return WSAPoll (pfd, nfds, timeout);
}
// #endif

//  In MinGW environment AI_NUMERICSERV is not defined.
// #ifndef AI_NUMERICSERV
// #define AI_NUMERICSERV 0x0400
// #endif

//  In MSVC prior to v14, snprintf is not available
//  The closest implementation is the _snprintf_s function
// #if defined(_MSC_VER) && _MSC_VER < 1900
// #define snprintf(buffer_, count, format_, ...)                                \
    _snprintf_s (buffer_, count, _TRUNCATE, format_, __VA_ARGS__)
// #endif

//  Workaround missing struct sockaddr_un in afunix.h.
//  Fix #3949.
// #if defined(ZMQ_HAVE_IPC) && !defined(ZMQ_HAVE_STRUCT_SOCKADDR_UN)
struct sockaddr_un
{
    ADDRESS_FAMILY sun_family; /* AF_UNIX */
    char sun_path[108];        /* pathname */
};
// #endif

// #endif
