use std::net::{Ipv4Addr, Ipv6Addr, IpAddr};
use {Ipv4Network, Ipv6Network, IpNetwork};
use treebitmap::{self, IpLookupTable, IpLookupTableOps};

pub struct Table<T> {
    ipv4: IpLookupTable<Ipv4Addr, T>,
    ipv6: IpLookupTable<Ipv6Addr, T>,
}

impl<T> Table<T> {
    pub fn new() -> Self {
        Self {
            ipv4: IpLookupTable::new(),
            ipv6: IpLookupTable::new(),
        }
    }

    pub fn with_capacity(ipv4_size: usize, ipv6_size: usize) -> Self {
        Self {
            ipv4: IpLookupTable::with_capacity(ipv4_size),
            ipv6: IpLookupTable::with_capacity(ipv6_size),
        }
    }

    /// Insert a value for the IpNetwork. If prefix existed previously, the old value is returned.
    ///
    /// # Example
    ///
    /// ```
    /// use ip_network::table::Table;
    /// use ip_network::Ipv6Network;
    /// use std::net::Ipv6Addr;
    ///
    /// let mut table: Table<&str> = Table::new();
    /// let network = Ipv6Network::from(Ipv6Addr::new(0x2001, 0xdb8, 0xdead, 0xbeef, 0, 0, 0, 0), 64).unwrap();
    ///
    /// assert_eq!(table.insert(network.clone(), "foo"), None);
    /// // Insert duplicate
    /// assert_eq!(table.insert(network.clone(), "bar"), Some("foo"));
    /// // Value is replaced
    /// assert_eq!(table.insert(network, "null"), Some("bar"));
    /// ```
    pub fn insert<N: Into<IpNetwork>>(&mut self, network: N, data: T) -> Option<T> {
        match network.into() {
            IpNetwork::V4(ipv4_network) => {
                self.ipv4.insert(ipv4_network.network_address, ipv4_network.netmask as u32, data)
            },
            IpNetwork::V6(ipv6_network) => {
                self.ipv6.insert(ipv6_network.network_address, ipv6_network.netmask as u32, data)
            },
        }
    }

    /// Insert a value for the IpNetwork. If prefix existed previously, the old value is returned.
    ///
    /// # Example
    ///
    /// ```
    /// use ip_network::table::Table;
    /// use ip_network::Ipv6Network;
    /// use std::net::Ipv6Addr;
    ///
    /// let mut table: Table<&str> = Table::new();
    /// let network = Ipv6Network::from(Ipv6Addr::new(0x2001, 0xdb8, 0xdead, 0xbeef, 0, 0, 0, 0), 64).unwrap();
    ///
    /// assert_eq!(table.insert(network.clone(), "foo"), None);
    /// // Remove network from table
    /// assert_eq!(table.remove(network.clone()), Some("foo"));
    /// // Network is removed
    /// assert_eq!(table.exact_match(network), None);
    /// ```
    pub fn remove<N: Into<IpNetwork>>(&mut self, network: N) -> Option<T> {
        match network.into() {
            IpNetwork::V4(ipv4_network) => {
                self.ipv4.remove(ipv4_network.network_address, ipv4_network.netmask as u32)
            },
            IpNetwork::V6(ipv6_network) => {
                self.ipv6.remove(ipv6_network.network_address, ipv6_network.netmask as u32)
            },
        }
    }

    pub fn exact_match<N: Into<IpNetwork>>(&self, network: N) -> Option<&T> {
        match network.into() {
            IpNetwork::V4(ipv4_network) => {
                self.ipv4.exact_match(ipv4_network.network_address, ipv4_network.netmask as u32)
            },
            IpNetwork::V6(ipv6_network) => {
                self.ipv6.exact_match(ipv6_network.network_address, ipv6_network.netmask as u32)
            },
        }
    }

    pub fn longest_match<A: Into<IpAddr>>(&self, ip: A) -> Option<(IpNetwork, &T)> {
        match ip.into() {
            IpAddr::V4(ipv4) => self.longest_match_ipv4(ipv4),
            IpAddr::V6(ipv6) => self.longest_match_ipv6(ipv6),
        }
    }

    #[inline]
    pub fn longest_match_ipv4(&self, ip: Ipv4Addr) -> Option<(IpNetwork, &T)> {
        match self.ipv4.longest_match(ip) {
            Some((addr, mask, data)) => Some((
                IpNetwork::V4(Ipv4Network::from(addr, mask as u8).unwrap()),
                data
            )),
            None => None,
        }
    }

    #[inline]
    pub fn longest_match_ipv6(&self, ip: Ipv6Addr) -> Option<(IpNetwork, &T)> {
        match self.ipv6.longest_match(ip) {
            Some((addr, mask, data)) => Some((
                IpNetwork::V6(Ipv6Network::from(addr, mask as u8).unwrap()),
                data
            )),
            None => None,
        }
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            ipv4: self.ipv4.iter(),
            ipv6: self.ipv6.iter(),
        }
    }
}

pub struct Iter<'a, T: 'a> {
    ipv4: treebitmap::Iter<'a, Ipv4Addr, T>,
    ipv6: treebitmap::Iter<'a, Ipv6Addr, T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (IpNetwork, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        match self.ipv4.next() {
            Some((addr, mask, data)) => Some((
                IpNetwork::V4(Ipv4Network::from(addr, mask as u8).unwrap()),
                data
            )),
            None => {
                match self.ipv6.next() {
                    Some((addr, mask, data)) => Some((
                        IpNetwork::V6(Ipv6Network::from(addr, mask as u8).unwrap()),
                        data
                    )),
                    None => None,
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use table::Table;
    use std::net::{Ipv4Addr, Ipv6Addr};
    use {Ipv4Network, Ipv6Network};

    #[test]
    fn test() {
        let mut table: Table<u32> = Table::new();
        table.insert(Ipv4Network::from(Ipv4Addr::new(127, 0, 0, 0), 16).unwrap(), 1);
        table.insert(Ipv6Network::from(Ipv6Addr::new(1, 2, 3, 4, 5, 6, 7, 8), 128).unwrap(), 1);
    }
}

