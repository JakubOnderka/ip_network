use std::fmt;
use std::net::Ipv6Addr;
use std::str::FromStr;
use crate::{IpNetworkError, IpNetworkParseError};
use crate::helpers;
use crate::iterator;

/// IPv6 Multicast Address Scopes
#[derive(Copy, PartialEq, Eq, Clone, Hash, Debug)]
pub enum Ipv6MulticastScope {
    InterfaceLocal,
    LinkLocal,
    RealmLocal,
    AdminLocal,
    SiteLocal,
    OrganizationLocal,
    Global,
}

/// IPv6 Network
#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Ipv6Network {
    network_address: Ipv6Addr,
    netmask: u8,
}

impl Ipv6Network {
    /// IPv4 address length in bits.
    pub const LENGTH: u8 = 128;

    /// Constructs new `Ipv6Network` based on [`Ipv6Addr`] and `netmask`.
    ///
    /// Returns error if netmask is bigger than 128 or if host bits are set in `network_address`.
    ///
    /// [`Ipv6Addr`]: https://doc.rust-lang.org/std/net/struct.Ipv6Addr.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ip_network::Ipv6Network;
    ///
    /// let ip = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0);
    /// let ip_network = Ipv6Network::new(ip, 32).unwrap();
    /// assert_eq!(ip_network.network_address(), ip);
    /// assert_eq!(ip_network.netmask(), 32);
    /// ```
    pub fn new(network_address: Ipv6Addr, netmask: u8) -> Result<Self, IpNetworkError> {
        if netmask > Self::LENGTH {
            return Err(IpNetworkError::NetmaskError(netmask));
        }

        if u128::from(network_address).trailing_zeros() < (Self::LENGTH as u32 - netmask as u32) {
            return Err(IpNetworkError::HostBitsSet);
        }

        Ok(Self {
            network_address,
            netmask,
        })
    }

    /// Constructs new `Ipv6Network` based on [`Ipv6Addr`] and `netmask` with truncating host bits
    /// from given `network_address`.
    ///
    /// Returns error if netmask is bigger than 128.
    ///
    /// [`Ipv6Addr`]: https://doc.rust-lang.org/std/net/struct.Ipv6Addr.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ip_network::Ipv6Network;
    ///
    /// let ip = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 1, 0, 0);
    /// let ip_network = Ipv6Network::new_truncate(ip, 32).unwrap();
    /// assert_eq!(ip_network.network_address(), Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0));
    /// assert_eq!(ip_network.netmask(), 32);
    /// ```
    pub fn new_truncate(network_address: Ipv6Addr, netmask: u8) -> Result<Self, IpNetworkError> {
        if netmask > Self::LENGTH {
            return Err(IpNetworkError::NetmaskError(netmask));
        }

        let network_address_u128 =
            u128::from(network_address) & helpers::get_bite_mask_u128(netmask);
        let network_address = Ipv6Addr::from(network_address_u128);

        Ok(Self {
            network_address,
            netmask,
        })
    }

    /// Returns network IP address (first address in range).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ip_network::Ipv6Network;
    ///
    /// let ip = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0);
    /// let ip_network = Ipv6Network::new(ip, 32).unwrap();
    /// assert_eq!(ip_network.network_address(), ip);
    /// ```
    #[inline]
    pub fn network_address(&self) -> Ipv6Addr {
        self.network_address
    }

    /// Returns network mask.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ip_network::Ipv6Network;
    ///
    /// let ip = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0);
    /// let ip_network = Ipv6Network::new(ip, 32).unwrap();
    /// assert_eq!(ip_network.netmask(), 32);
    /// ```
    #[inline]
    pub fn netmask(&self) -> u8 {
        self.netmask
    }

    /// Returns [`true`] if given [`IPv6Addr`] is inside this network.
    ///
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    /// [`Ipv6Addr`]: https://doc.rust-lang.org/std/net/struct.Ipv6Addr.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ip_network::Ipv6Network;
    ///
    /// let ip_network = Ipv6Network::new(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 64).unwrap();
    /// assert!(ip_network.contains(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1)));
    /// assert!(!ip_network.contains(Ipv6Addr::new(0x2001, 0xdb9, 0, 0, 0, 0, 0, 0)));
    /// ```
    pub fn contains(&self, ip: Ipv6Addr) -> bool {
        let truncated_ip = u128::from(ip) & helpers::get_bite_mask_u128(self.netmask);
        truncated_ip == u128::from(self.network_address)
    }

    /// Returns network with smaller netmask by one. If netmask is already zero, `None` will be returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ip_network::Ipv6Network;
    ///
    /// let network = Ipv6Network::new(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 32).unwrap();
    /// assert_eq!(network.supernet(), Some(Ipv6Network::new(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 31).unwrap()));
    /// ```
    pub fn supernet(&self) -> Option<Self> {
        if self.netmask == 0 {
            None
        } else {
            Some(Self::new_truncate(self.network_address, self.netmask - 1).unwrap())
        }
    }

    /// Returns `Ipv6NetworkIterator` over networks with netmask bigger one.
    /// If netmask is already 128, `None` will be returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ip_network::Ipv6Network;
    ///
    /// let ip_network = Ipv6Network::new(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 32).unwrap();
    /// let mut iterator = ip_network.subnets().unwrap();
    /// assert_eq!(iterator.next().unwrap(), Ipv6Network::new(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 33).unwrap());
    /// assert_eq!(iterator.last().unwrap(), Ipv6Network::new(Ipv6Addr::new(0x2001, 0xdb8, 0x8000, 0, 0, 0, 0, 0), 33).unwrap());
    /// ```
    pub fn subnets(&self) -> Option<iterator::Ipv6NetworkIterator> {
        if self.netmask == Self::LENGTH {
            None
        } else {
            Some(iterator::Ipv6NetworkIterator::new(self.clone(), self.netmask + 1))
        }
    }

    /// Returns `Ipv6NetworkIterator` over networks with defined netmask.
    ///
    /// # Panics
    ///
    /// This method panics when prefix is bigger than 128 or when prefix is lower or equal than netmask.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ip_network::Ipv6Network;
    ///
    /// let network = Ipv6Network::new(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 32).unwrap();
    /// let mut iterator = network.subnets_with_prefix(33);
    /// assert_eq!(iterator.next().unwrap(), Ipv6Network::new(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 33).unwrap());
    /// assert_eq!(iterator.last().unwrap(), Ipv6Network::new(Ipv6Addr::new(0x2001, 0xdb8, 0x8000, 0, 0, 0, 0, 0), 33).unwrap());
    /// ```
    pub fn subnets_with_prefix(&self, prefix: u8) -> iterator::Ipv6NetworkIterator {
        iterator::Ipv6NetworkIterator::new(self.clone(), prefix)
    }

    /// Returns [`true`] for the special 'unspecified' network (::/128).
    ///
    /// This property is defined in [IETF RFC 4291].
    ///
    /// [IETF RFC 4291]: https://tools.ietf.org/html/rfc4291
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ip_network::Ipv6Network;
    ///
    /// assert!(!Ipv6Network::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff), 128).unwrap().is_unspecified());
    /// assert!(Ipv6Network::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0), 128).unwrap().is_unspecified());
    /// ```
    pub fn is_unspecified(&self) -> bool {
        self.network_address.is_unspecified() && self.netmask == Self::LENGTH
    }

    /// Returns [`true`] if this is a loopback network (::1/128).
    ///
    /// This property is defined in [IETF RFC 4291].
    ///
    /// [IETF RFC 4291]: https://tools.ietf.org/html/rfc4291
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ip_network::Ipv6Network;
    ///
    /// assert!(Ipv6Network::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0x1), 128).unwrap().is_loopback());
    /// assert!(!Ipv6Network::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff), 128).unwrap().is_loopback());
    /// ```
    pub fn is_loopback(&self) -> bool {
        self.network_address.is_loopback()
    }

    /// Returns [`true`] if the address appears to be globally routable.
    ///
    /// The following return [`false`]:
    ///
    /// - the loopback network
    /// - link-local, site-local, and unique local unicast networks
    /// - interface-, link-, realm-, admin- and site-local multicast networks
    ///
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    /// [`false`]: https://doc.rust-lang.org/std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ip_network::Ipv6Network;
    ///
    /// assert!(Ipv6Network::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff), 128).unwrap().is_global());
    /// assert!(!Ipv6Network::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0x1), 128).unwrap().is_global());
    /// assert!(Ipv6Network::new(Ipv6Addr::new(0, 0, 0x1c9, 0, 0, 0xafc8, 0, 0x1), 128).unwrap().is_global());
    /// ```
    pub fn is_global(&self) -> bool {
        match self.multicast_scope() {
            Some(Ipv6MulticastScope::Global) => true,
            None => self.is_unicast_global(),
            _ => false,
        }
    }

    /// Returns [`true`] if this is a part of unique local network (fc00::/7).
    ///
    /// This property is defined in [IETF RFC 4193].
    ///
    /// [IETF RFC 4193]: https://tools.ietf.org/html/rfc4193
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ip_network::Ipv6Network;
    ///
    /// assert!(Ipv6Network::new(Ipv6Addr::new(0xfc02, 0, 0, 0, 0, 0, 0, 0), 16).unwrap().is_unique_local());
    /// assert!(!Ipv6Network::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff), 128).unwrap().is_unique_local());
    /// ```
    pub fn is_unique_local(&self) -> bool {
        (self.network_address.segments()[0] & 0xfe00) == 0xfc00 && self.netmask >= 7
    }

    /// Returns [`true`] if the network is part of unicast and link-local (fe80::/10).
    ///
    /// This property is defined in [IETF RFC 4291].
    ///
    /// [IETF RFC 4291]: https://tools.ietf.org/html/rfc4291
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ip_network::Ipv6Network;
    ///
    /// assert!(Ipv6Network::new(Ipv6Addr::new(0xfe8a, 0, 0, 0, 0, 0, 0, 0), 16).unwrap().is_unicast_link_local());
    /// assert!(!Ipv6Network::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff), 128).unwrap().is_unicast_link_local());
    /// ```
    pub fn is_unicast_link_local(&self) -> bool {
        (self.network_address.segments()[0] & 0xffc0) == 0xfe80 && self.netmask >= 10
    }

    /// Returns [`true`] if this is a deprecated unicast site-local network (fec0::/10).
    ///
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ip_network::Ipv6Network;
    ///
    /// assert!(Ipv6Network::new(Ipv6Addr::new(0xfec2, 0, 0, 0, 0, 0, 0, 0), 16).unwrap().is_unicast_site_local());
    /// assert!(!Ipv6Network::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff), 128).unwrap().is_unicast_site_local());
    /// ```
    pub fn is_unicast_site_local(&self) -> bool {
        (self.network_address.segments()[0] & 0xffc0) == 0xfec0 && self.netmask >= 10
    }

    /// Returns [`true`] if this is a part of network reserved for documentation (2001:db8::/32).
    ///
    /// This property is defined in [IETF RFC 3849].
    ///
    /// [IETF RFC 3849]: https://tools.ietf.org/html/rfc3849
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ip_network::Ipv6Network;
    ///
    /// assert!(Ipv6Network::new(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 32).unwrap().is_documentation());
    /// assert!(!Ipv6Network::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff), 128).unwrap().is_documentation());
    /// ```
    pub fn is_documentation(&self) -> bool {
        let segments = self.network_address.segments();
        segments[0] == 0x2001 && segments[1] == 0xdb8 && self.netmask >= 32
    }

    /// Returns [`true`] if the network is a globally routable unicast network.
    ///
    /// The following return [`false`]:
    ///
    /// - the loopback network
    /// - the link-local network
    /// - the (deprecated) site-local network
    /// - unique local network
    /// - the unspecified network
    /// - the network range reserved for documentation
    ///
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    /// [`false`]: https://doc.rust-lang.org/std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ip_network::Ipv6Network;
    ///
    /// assert!(!Ipv6Network::new(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 32).unwrap().is_unicast_global());
    /// assert!(Ipv6Network::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff), 128).unwrap().is_unicast_global());
    /// ```
    pub fn is_unicast_global(&self) -> bool {
        !self.is_multicast()
            && !self.is_loopback()
            && !self.is_unicast_link_local()
            && !self.is_unicast_site_local()
            && !self.is_unique_local()
            && !self.is_unspecified()
            && !self.is_documentation()
    }

    /// Returns [`true`] if this is a part of multicast network (ff00::/8).
    ///
    /// This property is defined by [IETF RFC 4291].
    ///
    /// [IETF RFC 4291]: https://tools.ietf.org/html/rfc4291
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ip_network::Ipv6Network;
    ///
    /// assert!(Ipv6Network::new(Ipv6Addr::new(0xff00, 0, 0, 0, 0, 0, 0, 0), 8).unwrap().is_multicast());
    /// assert!(!Ipv6Network::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff), 128).unwrap().is_multicast());
    /// ```
    pub fn is_multicast(&self) -> bool {
        (self.network_address.segments()[0] & 0xff00) == 0xff00 && self.netmask >= 8
    }

    /// Returns the network's multicast scope if the network is multicast.
    ///
    /// These scopes are defined in [IETF RFC 7346].
    ///
    /// [IETF RFC 7346]: https://tools.ietf.org/html/rfc7346
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ip_network::{Ipv6Network, Ipv6MulticastScope};
    ///
    /// assert_eq!(Ipv6Network::new(Ipv6Addr::new(0xff0e, 0, 0, 0, 0, 0, 0, 0), 32).unwrap().multicast_scope(),
    ///                              Some(Ipv6MulticastScope::Global));
    /// assert_eq!(Ipv6Network::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0xffff, 0xc00a, 0x2ff), 128).unwrap().multicast_scope(), None);
    /// ```
    pub fn multicast_scope(&self) -> Option<Ipv6MulticastScope> {
        if self.is_multicast() {
            match self.network_address.segments()[0] & 0x000f {
                1 => Some(Ipv6MulticastScope::InterfaceLocal),
                2 => Some(Ipv6MulticastScope::LinkLocal),
                3 => Some(Ipv6MulticastScope::RealmLocal),
                4 => Some(Ipv6MulticastScope::AdminLocal),
                5 => Some(Ipv6MulticastScope::SiteLocal),
                8 => Some(Ipv6MulticastScope::OrganizationLocal),
                14 => Some(Ipv6MulticastScope::Global),
                _ => None,
            }
        } else {
            None
        }
    }
}

impl fmt::Display for Ipv6Network {
    /// Converts `Ipv6Network` to string in format X:X::X/Y (CIDR notation).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ip_network::Ipv6Network;
    ///
    /// let ip_network = Ipv6Network::new(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 32).unwrap();
    /// assert_eq!(ip_network.to_string(), "2001:db8::/32");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.network_address, self.netmask)
    }
}

impl FromStr for Ipv6Network {
    type Err = IpNetworkParseError;

    /// Converts string in format X:X::X/Y (CIDR notation) to `Ipv6Network`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ip_network::Ipv6Network;
    /// use std::str::FromStr;
    ///
    /// let ip_network = Ipv6Network::from_str("2001:db8::/32").unwrap();
    /// assert_eq!(ip_network.network_address(), Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0));
    /// assert_eq!(ip_network.netmask(), 32);
    /// ```
    fn from_str(s: &str) -> Result<Ipv6Network, IpNetworkParseError> {
        let (ip, netmask) =
            helpers::split_ip_netmask(s).ok_or(IpNetworkParseError::InvalidFormatError)?;

        let network_address =
            Ipv6Addr::from_str(ip).map_err(|_| IpNetworkParseError::AddrParseError)?;
        let netmask =
            u8::from_str(netmask).map_err(|_| IpNetworkParseError::InvalidNetmaskFormat)?;

        Self::new(network_address, netmask).map_err(IpNetworkParseError::IpNetworkError)
    }
}

impl From<Ipv6Addr> for Ipv6Network {
    /// Converts `Ipv6Addr` to `Ipv6Network` with netmask 128.
    fn from(ip: Ipv6Addr) -> Self {
        Self {
            network_address: ip,
            netmask: Self::LENGTH,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv6Addr;
    use crate::Ipv6Network;

    fn return_test_ipv6_network() -> Ipv6Network {
        Ipv6Network::new(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 32).unwrap()
    }

    #[test]
    fn new() {
        let ip = Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 0);
        let network = Ipv6Network::new(ip, 7).unwrap();
        assert_eq!(
            network.network_address(),
            Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 0)
        );
        assert_eq!(network.netmask(), 7);
    }

    #[test]
    fn contains() {
        let ip_network = return_test_ipv6_network();
        assert!(!ip_network.contains(Ipv6Addr::new(
            0x2001, 0x0db7, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff
        )));
        assert!(ip_network.contains(Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 0)));
        assert!(ip_network.contains(Ipv6Addr::new(
            0x2001, 0x0db8, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff
        )));
        assert!(!ip_network.contains(Ipv6Addr::new(0x2001, 0x0db9, 0, 0, 0, 0, 0, 0)));
    }

    #[test]
    fn supernet() {
        let ip_network = return_test_ipv6_network();
        assert_eq!(
            ip_network.supernet(),
            Some(Ipv6Network::new(Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 0), 31).unwrap())
        );
    }

    #[test]
    fn subnets() {
        let mut subnets = return_test_ipv6_network().subnets().unwrap();
        assert_eq!(subnets.len(), 2);
        assert_eq!(
            subnets.next().unwrap(),
            Ipv6Network::new(Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 0), 33).unwrap()
        );
        assert_eq!(
            subnets.next().unwrap(),
            Ipv6Network::new(Ipv6Addr::new(0x2001, 0x0db8, 0x8000, 0, 0, 0, 0, 0), 33).unwrap()
        );
        assert!(subnets.next().is_none());
    }

    #[test]
    fn subnets_with_prefix() {
        let ip_network = return_test_ipv6_network();
        let mut subnets = ip_network.subnets_with_prefix(34);
        assert_eq!(subnets.len(), 4);
        assert_eq!(
            subnets.next().unwrap(),
            Ipv6Network::new(Ipv6Addr::new(0x2001, 0x0db8, 0x0000, 0, 0, 0, 0, 0), 34).unwrap()
        );
        assert_eq!(
            subnets.next().unwrap(),
            Ipv6Network::new(Ipv6Addr::new(0x2001, 0x0db8, 0x4000, 0, 0, 0, 0, 0), 34).unwrap()
        );
        assert_eq!(
            subnets.next().unwrap(),
            Ipv6Network::new(Ipv6Addr::new(0x2001, 0x0db8, 0x8000, 0, 0, 0, 0, 0), 34).unwrap()
        );
        assert_eq!(
            subnets.next().unwrap(),
            Ipv6Network::new(Ipv6Addr::new(0x2001, 0x0db8, 0xc000, 0, 0, 0, 0, 0), 34).unwrap()
        );
        assert!(subnets.next().is_none());
    }

    #[test]
    fn parse() {
        let ip_network: Ipv6Network = "2001:db8::/32".parse().unwrap();
        assert_eq!(ip_network, return_test_ipv6_network());
    }

    #[test]
    fn format() {
        let ip_network = return_test_ipv6_network();
        assert_eq!(ip_network.to_string(), "2001:db8::/32");
    }
}