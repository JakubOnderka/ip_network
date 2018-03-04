extern crate extprim;
#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;
#[cfg(feature = "postgres")]
#[macro_use]
extern crate postgres;
#[cfg(feature = "diesel")]
extern crate diesel;

use std::fmt;
use std::cmp;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use std::error::Error;

mod helpers;
/// `Ipv4RangeIterator`, `Ipv4NetworkIterator` and `Ipv6NetworkIterator`
pub mod iterator;

#[cfg(any(feature = "diesel", feature = "postgres"))]
mod postgres_common;
#[cfg(feature = "postgres")]
mod postgres_support;
#[cfg(feature = "diesel")]
mod diesel_support;

const IPV4_LENGTH: u8 = 32;
const IPV6_LENGTH: u8 = 128;

/// Holds IPv4 or IPv6 network
#[derive(Clone, Eq, PartialEq, Debug, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum IpNetwork {
    V4(Ipv4Network),
    V6(Ipv6Network),
}

impl IpNetwork {
    pub fn is_ipv4(&self) -> bool {
        match *self {
            IpNetwork::V4(_) => true,
            IpNetwork::V6(_) => false,
        }
    }

    pub fn is_ipv6(&self) -> bool {
        !self.is_ipv4()
    }
}

impl FromStr for IpNetwork {
    type Err = IpNetworkParseError;

    /// Converts string in format IPv4 (X.X.X.X/Y) or IPv6 (X:X::X/Y) CIDR notation to `IpNetwork`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    /// use std::str::FromStr;
    /// use ip_network::{IpNetwork, Ipv4Network};
    ///
    /// let ip_network = IpNetwork::from_str("192.168.1.0/24").unwrap();
    /// assert_eq!(ip_network, IpNetwork::V4(Ipv4Network::from(Ipv4Addr::new(192, 168, 1, 0), 24).unwrap()));
    /// ```
    fn from_str(s: &str) -> Result<IpNetwork, IpNetworkParseError> {
        let (ip, netmask) = match helpers::split_ip_netmask(s) {
            Some(output) => output,
            None => return Err(IpNetworkParseError::InvalidFormatError),
        };

        let netmask = u8::from_str(netmask)
            .map_err(|_| IpNetworkParseError::InvalidNetmaskFormat)?;

        if let Ok(network_address) = Ipv4Addr::from_str(ip) {
            let network = Ipv4Network::from(network_address, netmask)
                .map_err(IpNetworkParseError::IpNetworkError)?;
            Ok(IpNetwork::V4(network))

        } else if let Ok(network_address) = Ipv6Addr::from_str(ip) {
            let network = Ipv6Network::from(network_address, netmask)
                .map_err(IpNetworkParseError::IpNetworkError)?;
            Ok(IpNetwork::V6(network))

        } else {
            Err(IpNetworkParseError::AddrParseError)
        }
    }
}

impl fmt::Display for IpNetwork {
    /// Converts `IpNetwork` to string in format X.X.X.X/Y for IPv4 and X:X::X/Y for IPv6 (CIDR notation).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    /// use ip_network::{IpNetwork, Ipv4Network};
    ///
    /// let ip_network = IpNetwork::V4(Ipv4Network::from(Ipv4Addr::new(192, 168, 1, 0), 24).unwrap());
    /// assert_eq!(format!("{}", ip_network), "192.168.1.0/24");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IpNetwork::V4(ref network) => network.fmt(f),
            IpNetwork::V6(ref network) => network.fmt(f),
        }
    }
}

/// IPv4 Network
#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Ipv4Network {
    network_address: Ipv4Addr,
    netmask: u8,
}

impl Ipv4Network {
    /// Constructs new `Ipv4Network` based on `Ipv4Addr` and `netmask`.
    ///
    /// Returns error if netmask is biger than 32 or if host bits are set in `network_address`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    /// use ip_network::Ipv4Network;
    ///
    /// let ip_network = Ipv4Network::from(Ipv4Addr::new(192, 168, 1, 0), 24).unwrap();
    /// assert_eq!(ip_network.get_network_address(), Ipv4Addr::new(192, 168, 1, 0));
    /// assert_eq!(ip_network.get_netmask(), 24);
    /// ```
    pub fn from(network_address: Ipv4Addr, netmask: u8) -> Result<Self, IpNetworkError> {
        if netmask > IPV4_LENGTH {
            return Err(IpNetworkError::NetmaskError(netmask));
        }

        let network_address_u32 = u32::from(network_address);
        if network_address_u32 & helpers::get_bite_mask(netmask) != network_address_u32 {
            return Err(IpNetworkError::HostBitsSet);
        }

        Ok(Self {
            network_address,
            netmask,
        })
    }

    /// Constructs new `Ipv4Network` based on `Ipv4Addr` and `netmask` with truncating host bits
    /// from given `network_address`.
    ///
    /// Returns error if netmask is biger than 32.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    /// use ip_network::Ipv4Network;
    ///
    /// let ip_network = Ipv4Network::from_truncate(Ipv4Addr::new(192, 168, 1, 100), 24).unwrap();
    /// assert_eq!(ip_network.get_network_address(), Ipv4Addr::new(192, 168, 1, 0));
    /// assert_eq!(ip_network.get_netmask(), 24);
    /// ```
    pub fn from_truncate(network_address: Ipv4Addr, netmask: u8) -> Result<Self, IpNetworkError> {
        if netmask > IPV4_LENGTH {
            return Err(IpNetworkError::NetmaskError(netmask));
        }

        let network_address = Ipv4Addr::from(u32::from(network_address) & helpers::get_bite_mask(netmask));

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
    /// use std::net::Ipv4Addr;
    /// use ip_network::Ipv4Network;
    ///
    /// let ip_network = Ipv4Network::from(Ipv4Addr::new(192, 168, 1, 0), 24).unwrap();
    /// assert_eq!(ip_network.get_network_address(), Ipv4Addr::new(192, 168, 1, 0));
    /// ```
    pub fn get_network_address(&self) -> Ipv4Addr {
        self.network_address
    }

    /// Returns broadcast address of network (last address in range).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    /// use ip_network::Ipv4Network;
    ///
    /// let ip_network = Ipv4Network::from(Ipv4Addr::new(192, 168, 1, 0), 24).unwrap();
    /// assert_eq!(ip_network.get_broadcast_address(), Ipv4Addr::new(192, 168, 1, 255));
    /// ```
    pub fn get_broadcast_address(&self) -> Ipv4Addr {
        Ipv4Addr::from(u32::from(self.network_address) | !helpers::get_bite_mask(self.netmask))
    }

    /// Returns network mask as integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    /// use ip_network::Ipv4Network;
    ///
    /// let ip_network = Ipv4Network::from(Ipv4Addr::new(192, 168, 1, 0), 24).unwrap();
    /// assert_eq!(ip_network.get_netmask(), 24);
    /// ```
    pub fn get_netmask(&self) -> u8 {
        self.netmask
    }

    /// Returns network mask as IPv4 address.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    /// use ip_network::Ipv4Network;
    ///
    /// let ip_network = Ipv4Network::from(Ipv4Addr::new(192, 168, 1, 0), 24).unwrap();
    /// assert_eq!(ip_network.get_full_netmask(), Ipv4Addr::new(255, 255, 255, 0));
    /// ```
    pub fn get_full_netmask(&self) -> Ipv4Addr {
        Ipv4Addr::from(helpers::get_bite_mask(self.netmask))
    }

    /// Returns [`true`] if given IP address is inside this network.
    ///
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    /// use ip_network::Ipv4Network;
    ///
    /// let ip_network = Ipv4Network::from(Ipv4Addr::new(192, 168, 1, 0), 24).unwrap();
    /// assert!(ip_network.contains(Ipv4Addr::new(192, 168, 1, 2)));
    /// assert!(!ip_network.contains(Ipv4Addr::new(192, 168, 2, 2)));
    /// ```
    pub fn contains(&self, ip: Ipv4Addr) -> bool {
        u32::from(ip) & helpers::get_bite_mask(self.netmask) == u32::from(self.network_address)
    }

    /// Returns iterator over host IP addresses in range (without network and broadcast address).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    /// use ip_network::Ipv4Network;
    ///
    /// let ip = Ipv4Addr::new(192, 168, 1, 0);
    /// let mut hosts = Ipv4Network::from(ip, 24).unwrap().hosts();
    /// assert_eq!(hosts.next().unwrap(), Ipv4Addr::new(192, 168, 1, 1));
    /// assert_eq!(hosts.last().unwrap(), Ipv4Addr::new(192, 168, 1, 254));
    /// ```
    pub fn hosts(&self) -> iterator::Ipv4RangeIterator {
        let from = Ipv4Addr::from(u32::from(self.network_address).saturating_add(1));
        let to = Ipv4Addr::from(u32::from(self.get_broadcast_address()).saturating_sub(1));
        iterator::Ipv4RangeIterator::new(from, to)
    }

    /// Returns network with smaller netmask by one.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    /// use ip_network::Ipv4Network;
    ///
    /// let ip = Ipv4Addr::new(192, 168, 1, 0);
    /// let mut hosts = Ipv4Network::from(ip, 24).unwrap();
    /// assert_eq!(hosts.supernet(), Ipv4Network::from(Ipv4Addr::new(192, 168, 0, 0), 23).unwrap());
    /// ```
    pub fn supernet(&self) -> Self {
        Self::from_truncate(self.get_network_address(), self.get_netmask().saturating_sub(1)).unwrap()
    }

    /// Returns `Ipv4NetworkIterator` over networks with bigger netmask by one.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    /// use ip_network::Ipv4Network;
    ///
    /// let ip = Ipv4Addr::new(192, 168, 1, 0);
    /// let mut iterator = Ipv4Network::from(ip, 24).unwrap().subnets();
    /// assert_eq!(iterator.next().unwrap(), Ipv4Network::from(Ipv4Addr::new(192, 168, 1, 0), 25).unwrap());
    /// assert_eq!(iterator.last().unwrap(), Ipv4Network::from(Ipv4Addr::new(192, 168, 1, 128), 25).unwrap());
    /// ```
    pub fn subnets(&self) -> iterator::Ipv4NetworkIterator {
        iterator::Ipv4NetworkIterator::new(self.clone(), self.get_netmask().saturating_add(1))
    }

    /// Returns `Ipv4NetworkIterator` over networks with defined netmask.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    /// use ip_network::Ipv4Network;
    ///
    /// let ip = Ipv4Addr::new(192, 168, 1, 0);
    /// let mut iterator = Ipv4Network::from(ip, 24).unwrap().subnets_with_prefix(25);
    /// assert_eq!(iterator.next().unwrap(), Ipv4Network::from(Ipv4Addr::new(192, 168, 1, 0), 25).unwrap());
    /// assert_eq!(iterator.last().unwrap(), Ipv4Network::from(Ipv4Addr::new(192, 168, 1, 128), 25).unwrap());
    /// ```
    pub fn subnets_with_prefix(&self, prefix: u8) -> iterator::Ipv4NetworkIterator {
        iterator::Ipv4NetworkIterator::new(self.clone(), prefix)
    }

    /// Returns [`true`] if this network is inside loopback address range (127.0.0.0/8).
    ///
    /// This property is defined by [IETF RFC 1122].
    ///
    /// [IETF RFC 1122]: https://tools.ietf.org/html/rfc1122
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    /// use ip_network::Ipv4Network;
    ///
    /// let ip_network = Ipv4Network::from(Ipv4Addr::new(127, 0, 0, 0), 8).unwrap();
    /// assert!(ip_network.is_loopback());
    /// ```
    pub fn is_loopback(&self) -> bool {
        self.network_address.is_loopback() && self.get_broadcast_address().is_loopback()
    }

    /// Returns [`true`] if this is a broadcast network (255.255.255.255/32).
    ///
    /// A broadcast address has all octets set to 255 as defined in [IETF RFC 919].
    ///
    /// [IETF RFC 919]: https://tools.ietf.org/html/rfc919
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    /// use ip_network::Ipv4Network;
    ///
    /// let ip_network = Ipv4Network::from(Ipv4Addr::new(255, 255, 255, 255), 32).unwrap();
    /// assert!(ip_network.is_broadcast());
    /// ```
    pub fn is_broadcast(&self) -> bool {
        self.network_address.is_broadcast()
    }

    /// Returns [`true`] if this whole network range is inside private address ranges.
    ///
    /// The private address ranges are defined in [IETF RFC 1918] and include:
    ///
    ///  - 10.0.0.0/8
    ///  - 172.16.0.0/12
    ///  - 192.168.0.0/16
    ///
    /// [IETF RFC 1918]: https://tools.ietf.org/html/rfc1918
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    /// use ip_network::Ipv4Network;
    ///
    /// let ip_network = Ipv4Network::from(Ipv4Addr::new(192, 168, 1, 0), 24).unwrap();
    /// assert!(ip_network.is_private());
    /// ```
    pub fn is_private(&self) -> bool {
        self.network_address.is_private() && self.get_broadcast_address().is_private()
    }

    /// Returns [`true`] if the network is is inside link-local range (169.254.0.0/16).
    ///
    /// This property is defined by [IETF RFC 3927].
    ///
    /// [IETF RFC 3927]: https://tools.ietf.org/html/rfc3927
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    /// use ip_network::Ipv4Network;
    ///
    /// let ip_network = Ipv4Network::from(Ipv4Addr::new(169, 254, 1, 0), 24).unwrap();
    /// assert!(ip_network.is_link_local());
    /// ```
    pub fn is_link_local(&self) -> bool {
        self.network_address.is_link_local() && self.get_broadcast_address().is_link_local()
    }

    /// Returns [`true`] if this whole network is inside multicast address range (224.0.0.0/4).
    ///
    /// Multicast network addresses have a most significant octet between 224 and 239,
    /// and is defined by [IETF RFC 5771].
    ///
    /// [IETF RFC 5771]: https://tools.ietf.org/html/rfc5771
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    /// use ip_network::Ipv4Network;
    ///
    /// let ip_network = Ipv4Network::from(Ipv4Addr::new(224, 168, 1, 0), 24).unwrap();
    /// assert!(ip_network.is_multicast());
    /// ```
    pub fn is_multicast(&self) -> bool {
        self.network_address.is_multicast() && self.get_broadcast_address().is_multicast()
    }

    /// Returns [`true`] if this network is in a range designated for documentation.
    ///
    /// This is defined in [IETF RFC 5737]:
    ///
    /// - 192.0.2.0/24 (TEST-NET-1)
    /// - 198.51.100.0/24 (TEST-NET-2)
    /// - 203.0.113.0/24 (TEST-NET-3)
    ///
    /// [IETF RFC 5737]: https://tools.ietf.org/html/rfc5737
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    /// use ip_network::Ipv4Network;
    ///
    /// let ip_network = Ipv4Network::from(Ipv4Addr::new(192, 0, 2, 0), 24).unwrap();
    /// assert!(ip_network.is_documentation());
    /// ```
    pub fn is_documentation(&self) -> bool {
        self.network_address.is_documentation() && self.get_broadcast_address().is_documentation()
    }

    // TODO: Documentation
    pub fn summarize_address_range(first: Ipv4Addr, last: Ipv4Addr) -> Vec<Self> {
        let mut first_int = u32::from(first);
        let last_int = u32::from(last);

        let mut vector = Vec::with_capacity(1);

        while first_int <= last_int {
            let bit_length_diff;
            if last_int - first_int == std::u32::MAX {
                bit_length_diff = IPV4_LENGTH;
            } else {
                bit_length_diff = helpers::bit_length(last_int - first_int + 1) - 1
            }

            let nbits = cmp::min(
                first_int.trailing_zeros() as u8,
                bit_length_diff
            );

            vector.push(Self::from(
                Ipv4Addr::from(first_int),
                IPV4_LENGTH - nbits
            ).unwrap());

            if nbits == IPV4_LENGTH {
                break;
            }

            match first_int.checked_add(1 << nbits) {
                Some(x) => first_int = x,
                None => break,
            }
        }

        vector
    }
}

impl fmt::Display for Ipv4Network {
    /// Converts `Ipv4Network` to string in format X.X.X.X/Y (CIDR notation).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    /// use ip_network::Ipv4Network;
    ///
    /// let ip_network = Ipv4Network::from(Ipv4Addr::new(192, 168, 1, 0), 24).unwrap();
    /// assert_eq!(format!("{}", ip_network), "192.168.1.0/24");
    /// ```
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}/{}", self.network_address, self.netmask)
    }
}

impl FromStr for Ipv4Network {
    type Err = IpNetworkParseError;

    /// Converts string in format X.X.X.X/Y (CIDR notation) to `Ipv4Network`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    /// use ip_network::Ipv4Network;
    /// use std::str::FromStr;
    ///
    /// let ip_network = Ipv4Network::from_str("192.168.1.0/24").unwrap();
    /// assert_eq!(ip_network.get_network_address(), Ipv4Addr::new(192, 168, 1, 0));
    /// assert_eq!(ip_network.get_netmask(), 24);
    /// ```
    fn from_str(s: &str) -> Result<Ipv4Network, IpNetworkParseError> {
        let (ip, netmask) = match helpers::split_ip_netmask(s) {
            Some(a) => a,
            None => return Err(IpNetworkParseError::InvalidFormatError),
        };

        let network_address = Ipv4Addr::from_str(ip)
            .map_err(|_| IpNetworkParseError::AddrParseError)?;
        let netmask = u8::from_str(netmask)
            .map_err(|_| IpNetworkParseError::InvalidNetmaskFormat)?;

        Self::from(network_address, netmask).map_err(IpNetworkParseError::IpNetworkError)
    }
}

impl IntoIterator for Ipv4Network {
    type Item = Ipv4Addr;
    type IntoIter = iterator::Ipv4RangeIterator;

    /// Returns iterator over all IP addresses in range including network and broadcast addresses.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    /// use ip_network::Ipv4Network;
    ///
    /// let ip = Ipv4Addr::new(192, 168, 1, 0);
    /// let mut iter = Ipv4Network::from(ip, 24).unwrap().into_iter();
    /// assert_eq!(iter.next().unwrap(), Ipv4Addr::new(192, 168, 1, 0));
    /// assert_eq!(iter.next().unwrap(), Ipv4Addr::new(192, 168, 1, 1));
    /// assert_eq!(iter.last().unwrap(), Ipv4Addr::new(192, 168, 1, 255));
    /// ```
    fn into_iter(self) -> Self::IntoIter {
         Self::IntoIter::new(self.network_address, self.get_broadcast_address())
    }
}

/// IPv6 Network
#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Ipv6Network {
    network_address: Ipv6Addr,
    netmask: u8,
}

impl Ipv6Network {
    /// Constructs new `Ipv6Network` based on `Ipv6Addr` and `netmask`.
    ///
    /// Returns error if netmask is bigger than 128 or if host bits are set in `network_address`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ip_network::Ipv6Network;
    ///
    /// let ip = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0);
    /// let ip_network = Ipv6Network::from(ip, 32).unwrap();
    /// assert_eq!(ip_network.get_network_address(), ip);
    /// assert_eq!(ip_network.get_netmask(), 32);
    /// ```
    pub fn from(network_address: Ipv6Addr, netmask: u8) -> Result<Self, IpNetworkError> {
        if netmask > IPV6_LENGTH {
            return Err(IpNetworkError::NetmaskError(netmask));
        }

        let network_address_u128 = helpers::ipv6addr_to_u128(network_address);
        if network_address_u128 & helpers::get_bite_mask_u128(netmask) != network_address_u128 {
            return Err(IpNetworkError::HostBitsSet);
        }

        Ok(Self {
            network_address,
            netmask,
        })
    }

    /// Constructs new `Ipv6Network` based on `Ipv6Addr` and `netmask` with truncating host bits
    /// from given `network_address`.
    ///
    /// Returns error if netmask is bigger than 128.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ip_network::Ipv6Network;
    ///
    /// let ip = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 1, 0, 0);
    /// let ip_network = Ipv6Network::from_truncate(ip, 32).unwrap();
    /// assert_eq!(ip_network.get_network_address(), Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0));
    /// assert_eq!(ip_network.get_netmask(), 32);
    /// ```
    pub fn from_truncate(network_address: Ipv6Addr, netmask: u8) -> Result<Self, IpNetworkError> {
        if netmask > IPV6_LENGTH {
            return Err(IpNetworkError::NetmaskError(netmask));
        }

        let network_address_u128 = helpers::ipv6addr_to_u128(network_address) & helpers::get_bite_mask_u128(netmask);
        let network_address = helpers::u128_to_ipv6addr(network_address_u128);

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
    /// let ip_network = Ipv6Network::from(ip, 32).unwrap();
    /// assert_eq!(ip_network.get_network_address(), ip);
    /// ```
    pub fn get_network_address(&self) -> Ipv6Addr {
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
    /// let ip_network = Ipv6Network::from(ip, 32).unwrap();
    /// assert_eq!(ip_network.get_netmask(), 32);
    /// ```
    pub fn get_netmask(&self) -> u8 {
        self.netmask
    }

    /// Returns [`true`] if given IP address is inside this network.
    ///
    /// [`true`]: https://doc.rust-lang.org/std/primitive.bool.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ip_network::Ipv6Network;
    ///
    /// let ip_network = Ipv6Network::from(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 64).unwrap();
    /// assert!(ip_network.contains(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1)));
    /// assert!(!ip_network.contains(Ipv6Addr::new(0x2001, 0xdb9, 0, 0, 0, 0, 0, 0)));
    /// ```
    pub fn contains(&self, ip: Ipv6Addr) -> bool {
        let truncated_ip = helpers::ipv6addr_to_u128(ip) & helpers::get_bite_mask_u128(self.netmask);
        truncated_ip == helpers::ipv6addr_to_u128(self.network_address)
    }

    /// Returns network with smaller netmask by one.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ip_network::Ipv6Network;
    ///
    /// let network = Ipv6Network::from(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 32).unwrap();
    /// assert_eq!(network.supernet(), Ipv6Network::from(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 31).unwrap());
    /// ```
    pub fn supernet(&self) -> Self {
        Self::from_truncate(self.get_network_address(), self.get_netmask().saturating_sub(1)).unwrap()
    }

    /// Returns `Ipv6NetworkIterator` over networks with netmask bigger one.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ip_network::Ipv6Network;
    ///
    /// let network = Ipv6Network::from(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 32).unwrap();
    /// let mut iterator = network.subnets();
    /// assert_eq!(iterator.next().unwrap(), Ipv6Network::from(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 33).unwrap());
    /// assert_eq!(iterator.last().unwrap(), Ipv6Network::from(Ipv6Addr::new(0x2001, 0xdb8, 0x8000, 0, 0, 0, 0, 0), 33).unwrap());
    /// ```
    pub fn subnets(&self) -> iterator::Ipv6NetworkIterator {
        iterator::Ipv6NetworkIterator::new(self.clone(), self.get_netmask().saturating_add(1))
    }

    /// Returns `Ipv6NetworkIterator` over networks with defined netmask.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv6Addr;
    /// use ip_network::Ipv6Network;
    ///
    /// let network = Ipv6Network::from(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 32).unwrap();
    /// let mut iterator = network.subnets_with_prefix(33);
    /// assert_eq!(iterator.next().unwrap(), Ipv6Network::from(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 33).unwrap());
    /// assert_eq!(iterator.last().unwrap(), Ipv6Network::from(Ipv6Addr::new(0x2001, 0xdb8, 0x8000, 0, 0, 0, 0, 0), 33).unwrap());
    /// ```
    pub fn subnets_with_prefix(&self, prefix: u8) -> iterator::Ipv6NetworkIterator {
        iterator::Ipv6NetworkIterator::new(self.clone(), prefix)
    }
}

impl fmt::Display for Ipv6Network {
    // TODO
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.network_address, self.netmask)
    }
}

impl FromStr for Ipv6Network {
    type Err = IpNetworkParseError;

    // TODO
    fn from_str(s: &str) -> Result<Ipv6Network, IpNetworkParseError> {
        let (ip, netmask) = match helpers::split_ip_netmask(s) {
            Some(a) => a,
            None => return Err(IpNetworkParseError::InvalidFormatError),
        };

        let network_address = Ipv6Addr::from_str(ip)
            .map_err(|_| IpNetworkParseError::AddrParseError)?;
        let netmask = u8::from_str(netmask)
            .map_err(|_| IpNetworkParseError::InvalidNetmaskFormat)?;

        Self::from(network_address, netmask).map_err(IpNetworkParseError::IpNetworkError)
    }
}

/// Errors when creating new IPv4 or IPv6 networks
#[derive(Debug)]
pub enum IpNetworkError {
    /// Network mask is bigger than possible for given IP version (32 for IPv4, 128 for IPv6)
    NetmaskError(u8),
    /// Host bits are set in given network IP address
    HostBitsSet,
}

impl Error for IpNetworkError {
    fn description(&self) -> &str {
        match *self {
            IpNetworkError::NetmaskError(_) => "invalid netmask",
            IpNetworkError::HostBitsSet => "IP network address has host bits set",
        }
    }
}

impl fmt::Display for IpNetworkError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.description())
    }
}

/// Errors from IPv4 or IPv6 network parsing
#[derive(Debug)]
pub enum IpNetworkParseError {
    /// Network mask is not valid integer between 0-255
    InvalidNetmaskFormat,
    // Network address has invalid format (not X/Y)
    InvalidFormatError,
    /// Invalid IP address syntax (IPv4 or IPv6)
    AddrParseError,
    IpNetworkError(IpNetworkError),
}

impl Error for IpNetworkParseError {
    fn description(&self) -> &str {
        match *self {
            IpNetworkParseError::InvalidNetmaskFormat => "invalid netmask format",
            IpNetworkParseError::InvalidFormatError => "invalid format",
            IpNetworkParseError::AddrParseError => "invalid IP address syntax",
            IpNetworkParseError::IpNetworkError(ref ip_network_error) => ip_network_error.description(),
        }
    }
}

impl fmt::Display for IpNetworkParseError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.description())
    }
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr};
    use {IpNetwork, Ipv6Network, Ipv4Network, IpNetworkParseError, IpNetworkError};

    fn return_test_ipv4_network() -> Ipv4Network {
        Ipv4Network::from(Ipv4Addr::new(192, 168, 0, 0), 16).unwrap()
    }

    fn return_test_ipv6_network() -> Ipv6Network {
        Ipv6Network::from(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 32).unwrap()
    }

    #[test]
    fn test_ip_network_is_ipv4() {
        let ip_network = IpNetwork::V4(return_test_ipv4_network());
        assert!(ip_network.is_ipv4());
        assert!(!ip_network.is_ipv6());
    }

    #[test]
    fn test_ip_network_is_ipv6() {
        let ip_network = IpNetwork::V6(return_test_ipv6_network());
        assert!(ip_network.is_ipv6());
        assert!(!ip_network.is_ipv4());
    }

    #[test]
    fn test_ip_network_parse_ipv4() {
        let ip_network: IpNetwork = "192.168.0.0/16".parse().unwrap();
        assert_eq!(ip_network, IpNetwork::V4(return_test_ipv4_network()));
    }

    #[test]
    fn test_ip_network_parse_ipv6() {
        let ip_network: IpNetwork = "2001:db8::/32".parse().unwrap();
        assert_eq!(ip_network, IpNetwork::V6(return_test_ipv6_network()));
    }

    #[test]
    fn test_ip_network_parse_empty() {
        let ip_network = "".parse::<IpNetwork>();
        assert!(ip_network.is_err());
        assert!(match ip_network.err().unwrap() {
            IpNetworkParseError::InvalidFormatError => true,
            _ => false,
        });
    }

    #[test]
    fn test_ip_network_parse_invalid_netmask() {
        let ip_network = "192.168.0.0/a".parse::<IpNetwork>();
        assert!(ip_network.is_err());
        assert!(match ip_network.err().unwrap() {
            IpNetworkParseError::InvalidNetmaskFormat => true,
            _ => false,
        });
    }

    #[test]
    fn test_ip_network_parse_invalid_ip() {
        let ip_network = "192.168.0.0a/16".parse::<IpNetwork>();
        assert!(ip_network.is_err());
        assert!(match ip_network.err().unwrap() {
            IpNetworkParseError::AddrParseError => true,
            _ => false,
        });
    }

    #[test]
    fn test_ip_network_parse_ipv4_host_bits_set() {
        let ip_network = "192.168.0.1/16".parse::<IpNetwork>();
        assert!(ip_network.is_err());
        assert!(match ip_network.err().unwrap() {
            IpNetworkParseError::IpNetworkError(_) => true,
            _ => false,
        });
    }

    #[test]
    fn test_ip_network_parse_ipv6_host_bits_set() {
        let ip_network = "2001:db8::1/32".parse::<IpNetwork>();
        assert!(ip_network.is_err());
        assert!(match ip_network.err().unwrap() {
            IpNetworkParseError::IpNetworkError(_) => true,
            _ => false,
        });
    }

    #[test]
    fn test_ip_network_format_ipv4() {
        let ip_network = IpNetwork::V4(return_test_ipv4_network());
        assert_eq!(format!("{}", ip_network), "192.168.0.0/16");
    }

    #[test]
    fn test_ip_network_format_ipv6() {
        let ip_network = IpNetwork::V6(return_test_ipv6_network());
        assert_eq!(format!("{}", ip_network), "2001:db8::/32");
    }

    #[test]
    fn test_ipv4_network_from_host_bits_set() {
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let ip_network = Ipv4Network::from(ip, 8);
        assert!(ip_network.is_err());
        assert!(match ip_network.err().unwrap() {
            IpNetworkError::HostBitsSet => true,
            _ => false,
        });
    }

    #[test]
    fn test_ipv4_network_from_big_invalid_netmask() {
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let ip_network = Ipv4Network::from(ip, 35);
        assert!(ip_network.is_err());
        assert!(match ip_network.err().unwrap() {
            IpNetworkError::NetmaskError(_) => true,
            _ => false,
        });
    }

    #[test]
    fn test_ipv4_network_from_truncate_host_bits_set() {
        let ip = Ipv4Addr::new(127, 0, 0, 1);
        let ip_network = Ipv4Network::from_truncate(ip, 8).unwrap();
        assert_eq!(ip_network.get_network_address(), Ipv4Addr::new(127, 0, 0, 0));
    }

    #[test]
    fn test_ipv4_network_basic_getters() {
        let ip_network = return_test_ipv4_network();
        assert_eq!(ip_network.get_network_address(), Ipv4Addr::new(192, 168, 0, 0));
        assert_eq!(ip_network.get_netmask(), 16);
        assert_eq!(ip_network.get_broadcast_address(), Ipv4Addr::new(192, 168, 255, 255));
        assert_eq!(ip_network.get_full_netmask(), Ipv4Addr::new(255, 255, 0, 0));
        assert_eq!(ip_network.supernet(), Ipv4Network::from(Ipv4Addr::new(192, 168, 0, 0), 15).unwrap());
        assert_eq!(ip_network.hosts().len(), 256 * 256 - 2);
    }

    #[test]
    fn test_ipv4_network_iterator() {
        let ip_network = return_test_ipv4_network();
        assert_eq!(ip_network.into_iter().len(), 256 * 256);
    }

    #[test]
    fn test_ipv4_network_iterator_for() {
        let mut i = 0;
        for _ in return_test_ipv4_network() {
            i += 1;
        }
        assert_eq!(i, 256 * 256);
    }

    #[test]
    fn test_ipv4_network_contains() {
        let ip_network = return_test_ipv4_network();
        assert!(!ip_network.contains(Ipv4Addr::new(192, 167, 255, 255)));
        assert!(ip_network.contains(Ipv4Addr::new(192, 168, 0, 0)));
        assert!(ip_network.contains(Ipv4Addr::new(192, 168, 255, 255)));
        assert!(!ip_network.contains(Ipv4Addr::new(192, 169, 0, 0)));
    }

    #[test]
    fn test_ipv4_network_subnets() {
        let ip_network = return_test_ipv4_network();
        let mut subnets = ip_network.subnets();
        assert_eq!(subnets.len(), 2);
        assert_eq!(subnets.next().unwrap(), Ipv4Network::from(Ipv4Addr::new(192, 168, 0, 0), 17).unwrap());
        assert_eq!(subnets.next().unwrap(), Ipv4Network::from(Ipv4Addr::new(192, 168, 128, 0), 17).unwrap());
        assert!(subnets.next().is_none());
    }

    #[test]
    fn test_ipv4_network_subnets_with_prefix() {
        let ip_network = return_test_ipv4_network();
        let mut subnets = ip_network.subnets_with_prefix(18);
        assert_eq!(subnets.len(), 4);
        assert_eq!(subnets.next().unwrap(), Ipv4Network::from(Ipv4Addr::new(192, 168, 0, 0), 18).unwrap());
        assert_eq!(subnets.next().unwrap(), Ipv4Network::from(Ipv4Addr::new(192, 168, 64, 0), 18).unwrap());
        assert_eq!(subnets.next().unwrap(), Ipv4Network::from(Ipv4Addr::new(192, 168, 128, 0), 18).unwrap());
        assert_eq!(subnets.next().unwrap(), Ipv4Network::from(Ipv4Addr::new(192, 168, 192, 0), 18).unwrap());
        assert!(subnets.next().is_none());
    }

    #[test]
    fn test_ipv4_network_parse() {
        let ip_network: Ipv4Network = "192.168.0.0/16".parse().unwrap();
        assert_eq!(ip_network, return_test_ipv4_network());
    }

    #[test]
    fn test_ipv4_network_format() {
        let ip_network = return_test_ipv4_network();
        assert_eq!(format!("{}", ip_network), "192.168.0.0/16");
    }

    #[test]
    fn test_ipv4_network_cmd_different_ip() {
        let a = Ipv4Network::from(Ipv4Addr::new(127, 0, 0, 0), 8).unwrap();
        let b = Ipv4Network::from(Ipv4Addr::new(128, 0, 0, 0), 8).unwrap();
        assert!(b > a);
    }

    #[test]
    fn test_ipv4_network_cmd_different_netmask() {
        let a = Ipv4Network::from(Ipv4Addr::new(127, 0, 0, 0), 8).unwrap();
        let b = Ipv4Network::from(Ipv4Addr::new(127, 0, 0, 0), 16).unwrap();
        assert!(b > a);
    }

    #[test]
    fn test_ipv4_network_hashmap() {
        use std::collections::HashMap;

        let ip = Ipv4Addr::new(127, 0, 0, 0);
        let network = Ipv4Network::from(ip, 8).unwrap();

        let mut networks = HashMap::new();
        networks.insert(network, 256);

        let ip_contains = Ipv4Addr::new(127, 0, 0, 0);
        let network_contains = Ipv4Network::from(ip_contains, 8).unwrap();
        assert!(networks.contains_key(&network_contains));

        let ip_not_contains = Ipv4Addr::new(127, 0, 0, 0);
        let network_not_contains = Ipv4Network::from(ip_not_contains, 9).unwrap();
        assert!(!networks.contains_key(&network_not_contains));
    }

    #[test]
    fn test_ipv4_network_summarize_address_range() {
        let networks = Ipv4Network::summarize_address_range(
            Ipv4Addr::new(194, 249, 198, 0),
            Ipv4Addr::new(194, 249, 198, 159)
        );
        assert_eq!(networks.len(), 2);
        assert_eq!(
            networks[0],
            Ipv4Network::from(Ipv4Addr::new(194, 249, 198, 0), 25).unwrap()
        );
        assert_eq!(
            networks[1],
            Ipv4Network::from(Ipv4Addr::new(194, 249, 198, 128), 27).unwrap()
        );
    }

    #[test]
    fn test_ipv4_network_summarize_address_range_whole_range() {
        let networks = Ipv4Network::summarize_address_range(
            Ipv4Addr::new(0, 0, 0, 0),
            Ipv4Addr::new(255, 255, 255, 255)
        );
        assert_eq!(networks.len(), 1);
        assert_eq!(
            networks[0],
            Ipv4Network::from(Ipv4Addr::new(0, 0, 0, 0), 0).unwrap()
        );
    }

    #[test]
    fn test_ipv6_network_from() {
        let ip = Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 0);
        let network = Ipv6Network::from(ip, 7).unwrap();
        assert_eq!(network.get_network_address(), Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 0));
        assert_eq!(network.get_netmask(), 7);
    }

    #[test]
    fn test_ipv6_network_contains() {
        let ip_network = return_test_ipv6_network();
        assert!(!ip_network.contains(Ipv6Addr::new(0x2001, 0x0db7, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff)));
        assert!(ip_network.contains(Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 0)));
        assert!(ip_network.contains(Ipv6Addr::new(0x2001, 0x0db8, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff)));
        assert!(!ip_network.contains(Ipv6Addr::new(0x2001, 0x0db9, 0, 0, 0, 0, 0, 0)));
    }

    #[test]
    fn test_ipv6_network_supernet() {
        let ip_network = return_test_ipv6_network();
        assert_eq!(ip_network.supernet(), Ipv6Network::from(Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 0), 31).unwrap());
    }

    #[test]
    fn test_ipv6_network_subnets() {
        let mut subnets = return_test_ipv6_network().subnets();
        assert_eq!(subnets.len(), 2);
        assert_eq!(subnets.next().unwrap(), Ipv6Network::from(Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 0), 33).unwrap());
        assert_eq!(subnets.next().unwrap(), Ipv6Network::from(Ipv6Addr::new(0x2001, 0x0db8, 0x8000, 0, 0, 0, 0, 0), 33).unwrap());
        assert!(subnets.next().is_none());
    }

    #[test]
    fn test_ipv6_network_subnets_with_prefix() {
        let ip_network = return_test_ipv6_network();
        let mut subnets = ip_network.subnets_with_prefix(34);
        assert_eq!(subnets.len(), 4);
        assert_eq!(subnets.next().unwrap(), Ipv6Network::from(Ipv6Addr::new(0x2001, 0x0db8, 0x0000, 0, 0, 0, 0, 0), 34).unwrap());
        assert_eq!(subnets.next().unwrap(), Ipv6Network::from(Ipv6Addr::new(0x2001, 0x0db8, 0x4000, 0, 0, 0, 0, 0), 34).unwrap());
        assert_eq!(subnets.next().unwrap(), Ipv6Network::from(Ipv6Addr::new(0x2001, 0x0db8, 0x8000, 0, 0, 0, 0, 0), 34).unwrap());
        assert_eq!(subnets.next().unwrap(), Ipv6Network::from(Ipv6Addr::new(0x2001, 0x0db8, 0xc000, 0, 0, 0, 0, 0), 34).unwrap());
        assert!(subnets.next().is_none());
    }

    #[test]
    fn test_ipv6_network_parse() {
        let ip_network: Ipv6Network = "2001:db8::/32".parse().unwrap();
        assert_eq!(ip_network, return_test_ipv6_network());
    }

    #[test]
    fn test_ipv6_network_format() {
        let ip_network = return_test_ipv6_network();
        assert_eq!(format!("{}", ip_network), "2001:db8::/32");
    }
}
