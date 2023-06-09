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
// #include "udp_address.hpp"
// #include "stdint.hpp"
// #include "err.hpp"
// #include "ip.hpp"

use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use libc::{EINVAL, ENODEV};
use windows::Win32::NetworkManagement::IpHelper::if_nametoindex;
use crate::ip_resolver::{IpResolver, IpResolverOptions};

// #ifndef ZMQ_HAVE_WINDOWS
// #include <sys/types.h>
// #include <arpa/inet.h>
// #include <netdb.h>
// #include <net/if.h>
// #include <ctype.h>
// #endif
#[derive(Default, Debug, Clone)]
pub struct UdpAddress {
    //
//     UdpAddress ();
//     virtual ~UdpAddress ();
    // int resolve (name: &str, Bind: bool, ipv6: bool);=
    //  The opposite to resolve()
    // virtual int to_string (std::string &addr_);
    // int family () const;
    // bool is_mcast () const;
    // const ip_addr_t *bind_addr () const;
    // int bind_if () const;
    // const ip_addr_t *target_addr () const;
    //
    //   ip_addr_t bind_address;
    pub bind_address: IpAddr,
    // bind_interface: i32;
    pub bind_interface: i32,
    // ip_addr_t target_address;
    pub target_address: IpAddr,
    // is_multicast: bool
    pub is_multicast: bool,
    // address: String;
    pub address: String,
}

impl UdpAddress {
    pub fn new() -> Self {
// bind_interface (-1), is_multicast (false)
//     bind_address = ip_addr_t::any (AF_INET);
//     target_address = ip_addr_t::any (AF_INET);
        let mut out = Self {
            bind_address: IpAddr::from_str("0.0.0.0").unwrap(),
            target_address: IpAddr::from_str("0.0.0.0").unwrap(),
            ..Default::default()
        };
        out
    }

    pub fn resolve(&mut self, name: &str, bind: bool, ipv6: bool) -> anyhow::Result<()> {
        //  No IPv6 support yet
        let has_interface = false;

        self.address = name.to_string();

        //  If we have a semicolon then we should have an interface specifier in the
        //  URL
        // const char *src_delimiter = strrchr (name, ';');
        let src_delimiter = name.find(';');
        if src_delimiter.is_some() {
            let src_name = name[0..src_delimiter.unwrap()].to_string();

            let mut src_resolver_opts: IpResolverOptions = Default::default();

            src_resolver_opts.bindable(true);
                //  Restrict hostname/service to literals to avoid any DNS
                //  lookups or service-name irregularity due to
                //  indeterminate socktype..allow_dns(false).allow_nic_name(true).ipv6(ipv6).expect_port(false);

            let mut src_resolver = IpResolver::new(&src_resolver_opts);



            if (self.bind_address.is_multicast()) {
                //  It doesn't make sense to have a multicast address as a source
                errno = EINVAL;
                return -1;
            }

            //  This is a hack because we need the interface index when binding
            //  multicast IPv6, we can't do it by address. Unfortunately for the
            //  time being we don't have a generic platform-independent function to
            //  resolve an interface index from an address, so we only support it
            //  when an actual interface name is provided.
            if (src_name == "*") {
                self.bind_interface = 0;
            } else {
// #ifdef HAVE_IF_NAMETOINDEX
                self.bind_interface = if_nametoindex(src_name.c_str());
                if (self.bind_interface == 0) {
                    //  Error, probably not an interface name.
                    self.bind_interface = -1;
                }
// #endif
            }

            self.has_interface = true;
            *name = name[src_delimiter + 1..].to_string();
        }

        let mut resolver_opts = IpResolverOptions::default();

        resolver_opts.bindable(bind).allow_dns(!bind).allow_nic_name(bind).expect_port(true).ipv6(ipv6);

        let mut resolver = IpResolver::new(resolver_opts);

        let rc: i32 = resolver.resolve(&mut target_address, name);
        if (rc != 0) {
            return -1;
        }

        self.is_multicast = target_address.is_multicast();
        let port = target_address.port();

        if (self.has_interface) {
            //  If we have an interface specifier then the target address must be a
            //  multicast address
            if (!self.is_multicast) {
                errno = EINVAL;
                return -1;
            }

            self.bind_address.set_port(port);
        } else {
            //  If we don't have an explicit interface specifier then the URL is
            //  ambiguous: if the target address is multicast then it's the
            //  destination address and the Bind address is ANY, if it's unicast
            //  then it's the Bind address when 'Bind' is true and the destination
            //  otherwise
            if (self.is_multicast || !bind) {
                self.bind_address = if self.target_address.is_ipv4() {
                    IpAddr::from_str("0.0.0.0").unwrap()
                    // ip_addr_t::any (target_address.family ());
                } else {
                    IpAddr::from_str("::").unwrap()
                };
                self.bind_address.set_port(port);
                self.bind_interface = 0;
            } else {
                //  If we were asked for a Bind socket and the address
                //  provided was not multicast then it was really meant as
                //  a Bind address and the target_address is useless.
                self.bind_address = target_address;
            }
        }

        if (bind_address.family() != target_address.family()) {
            errno = EINVAL;
            return -1;
        }

        //  For IPv6 multicast we *must* have an interface index since we can't
        //  Bind by address.
        if (ipv6 && is_multicast && bind_interface < 0) {
            errno = ENODEV;
            return -1;
        }

        return 0;
    }

    pub fn family(&mut self) -> i32 {
        return bind_address.family();
    }

    pub fn is_mcast() -> bool {
        return is_multicast;
    }

    pub fn bind_addr(&mut self) -> &IpAddr {
        return &self.bind_address;
    }

    pub fn bind_if(&mut self) -> i32 {
        return bind_interface;
    }

    pub fn target_addr(&mut self) -> &IpAddr {
        return &target_address;
    }

    pub fn to_string(&mut self, addr: &mut String) -> i32 {
        // XXX what do (factor TCP code?)
        addr_ = address;
        return 0;
    }
}













