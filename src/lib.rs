#[cfg(feature = "diesel")]
#[macro_use]
extern crate diesel;

#[cfg(feature = "diesel")]
/// Support for Diesel PostgreSQL CIDR type
pub mod diesel_support;
mod helpers;
mod ip_network;
mod ipv4_network;
mod ipv6_network;
/// `Ipv4RangeIterator`, `Ipv4NetworkIterator` and `Ipv6NetworkIterator`
pub mod iterator;
#[cfg(any(feature = "diesel", feature = "postgres"))]
mod postgres_common;
#[cfg(feature = "postgres")]
mod postgres_support;
#[cfg(feature = "serde")]
mod serde_support;

use std::error::Error;
use std::fmt;

pub use self::ip_network::IpNetwork;
pub use self::ipv4_network::Ipv4Network;
pub use self::ipv6_network::{Ipv6MulticastScope, Ipv6Network};

/// Errors when creating new IPv4 or IPv6 networks
#[derive(Debug, PartialEq)]
pub enum IpNetworkError {
    /// Network mask is bigger than possible for given IP version (32 for IPv4, 128 for IPv6)
    NetmaskError(u8),
    /// Host bits are set in given network IP address
    HostBitsSet,
}

impl Error for IpNetworkError {}

impl fmt::Display for IpNetworkError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let description = match *self {
            IpNetworkError::NetmaskError(_) => "invalid netmask",
            IpNetworkError::HostBitsSet => "IP network address has host bits set",
        };
        write!(fmt, "{}", description)
    }
}

/// Errors from IPv4 or IPv6 network parsing
#[derive(Debug, PartialEq)]
pub enum IpNetworkParseError {
    /// Network mask is not valid integer between 0-255
    InvalidNetmaskFormat,
    /// Network address has invalid format (not X/Y)
    InvalidFormatError,
    /// Invalid IP address syntax (IPv4 or IPv6)
    AddrParseError,
    /// Error when creating new IPv4 or IPv6 networks
    IpNetworkError(IpNetworkError),
}

impl Error for IpNetworkParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match *self {
            IpNetworkParseError::IpNetworkError(ref ip_network_error) => Some(ip_network_error),
            _ => None,
        }
    }
}

impl fmt::Display for IpNetworkParseError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IpNetworkParseError::InvalidNetmaskFormat => write!(fmt, "invalid netmask format"),
            IpNetworkParseError::InvalidFormatError => write!(fmt, "invalid format"),
            IpNetworkParseError::AddrParseError => write!(fmt, "invalid IP address syntax"),
            IpNetworkParseError::IpNetworkError(ref ip_network_error) => {
                write!(fmt, "{}", ip_network_error)
            }
        }
    }
}
