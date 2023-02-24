use std::net::SocketAddr;
use std::ptr::null_mut;
use crate::address_family::{AF_INET, AF_INET6};
use crate::ip_resolver::{IpResolver, IpResolverOptions};
use libc;
use libc::EINVAL;

pub const ipv4_prefix: &'static str = "tcp://";
pub const ipv4_suffix: &'static str = ":";
pub const ipv6_prefix: &'static str = "tcp://[";
pub const ipv6_suffix: &'static str = "]:";

#[derive(Default, Debug, Clone)]
pub struct TcpAddress {
    pub family: u16,
    pub addr: SocketAddr,
    pub src_addr: Option<SocketAddr>,
    pub has_src_addr: bool,
}

impl TcpAddress {
    // TcpAddress ();
    // TcpAddress (const sockaddr *sa_, socklen_t sa_len_);
    pub fn new(sa_: &SocketAddr) -> Self {
        Self {
            family: if sa_.is_ipv4() { AF_INET } else { AF_INET6 },
            addr: sa.clone(),
            src_addr: None,
            has_src_addr: false,
        }
    }

    //  This function translates textual TCP address into an address
    //  structure. If 'local' is true, names are resolved as local interface
    //  names. If it is false, names are resolved as remote hostnames.
    //  If 'ipv6' is true, the name may resolve to IPv6 address.
    // int resolve (name_: *const c_char, bool local_, bool ipv6_);

    //  The opposite to resolve()
    // int to_string (std::string &addr_) const;
    // TcpAddress () : _has_src_addr (false)
    // {
    //     memset (&_address, 0, mem::size_of::<_address>());
    //     memset (&_source_address, 0, mem::size_of::<_source_address>());
    // }
    
    // TcpAddress (const sockaddr *sa_, socklen_t sa_len_) :
    // _has_src_addr (false)
    // {
    //     zmq_assert (sa_ && sa_len_ > 0);
    // 
    //     memset (&_address, 0, mem::size_of::<_address>());
    //     memset (&_source_address, 0, mem::size_of::<_source_address>());
    //     if (sa_->sa_family == AF_INET
    //         && sa_len_ >= static_cast<socklen_t> (sizeof (_address.ipv4)))
    //         memcpy (&_address.ipv4, sa_, sizeof (_address.ipv4));
    //     else if (sa_->sa_family == AF_INET6
    //              && sa_len_ >= static_cast<socklen_t> (sizeof (_address.ipv6)))
    //         memcpy (&_address.ipv6, sa_, sizeof (_address.ipv6));
    // }
    
    pub fn resolve(&mut self, name: &mut str, local: bool, ipv6: bool) -> i32 {
        // Test the ';' to know if we have a source address in name_
        // const char *src_delimiter = strrchr (name_, ';');
        let src_delimeter = name.find(';');
        if src_delimiter.is_some() {
            let src_name = &name[..src_delimeter.unwrap()];
            // const std::string src_name (name_, src_delimiter - name_);

            let mut src_resolver_opts = IpResolverOptions {
                bindable: true,
                allow_dns: true,
                allow_nic_name: true,
                ipv6,
                expect_port: true,
                allow_path: false,
            };

            // src_resolver_opts
            //   .bindable (true)
            //   //  Restrict hostname/service to literals to avoid any DNS
            //   //  lookups or service-name irregularity due to
            //   //  indeterminate socktype.
            //   .allow_dns (false)
            //   .allow_nic_name (true)
            //   .ipv6 (ipv6_)
            //   .expect_port (true);
            let mut src_resolver = IpResolver::new(&src_resolver_opts);
            let rc =  src_resolver.resolve (&mut self.src_addr.unwrap(), src_name);
            if rc != 0 {
                return -1;
            }
            *name = src_delimiter[1..];
            self.has_src_addr = true;
        }

        // IpResolverOptions resolver_opts;
        // resolver_opts.bindable (local_)
        //           .allow_dns (!local_)
        //           .allow_nic_name (local_)
        //           .ipv6 (ipv6_)
        //           .expect_port (true);
        let resolver_opts = IpResolverOptions {
            bindable: false,
            allow_dns: !local,
            allow_nic_name: local,
            ipv6,
            expect_port: true,
            allow_path: false,
        };

        // ip_resolver_t resolver (resolver_opts);
        let mut resolver = IpResolver::new(&resolver_opts);

        // return resolver.resolve (&_address, name_);
        return resolver.resolve(&mut self.addr, name);
    }

    pub fn as_string(&mut self, addr_: &mut String) -> i32 {
        if self.addr.ip().is_ipv4() == false && self.addr.ip().is_ipv6() == false {
            // self.addr.clear();
            return -1;
        }

        //  Not using service resolving because of
        //  https://github.com/zeromq/libzmq/commit/1824574f9b5a8ce786853320e3ea09fe1f822bc4
        // char hbuf[NI_MAXHOST];
        // let mut hbuf = String::new();
        // let mut rc = libc::getnameinfo(addr (), addrlen (), hbuf, mem::size_of::<hbuf>(), NULL,
        //                             0, NI_NUMERICHOST);
        // dns_lookup::getnameinfo()
        // if (rc != 0) {
        //     addr_.clear ();
        //     return rc;
        // }
        let mut hbuf = String::from(gethostname::gethostname().to_str().unwrap());


        if self.addr.ip().is_ipv6() {
            *addr_ = make_address_string(&hbuf, self.addr.port, ipv6_prefix,
                                         ipv6_suffix);
        } else {
            *addr_ = make_address_string(&hbuf, self.addr.port, ipv4_prefix,
                                         ipv4_suffix);
        }
        return 0;
    }
}

#[derive(Default,Debug,Clone)]
pub struct TcpAddressMask
{
  // public:
  //   tcp_address_mask_t ();
  // private:
  //   ip_addr_t _network_address;
  pub network_address: SocketAddr,
    // int _address_mask;
    pub address_mask: i32,
}

impl TcpAddressMask {
    // This function enhances tcp_address_t::resolve() with ability to parse
    // additional cidr-like(/xx) mask value at the end of the name string.
    // Works only with remote hostnames.
    // int resolve (const char *name_, bool ipv6_);
    pub fn resolve(&mut self, name_: &str, ipv6_: bool) -> i32 {
        // Find '/' at the end that separates address from the cidr mask number.
        // Allow empty mask clause and treat it like '/32' for ipv4 or '/128' for ipv6.
        // std::string addr_str, mask_str;
        let mut addr_str: &str = "";
        let mut mask_str: &str = "";

        // const char *delimiter = strrchr (name_, '/');
        let mut delimiter = name_.find('/');
        if delimiter.is_some() {
            addr_str = &name_[..delimiter.unwrap()];
            mask_str = &name_[delimiter.unwrap()+1..];
            if mask_str.is_empty () {
                errno = EINVAL;
                return -1;
            }
        } else {
            addr_str = name_;
        }

        // Parse address part using standard routines.
        // IpResolverOptions resolver_opts;
        // resolver_opts.bindable (false)
        //   .allow_dns (false)
        //   .allow_nic_name (false)
        //   .ipv6 (ipv6_)
        //   .expect_port (false);
        let mut resolver_opts = IpResolverOptions {
            bindable: false,
            allow_dns: false,
            allow_nic_name: false,
            ipv6: ipv6_,
            expect_port: false,
            allow_path: false,
        };
        // IpResolver resolver (resolver_opts);
        let mut resolver = IpResolver::new(&resolver_opts);

        let rc = resolver.resolve (&mut self.network_address, addr_str);
        if (rc != 0) {
            return rc;
        }

        // Parse the cidr mask number.
        let full_mask_ipv4 = 32;
          // sizeof (_network_address.ipv4.sin_addr) * CHAR_BIT;
        let full_mask_ipv6 = 128;
        if (mask_str.empty ()) {
            // _address_mask = _network_address.family () == AF_INET6 ? full_mask_ipv6
            //                                                        : full_mask_ipv4;
            self.address_mask = if self.network_address.is_ipv4() {
                full_mask_ipv4
            } else {
                full_mask_ipv6
            };
        } else if mask_str == "0" {
            self.address_mask = 0;
        }
        else {
            let mask = i32::from(mask_str);
            if (mask < 1)
                || (self.network_address.family () == AF_INET6 && mask > full_mask_ipv6)
                || (self.network_address.family () != AF_INET6
                    && mask > full_mask_ipv4) {
                errno = EINVAL;
                return -1;
            }
            self.address_mask = mask;
        }

        return 0;
    }

    // bool match_address (const struct sockaddr *ss_, socklen_t ss_len_) const;
    pub fn match_address (&mut self, ss: &SocketAddr) -> bool
{
    // zmq_assert (self.address_mask != -1 && ss_ != NULL
    //             && ss_len_
    //                  >= static_cast<socklen_t> (sizeof (struct sockaddr)));

    if ss.is_ipv4() != self.network_address.is_ipv4() {
        return false;
    }

    if self.address_mask > 0 {
        let mut mask = 0i32;
        // const uint8_t *our_bytes, *their_bytes;
        let their_bytes: *mut u8 = null_mut();
        let out_butes: *mut u8 = null_mut();
        if ss.is_ipv6() {
            // zmq_assert (ss_len_ == sizeof (struct sockaddr_in6));
            ss.ip().
            their_bytes = reinterpret_cast<const uint8_t *> (
              &((reinterpret_cast<const struct sockaddr_in6 *> (ss.))
                  ->sin6_addr));
            our_bytes = reinterpret_cast<const uint8_t *> (
              &self.network_address.ipv6.sin6_addr);
            mask = sizeof (struct in6_addr) * 8;
        } else {
            zmq_assert (ss_len_ == sizeof (struct sockaddr_in));
            their_bytes = reinterpret_cast<const uint8_t *> (&(
              (reinterpret_cast<const struct sockaddr_in *> (ss.))->sin_addr));
            our_bytes = reinterpret_cast<const uint8_t *> (
              &self.network_address.ipv4.sin_addr);
            mask = sizeof (struct in_addr) * 8;
        }
        if (self.address_mask < mask) {
            mask = self.address_mask;
        }

        let full_bytes = mask / 8;
        if (memcmp (our_bytes, their_bytes, full_bytes) != 0) {
            return false;
        }

        let last_byte_bits = 0xffU << (8 - mask % 8);
        if (last_byte_bits) {
            if ((their_bytes[full_bytes] & last_byte_bits)
                != (our_bytes[full_bytes] & last_byte_bits)) {
                return false;
            }
        }
    }

    return true;
}

}

pub fn make_address_string(hbuf_: &str, port_: u16,ipv6_prefix: &str, ipv6_suffix: &str) -> String
{
    let mut max_port_str_length = 5usize;
    // char buf[NI_MAXHOST + sizeof ipv6_prefix_ + sizeof ipv6_suffix_
    //          + max_port_str_length];
    let mut buf = String::new();
    let mut pos = 0usize;
    // char *pos = buf;
    // memcpy (pos, ipv6_prefix_, sizeof ipv6_prefix_ - 1);
    buf += ipv6_prefix;
    pos += ipv6_prefix.len(); - 1;
    // pos += sizeof ipv6_prefix_ - 1;
    // const size_t hbuf_len = strlen (hbuf_);
    let mut hbuf_len = hbuf_.len();
    // memcpy (pos, hbuf_, hbuf_len);
    buf += hbuf_;
    pos += hbuf.len();
    // pos += hbuf_len;
    // memcpy (pos, ipv6_suffix_, sizeof ipv6_suffix_ - 1);
    buf += ipv6_suffix;
    pos += ipv6_suffix.len() - 1;
    // pos += sizeof ipv6_suffix_ - 1;
    // pos += sprintf (pos, "%d", ntohs (port_));
    hbuf += port_.to_be().to_string();
    // return std::string (buf, pos - buf);
    hbuf
}
