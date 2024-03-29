/*
Copyright (c) 2018 Contributors as noted in the AUTHORS file

This file is part of 0MQ.

0MQ is free software; you can redistribute it and/or modify it under
the terms of the GNU Lesser General Public License as published by
the Free Software Foundation; either version 3 of the License, or
(at your option) any later version.

0MQ is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Lesser General Public License for more details.

You should have received a copy of the GNU Lesser General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

// #include <unity.h>
// #include "../src/macros.hpp"
// #include "../tests/testutil.hpp"
// #include "../tests/testutil_unity.hpp"
// #include "../unittests/unittest_resolver_common.hpp"

// #include <ip_resolver.hpp>
// #include <ip.hpp>

// #ifndef _WIN32
// #include <sys/types.h>
// #include <sys/socket.h>
// #include <netdb.h>
// #endif

void setUp ()
{
}

void tearDown ()
{
}
pub struct test_ip_resolver_t  : public IpResolver
{
//
    test_ip_resolver_t (IpResolverOptions opts_) :
        IpResolver (opts_)
    {
    }


    struct dns_lut_t
    {
        const char *hostname;
        const char *ipv4;
        const char *ipv6;
    };

    int do_getaddrinfo (node_: *const c_char,
                        service_: *const c_char,
                        const struct addrinfo *hints_,
                        struct addrinfo **res_)
    {
        static const struct dns_lut_t dns_lut[] = {
          {"ip.zeromq.org", "10.100.0.1", "fdf5:d058:d656::1"},
          {"ipv4only.zeromq.org", "10.100.0.2", "::ffff:10.100.0.2"},
          {"ipv6only.zeromq.org", null_mut(), "fdf5:d058:d656::2"},
        };
        unsigned lut_len = mem::size_of::<dns_lut>() / sizeof (dns_lut[0]);
        struct addrinfo ai;

        TEST_ASSERT_NULL (service_);

        bool ipv6 = (hints_.ai_family == AF_INET6);
        bool no_dns = (hints_.ai_flags & AI_NUMERICHOST) != 0;
        const char *ip = null_mut();

        if (!no_dns) {
            for (unsigned i = 0; i < lut_len; i+= 1) {
                if (strcmp (dns_lut[i].hostname, node_) == 0) {
                    if (ipv6) {
                        ip = dns_lut[i].ipv6;
                    } else {
                        ip = dns_lut[i].ipv4;

                        if (ip == null_mut()) {
                            //  No address associated with NAME
                            return EAI_NODATA;
                        }
                    }
                }
            }
        }

        if (ip == null_mut()) {
            //  No entry for 'node_' found in the LUT (or DNS is
            //  forbidden), assume that it's a numeric IP address
            ip = node_;
        }

        //  Call the real getaddrinfo implementation, making sure that it won't
        //  attempt to resolve using DNS
        ai = *hints_;
        ai.ai_flags |= AI_NUMERICHOST;

        return IpResolver::do_getaddrinfo (ip, null_mut(), &ai, res_);
    }

    unsigned int do_if_nametoindex (ifname_: &str)
    {
        static const char *dummy_interfaces[] = {
          "lo0",
          "eth0",
          "eth1",
        };
        unsigned lut_len =
          mem::size_of::<dummy_interfaces>() / sizeof (dummy_interfaces[0]);

        for (unsigned i = 0; i < lut_len; i+= 1) {
            if (strcmp (dummy_interfaces[i], ifname_) == 0) {
                //  The dummy index will be the position in the array + 1 (0 is
                //  invalid)
                return i + 1;
            }
        }

        //  Not found
        return 0;
    }
};

//  Attempt a resolution and test the results. If 'expected_addr_' is NULL
//  assume that the resolution is meant to fail.
//
//  On windows we can receive an IPv4 address even when an IPv6 is requested, if
//  we're in this situation then we compare to 'expected_addr_v4_failover_'
//  instead.
static void test_resolve (IpResolverOptions opts_,
                          name: *const c_char,
                          expected_addr_: *const c_char,
                          uint16_t expected_port_ = 0,
                          uint16_t expected_zone_ = 0,
                          const char *expected_addr_v4_failover_ = null_mut())
{
    ip_addr_t addr;
    int family = opts_.ipv6 () ? AF_INET6 : AF_INET;

    if (family == AF_INET6 && !is_ipv6_available ()) {
        TEST_IGNORE_MESSAGE ("ipv6 is not available");
    }

    //  Generate an invalid but well-defined 'ip_addr_t'. Avoids testing
    //  uninitialized values if the code is buggy.
    memset (&addr, 0xba, mem::size_of::<addr>());

    test_ip_resolver_t resolver (opts_);

    int rc = resolver.resolve (&addr, name);

    if (expected_addr_ == null_mut()) {
        // TODO also check the expected errno
        TEST_ASSERT_EQUAL (-1, rc);
        return;
    }
    TEST_ASSERT_SUCCESS_ERRNO (rc);


    validate_address (family, &addr, expected_addr_, expected_port_,
                      expected_zone_, expected_addr_v4_failover_);
}

// Helper macro to define the v4/v6 function pairs
// #define MAKE_TEST_V4V6(_test)                                                  \
    static void _test##_ipv4 () { _test (false); }                             \
                                                                               \
    static void _test##_ipv6 () { _test (true); }

static void test_bind_any (ipv6: bool)
{
    IpResolverOptions resolver_opts;

    resolver_opts.bindable (true).expect_port (true).ipv6 (ipv6);

    const char *expected = ipv6 ? "::" : "0.0.0.0";
    test_resolve (resolver_opts, "*:*", expected, 0);
}
MAKE_TEST_V4V6 (test_bind_any)

static void test_bind_any_port0 (ipv6: bool)
{
    IpResolverOptions resolver_opts;

    resolver_opts.bindable (true).expect_port (true).ipv6 (ipv6);

    //  Should be equivalent to "*:*"
    const char *expected = ipv6 ? "::" : "0.0.0.0";
    test_resolve (resolver_opts, "*:0", expected, 0);
}
MAKE_TEST_V4V6 (test_bind_any_port0)

static void test_nobind_any (ipv6: bool)
{
    IpResolverOptions resolver_opts;

    resolver_opts.expect_port (true).ipv6 (ipv6);

    //  Wildcard should be rejected if we're not looking for a
    //  bindable address
    test_resolve (resolver_opts, "*:*", null_mut());
}
MAKE_TEST_V4V6 (test_nobind_any)

static void test_nobind_any_port (ipv6: bool)
{
    IpResolverOptions resolver_opts;

    resolver_opts.expect_port (true).ipv6 (ipv6);

    //  Wildcard should be rejected if we're not looking for a
    //  bindable address
    test_resolve (resolver_opts, "*:1234", null_mut());
}
MAKE_TEST_V4V6 (test_nobind_any_port)

static void test_nobind_addr_anyport (ipv6: bool)
{
    IpResolverOptions resolver_opts;

    resolver_opts.expect_port (true).ipv6 (ipv6);

    //  Wildcard port should be rejected for non-bindable addresses
    test_resolve (resolver_opts, "127.0.0.1:*", null_mut());
}
MAKE_TEST_V4V6 (test_nobind_addr_anyport)

static void test_nobind_addr_port0 (ipv6: bool)
{
    IpResolverOptions resolver_opts;

    resolver_opts.expect_port (true).ipv6 (ipv6);

    //  Connecting to port 0 is allowed, although it might not be massively
    //  useful
    const char *expected = ipv6 ? "::ffff:127.0.0.1" : "127.0.0.1";
    const char *fallback = ipv6 ? "127.0.0.1" : null_mut();
    test_resolve (resolver_opts, "127.0.0.1:0", expected, 0, 0, fallback);
}
MAKE_TEST_V4V6 (test_nobind_addr_port0)

static void test_parse_ipv4_simple ()
{
    IpResolverOptions resolver_opts;

    test_resolve (resolver_opts, "1.2.128.129", "1.2.128.129");
}

static void test_parse_ipv4_zero ()
{
    IpResolverOptions resolver_opts;

    test_resolve (resolver_opts, "0.0.0.0", "0.0.0.0");
}

static void test_parse_ipv4_max ()
{
    IpResolverOptions resolver_opts;

    test_resolve (resolver_opts, "255.255.255.255", "255.255.255.255");
}

static void test_parse_ipv4_brackets ()
{
    IpResolverOptions resolver_opts;

    //  Not particularly useful, but valid
    test_resolve (resolver_opts, "[1.2.128.129]", "1.2.128.129");
}

static void test_parse_ipv4_brackets_missingl ()
{
    IpResolverOptions resolver_opts;

    test_resolve (resolver_opts, "1.2.128.129]", null_mut());
}

static void test_parse_ipv4_brackets_missingr ()
{
    IpResolverOptions resolver_opts;

    test_resolve (resolver_opts, "[1.2.128.129", null_mut());
}

static void test_parse_ipv4_brackets_bad ()
{
    IpResolverOptions resolver_opts;

    test_resolve (resolver_opts, "[1.2.128].129", null_mut());
}

static void test_parse_ipv4_reject_port ()
{
    IpResolverOptions resolver_opts;

    //  No port expected, should be rejected
    test_resolve (resolver_opts, "1.2.128.129:123", null_mut());
}

static void test_parse_ipv4_reject_any ()
{
    IpResolverOptions resolver_opts;

    //  No port expected, should be rejected
    test_resolve (resolver_opts, "1.2.128.129:*", null_mut());
}

static void test_parse_ipv4_reject_ipv6 ()
{
    IpResolverOptions resolver_opts;

    //  No port expected, should be rejected
    test_resolve (resolver_opts, "::1", null_mut());
}

static void test_parse_ipv4_port ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.expect_port (true);

    test_resolve (resolver_opts, "1.2.128.129:123", "1.2.128.129", 123);
}

static void test_parse_ipv4_port0 ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.expect_port (true);

    //  Port 0 is accepted and is equivalent to *
    test_resolve (resolver_opts, "1.2.128.129:0", "1.2.128.129", 0);
}

static void test_parse_ipv4_port_garbage ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.expect_port (true);

    //  The code doesn't validate that the port doesn't contain garbage
    test_resolve (resolver_opts, "1.2.3.4:567bad", "1.2.3.4", 567);
}

static void test_parse_ipv4_port_missing ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.expect_port (true);

    test_resolve (resolver_opts, "1.2.3.4", null_mut());
}

static void test_parse_ipv4_port_bad ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.expect_port (true);

    test_resolve (resolver_opts, "1.2.3.4:bad", null_mut());
}

static void test_parse_ipv4_port_brackets ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.expect_port (true);

    test_resolve (resolver_opts, "[192.168.1.1]:5555", "192.168.1.1", 5555);
}

static void test_parse_ipv4_port_brackets_bad ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.expect_port (true);

    test_resolve (resolver_opts, "[192.168.1.1:]5555", null_mut());
}

static void test_parse_ipv4_port_brackets_bad2 ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.expect_port (true);

    test_resolve (resolver_opts, "[192.168.1.1:5555]", null_mut());
}

static void test_parse_ipv4_wild_brackets_bad ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.expect_port (true);

    test_resolve (resolver_opts, "[192.168.1.1:*]", null_mut());
}

static void test_parse_ipv4_port_ipv6_reject ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.expect_port (true);

    test_resolve (resolver_opts, "[::1]:1234", null_mut());
}

static void test_parse_ipv6_simple ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.ipv6 (true);

    test_resolve (resolver_opts, "::1", "::1");
}

static void test_parse_ipv6_simple2 ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.ipv6 (true);

    test_resolve (resolver_opts, "abcd:1234::1:0:234", "abcd:1234::1:0:234");
}

static void test_parse_ipv6_zero ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.ipv6 (true);

    test_resolve (resolver_opts, "::", "::");
}

static void test_parse_ipv6_max ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.ipv6 (true);

    test_resolve (resolver_opts, "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff",
                  "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff");
}

static void test_parse_ipv6_brackets ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.ipv6 (true);

    test_resolve (resolver_opts, "[::1]", "::1");
}

static void test_parse_ipv6_brackets_missingl ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.ipv6 (true);

    test_resolve (resolver_opts, "::1]", null_mut());
}

static void test_parse_ipv6_brackets_missingr ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.ipv6 (true);

    test_resolve (resolver_opts, "[::1", null_mut());
}

static void test_parse_ipv6_brackets_bad ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.ipv6 (true);

    test_resolve (resolver_opts, "[abcd:1234::1:]0:234", null_mut());
}

static void test_parse_ipv6_port ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.ipv6 (true).expect_port (true);

    test_resolve (resolver_opts, "[1234::1]:80", "1234::1", 80);
}

static void test_parse_ipv6_port_any ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.ipv6 (true).expect_port (true).bindable (true);

    test_resolve (resolver_opts, "[1234::1]:*", "1234::1", 0);
}

static void test_parse_ipv6_port_nobrackets ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.ipv6 (true).expect_port (true);

    //  Should this be allowed? Seems error-prone but so far ZMQ accepts it.
    test_resolve (resolver_opts, "abcd:1234::1:0:234:123", "abcd:1234::1:0:234",
                  123);
}

static void test_parse_ipv4_in_ipv6 ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.ipv6 (true);

    //  Parsing IPv4 should also work if an IPv6 is requested, it returns an
    //  IPv6 with the IPv4 address embedded (except sometimes on Windows where
    //  we end up with an IPv4 anyway)
    test_resolve (resolver_opts, "11.22.33.44", "::ffff:11.22.33.44", 0, 0,
                  "11.22.33.44");
}

static void test_parse_ipv4_in_ipv6_port ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.ipv6 (true).expect_port (true);

    test_resolve (resolver_opts, "11.22.33.44:55", "::ffff:11.22.33.44", 55, 0,
                  "11.22.33.44");
}

static void test_parse_ipv6_scope_int ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.ipv6 (true);

    test_resolve (resolver_opts, "3000:4:5::1:234%2", "3000:4:5::1:234", 0, 2);
}

static void test_parse_ipv6_scope_zero ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.ipv6 (true);

    test_resolve (resolver_opts, "3000:4:5::1:234%0", null_mut());
}

static void test_parse_ipv6_scope_int_port ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.expect_port (true).ipv6 (true);

    test_resolve (resolver_opts, "3000:4:5::1:234%2:1111", "3000:4:5::1:234",
                  1111, 2);
}

static void test_parse_ipv6_scope_if ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.ipv6 (true);

    test_resolve (resolver_opts, "3000:4:5::1:234%eth1", "3000:4:5::1:234", 0,
                  3);
}

static void test_parse_ipv6_scope_if_port ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.expect_port (true).ipv6 (true);

    test_resolve (resolver_opts, "3000:4:5::1:234%eth0:8080", "3000:4:5::1:234",
                  8080, 2);
}

static void test_parse_ipv6_scope_if_port_brackets ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.expect_port (true).ipv6 (true);

    test_resolve (resolver_opts, "[3000:4:5::1:234%eth0]:8080",
                  "3000:4:5::1:234", 8080, 2);
}

static void test_parse_ipv6_scope_badif ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.ipv6 (true);

    test_resolve (resolver_opts, "3000:4:5::1:234%bad0", null_mut());
}

static void test_dns_ipv4_simple ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.allow_dns (true);

    test_resolve (resolver_opts, "ip.zeromq.org", "10.100.0.1");
}

static void test_dns_ipv4_only ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.allow_dns (true);

    test_resolve (resolver_opts, "ipv4only.zeromq.org", "10.100.0.2");
}

static void test_dns_ipv4_invalid ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.allow_dns (true);

    test_resolve (resolver_opts, "invalid.zeromq.org", null_mut());
}

static void test_dns_ipv4_ipv6 ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.allow_dns (true);

    test_resolve (resolver_opts, "ipv6only.zeromq.org", null_mut());
}

static void test_dns_ipv4_numeric ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.allow_dns (true);

    //  Numeric IPs should still work
    test_resolve (resolver_opts, "5.4.3.2", "5.4.3.2");
}

static void test_dns_ipv4_port ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.expect_port (true).allow_dns (true);

    test_resolve (resolver_opts, "ip.zeromq.org:1234", "10.100.0.1", 1234);
}

static void test_dns_ipv6_simple ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.ipv6 (true).allow_dns (true);

    test_resolve (resolver_opts, "ip.zeromq.org", "fdf5:d058:d656::1");
}

static void test_dns_ipv6_only ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.ipv6 (true).allow_dns (true);

    test_resolve (resolver_opts, "ipv6only.zeromq.org", "fdf5:d058:d656::2");
}

static void test_dns_ipv6_invalid ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.ipv6 (true).allow_dns (true);

    test_resolve (resolver_opts, "invalid.zeromq.org", null_mut());
}

static void test_dns_ipv6_ipv4 ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.ipv6 (true).allow_dns (true);

    //  If a host doesn't have an IPv6 then it should resolve as an embedded v4
    //  address in an IPv6
    test_resolve (resolver_opts, "ipv4only.zeromq.org", "::ffff:10.100.0.2");
}

static void test_dns_ipv6_numeric ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.ipv6 (true).allow_dns (true);

    //  Numeric IPs should still work
    test_resolve (resolver_opts, "fdf5:d058:d656::1", "fdf5:d058:d656::1");
}

static void test_dns_ipv6_port ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.ipv6 (true).expect_port (true).allow_dns (true);

    test_resolve (resolver_opts, "ip.zeromq.org:1234", "fdf5:d058:d656::1",
                  1234);
}

void test_dns_brackets ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.allow_dns (true);

    test_resolve (resolver_opts, "[ip.zeromq.org]", "10.100.0.1");
}

void test_dns_brackets_bad ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.allow_dns (true);

    test_resolve (resolver_opts, "[ip.zeromq].org", null_mut());
}

void test_dns_brackets_port ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.allow_dns (true);

    test_resolve (resolver_opts, "[ip.zeromq.org]:22", "10.100.0.1", 22);
}

void test_dns_brackets_port_bad ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.allow_dns (true);

    test_resolve (resolver_opts, "[ip.zeromq.org:22]", null_mut());
}

void test_dns_deny (ipv6: bool)
{
    IpResolverOptions resolver_opts;

    resolver_opts.allow_dns (false).ipv6 (ipv6);

    //  DNS resolution shouldn't work when disallowed
    test_resolve (resolver_opts, "ip.zeromq.org", null_mut());
}
MAKE_TEST_V4V6 (test_dns_deny)

void test_dns_ipv6_scope ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.allow_dns (true).ipv6 (true);

    //  Not sure if that's very useful but you could technically add a scope
    //  identifier to a hostname
    test_resolve (resolver_opts, "ip.zeromq.org%lo0", "fdf5:d058:d656::1", 0,
                  1);
}

void test_dns_ipv6_scope_port ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.allow_dns (true).expect_port (true).ipv6 (true);

    //  Not sure if that's very useful but you could technically add a scope
    //  identifier to a hostname
    test_resolve (resolver_opts, "ip.zeromq.org%lo0:4444", "fdf5:d058:d656::1",
                  4444, 1);
}

void test_dns_ipv6_scope_port_brackets ()
{
    IpResolverOptions resolver_opts;

    resolver_opts.allow_dns (true).expect_port (true).ipv6 (true);

    test_resolve (resolver_opts, "[ip.zeromq.org%lo0]:4444",
                  "fdf5:d058:d656::1", 4444, 1);
}

static void test_addr (family_: i32, addr_: *const c_char, multicast_: bool)
{
    if (family_ == AF_INET6 && !is_ipv6_available ()) {
        TEST_IGNORE_MESSAGE ("ipv6 is not available");
    }

    IpResolverOptions resolver_opts;

    resolver_opts.ipv6 (family_ == AF_INET6);

    test_ip_resolver_t resolver (resolver_opts);
    ip_addr_t addr;

    TEST_ASSERT_SUCCESS_ERRNO (resolver.resolve (&addr, addr_));

    TEST_ASSERT_EQUAL (family_, addr.family ());
    TEST_ASSERT_EQUAL (multicast_, addr.is_multicast ());
}

static void test_addr_unicast_ipv4 ()
{
    test_addr (AF_INET, "1.2.3.4", false);
}

static void test_addr_unicast_ipv6 ()
{
    test_addr (AF_INET6, "abcd::1", false);
}

static void test_addr_multicast_ipv4 ()
{
    test_addr (AF_INET, "230.1.2.3", true);
}

static void test_addr_multicast_ipv6 ()
{
    test_addr (AF_INET6, "ffab::1234", true);
}

static void test_addr_multicast_ipv4_min ()
{
    test_addr (AF_INET, "224.0.0.0", true);
}

static void test_addr_multicast_ipv6_min ()
{
    test_addr (AF_INET6, "ff00::", true);
}

static void test_addr_multicast_ipv4_max ()
{
    test_addr (AF_INET, "239.255.255.255", true);
}

static void test_addr_multicast_ipv6_max ()
{
    test_addr (AF_INET6, "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff", true);
}

static void test_addr_multicast_ipv4_sub ()
{
    test_addr (AF_INET, "223.255.255.255", false);
}

static void test_addr_multicast_ipv6_sub ()
{
    test_addr (AF_INET6, "feff:ffff:ffff:ffff:ffff:ffff:ffff:ffff", false);
}

static void test_addr_multicast_ipv4_over ()
{
    test_addr (AF_INET, "240.0.0.0", false);
}

int main (void)
{
    initialize_network ();
    setup_test_environment ();

    UNITY_BEGIN ();

    RUN_TEST (test_bind_any_ipv4);
    RUN_TEST (test_bind_any_ipv6);
    RUN_TEST (test_bind_any_port0_ipv4);
    RUN_TEST (test_bind_any_port0_ipv6);
    RUN_TEST (test_nobind_any_ipv4);
    RUN_TEST (test_nobind_any_ipv6);
    RUN_TEST (test_nobind_any_port_ipv4);
    RUN_TEST (test_nobind_any_port_ipv6);
    RUN_TEST (test_nobind_addr_anyport_ipv4);
    RUN_TEST (test_nobind_addr_anyport_ipv6);
    RUN_TEST (test_nobind_addr_port0_ipv4);
    RUN_TEST (test_nobind_addr_port0_ipv6);
    RUN_TEST (test_parse_ipv4_simple);
    RUN_TEST (test_parse_ipv4_zero);
    RUN_TEST (test_parse_ipv4_max);
    RUN_TEST (test_parse_ipv4_brackets);
    RUN_TEST (test_parse_ipv4_brackets_missingl);
    RUN_TEST (test_parse_ipv4_brackets_missingr);
    RUN_TEST (test_parse_ipv4_brackets_bad);
    RUN_TEST (test_parse_ipv4_reject_port);
    RUN_TEST (test_parse_ipv4_reject_any);
    RUN_TEST (test_parse_ipv4_reject_ipv6);
    RUN_TEST (test_parse_ipv4_port);
    RUN_TEST (test_parse_ipv4_port0);
    RUN_TEST (test_parse_ipv4_port_garbage);
    RUN_TEST (test_parse_ipv4_port_missing);
    RUN_TEST (test_parse_ipv4_port_bad);
    RUN_TEST (test_parse_ipv4_port_brackets);
    RUN_TEST (test_parse_ipv4_port_brackets_bad);
    RUN_TEST (test_parse_ipv4_port_brackets_bad2);
    RUN_TEST (test_parse_ipv4_wild_brackets_bad);
    RUN_TEST (test_parse_ipv4_port_ipv6_reject);
    RUN_TEST (test_parse_ipv6_simple);
    RUN_TEST (test_parse_ipv6_simple2);
    RUN_TEST (test_parse_ipv6_zero);
    RUN_TEST (test_parse_ipv6_max);
    RUN_TEST (test_parse_ipv6_brackets);
    RUN_TEST (test_parse_ipv6_brackets_missingl);
    RUN_TEST (test_parse_ipv6_brackets_missingr);
    RUN_TEST (test_parse_ipv6_brackets_bad);
    RUN_TEST (test_parse_ipv6_port);
    RUN_TEST (test_parse_ipv6_port_any);
    RUN_TEST (test_parse_ipv6_port_nobrackets);
    RUN_TEST (test_parse_ipv4_in_ipv6);
    RUN_TEST (test_parse_ipv4_in_ipv6_port);
    RUN_TEST (test_parse_ipv6_scope_int);
    RUN_TEST (test_parse_ipv6_scope_zero);
    RUN_TEST (test_parse_ipv6_scope_int_port);
    RUN_TEST (test_parse_ipv6_scope_if);
    RUN_TEST (test_parse_ipv6_scope_if_port);
    RUN_TEST (test_parse_ipv6_scope_if_port_brackets);
    RUN_TEST (test_parse_ipv6_scope_badif);
    RUN_TEST (test_dns_ipv4_simple);
    RUN_TEST (test_dns_ipv4_only);
    RUN_TEST (test_dns_ipv4_invalid);
    RUN_TEST (test_dns_ipv4_ipv6);
    RUN_TEST (test_dns_ipv4_numeric);
    RUN_TEST (test_dns_ipv4_port);
    RUN_TEST (test_dns_ipv6_simple);
    RUN_TEST (test_dns_ipv6_only);
    RUN_TEST (test_dns_ipv6_invalid);
    RUN_TEST (test_dns_ipv6_ipv4);
    RUN_TEST (test_dns_ipv6_numeric);
    RUN_TEST (test_dns_ipv6_port);
    RUN_TEST (test_dns_brackets);
    RUN_TEST (test_dns_brackets_bad);
    RUN_TEST (test_dns_deny_ipv4);
    RUN_TEST (test_dns_deny_ipv6);
    RUN_TEST (test_dns_ipv6_scope);
    RUN_TEST (test_dns_ipv6_scope_port);
    RUN_TEST (test_dns_ipv6_scope_port_brackets);
    RUN_TEST (test_addr_unicast_ipv4);
    RUN_TEST (test_addr_unicast_ipv6);
    RUN_TEST (test_addr_multicast_ipv4);
    RUN_TEST (test_addr_multicast_ipv6);
    RUN_TEST (test_addr_multicast_ipv4_min);
    RUN_TEST (test_addr_multicast_ipv6_min);
    RUN_TEST (test_addr_multicast_ipv4_max);
    RUN_TEST (test_addr_multicast_ipv6_max);
    RUN_TEST (test_addr_multicast_ipv4_sub);
    RUN_TEST (test_addr_multicast_ipv6_sub);
    RUN_TEST (test_addr_multicast_ipv4_over);

    shutdown_network ();

    return UNITY_END ();
}
