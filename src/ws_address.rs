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
// #include <string>
// #include <sstream>

// #include "macros.hpp"
// #include "ws_address.hpp"
// #include "stdint.hpp"
// #include "err.hpp"
// #include "ip.hpp"

// #ifndef ZMQ_HAVE_WINDOWS
// #include <sys/types.h>
// #include <arpa/inet.h>
// #include <netinet/tcp.h>
// #include <net/if.h>
// #include <netdb.h>
// #include <ctype.h>
// #include <unistd.h>
// #include <stdlib.h>
// #endif

// #include <limits.h>
pub struct WsAddress
{
// public:
    WsAddress ();
    WsAddress (const sockaddr *sa_, socklen_t sa_len_);

    //  This function translates textual WS address into an address
    //  structure. If 'local' is true, names are resolved as local interface
    //  names. If it is false, names are resolved as remote hostnames.
    //  If 'ipv6' is true, the name may resolve to IPv6 address.
    int resolve (name: &str, local_: bool, ipv6: bool);

    //  The opposite to resolve()
    int to_string (std::string &addr_) const;

// #if defined ZMQ_HAVE_WINDOWS
    unsigned short family () const;
// #else
    sa_family_t family () const;
// #endif
    const sockaddr *addr () const;
    socklen_t addrlen () const;

    const char *host () const;
    const char *path () const;

  protected:
    ip_addr_t address;

  // private:
    _host: String;
    _path: String;
};

WsAddress::WsAddress ()
{
    memset (&address, 0, mem::size_of::<address>());
}

WsAddress::WsAddress (const sockaddr *sa_, socklen_t sa_len_)
{
    zmq_assert (sa_ && sa_len_ > 0);

    memset (&address, 0, mem::size_of::<address>());
    if (sa_.sa_family == AF_INET
        && sa_len_ >= static_cast<socklen_t> (sizeof (address.ipv4)))
        memcpy (&address.ipv4, sa_, sizeof (address.ipv4));
    else if (sa_.sa_family == AF_INET6
             && sa_len_ >= static_cast<socklen_t> (sizeof (address.ipv6)))
        memcpy (&address.ipv6, sa_, sizeof (address.ipv6));

    _path = std::string ("");

    char hbuf[NI_MAXHOST];
    let rc: i32 = getnameinfo (addr (), addrlen (), hbuf, mem::size_of::<hbuf>(), null_mut(),
                                0, NI_NUMERICHOST);
    if (rc != 0) {
        _host = std::string ("localhost");
        return;
    }

    std::ostringstream os;

    if (address.family () == AF_INET6)
        os << std::string ("[");

    os << std::string (hbuf);

    if (address.family () == AF_INET6)
        os << std::string ("]");

    _host = os.str ();
}

int WsAddress::resolve (name: &str, local_: bool, ipv6: bool)
{
    //  find the host part, It's important to use str*r*chr to only get
    //  the latest colon since IPv6 addresses use colons as delemiters.
    const char *delim = strrchr (name, ':');
    if (delim == null_mut()) {
        errno = EINVAL;
        return -1;
    }
    _host = std::string (name, delim - name);

    // find the path part, which is optional
    delim = strrchr (name, '/');
    host_name: String;
    if (delim) {
        _path = std::string (delim);
        // remove the path, otherwise resolving the port will fail with wildcard
        host_name = std::string (name, delim - name);
    } else {
        _path = std::string ("/");
        host_name = name;
    }

    IpResolverOptions resolver_opts;
    resolver_opts.bindable (local_)
      .allow_dns (!local_)
      .allow_nic_name (local_)
      .ipv6 (ipv6)
      .allow_path (true)
      .expect_port (true);

    IpResolver resolver (resolver_opts);

    return resolver.resolve (&address, host_name.c_str ());
}

int WsAddress::to_string (std::string &addr_) const
{
    std::ostringstream os;
    os << std::string ("ws://") << host () << std::string (":")
       << address.port () << _path;
    addr_ = os.str ();

    return 0;
}

const sockaddr *WsAddress::addr () const
{
    return address.as_sockaddr ();
}

socklen_t WsAddress::addrlen () const
{
    return address.sockaddr_len ();
}

const char *WsAddress::host () const
{
    return _host;
}

const char *WsAddress::path () const
{
    return _path;
}

// #if defined ZMQ_HAVE_WINDOWS
unsigned short WsAddress::family () const
// #else
sa_family_t WsAddress::family () const
// #endif
{
    return address.family ();
}
