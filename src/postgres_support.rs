use {Ipv4Network, Ipv6Network, IpNetwork, IPV4_LENGTH, IPV6_LENGTH};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::error::Error;
use postgres::types::{FromSql, ToSql, Type, IsNull, CIDR};

const IPV4_TYPE: u8 = 2;
const IPV6_TYPE: u8 = 3;

impl ToSql for Ipv4Network {
    fn to_sql(&self, _: &Type, w: &mut Vec<u8>) -> Result<IsNull, Box<Error + Sync + Send>> {
        let ip_octets = self.network_address.octets();
        let mut bytes = [0; 8];
        bytes[0] = IPV4_TYPE;
        bytes[1] = self.netmask;
        bytes[2] = 1;
        bytes[3] = IPV4_LENGTH / 8;
        bytes[4] = ip_octets[0];
        bytes[5] = ip_octets[1];
        bytes[6] = ip_octets[2];
        bytes[7] = ip_octets[3];
        w.extend_from_slice(&bytes);

        Ok(IsNull::No)
    }

    accepts!(CIDR);
    to_sql_checked!();
}

impl ToSql for Ipv6Network {
    fn to_sql(&self, _: &Type, w: &mut Vec<u8>) -> Result<IsNull, Box<Error + Sync + Send>> {
        let mut bytes = [0; 4];
        bytes[0] = IPV6_TYPE;
        bytes[1] = self.netmask;
        bytes[2] = 1;
        bytes[3] = IPV6_LENGTH / 8;
        w.extend_from_slice(&bytes);

        let ip_octets = self.network_address.octets();
        w.extend_from_slice(&ip_octets);

        Ok(IsNull::No)
    }

    accepts!(CIDR);
    to_sql_checked!();
}

impl FromSql for Ipv4Network {
    fn from_sql(_: &Type, raw: &[u8]) -> Result<Ipv4Network, Box<Error + Sync + Send>> {
        if raw[0] != IPV4_TYPE {
            return Err("CIDR is not IP version 4".into())
        }

        if raw[2] != 1 {
            return Err("This field is not CIDR type, probably INET type".into())
        }

        if raw[3] != IPV4_LENGTH / 8 {
            return Err(format!("CIDR is IP version 4, but have bad length '{}'", raw[3]).into())
        }

        let network_address = Ipv4Addr::new(raw[4], raw[5], raw[6], raw[7]);
        let netmask = raw[1];
        Ok(Ipv4Network::from(network_address, netmask)?)
    }

    accepts!(CIDR);
}

impl FromSql for Ipv6Network {
    fn from_sql(_: &Type, raw: &[u8]) -> Result<Ipv6Network, Box<Error + Sync + Send>> {
        if raw[0] != IPV6_TYPE {
            return Err("CIDR is not IP version 6".into())
        }

        if raw[2] != 1 {
            return Err("This field is not CIDR type, probably INET type".into())
        }

        if raw[3] != IPV6_LENGTH / 8 {
            return Err(format!("CIDR is IP version 6, but have bad length '{}'", raw[3]).into())
        }

        let mut octets = [0; 16];
        octets.copy_from_slice(&raw[4..]);
        let network_address = Ipv6Addr::from(octets);

        let netmask = raw[1];
        Ok(Ipv6Network::from(network_address, netmask)?)
    }

    accepts!(CIDR);
}

impl FromSql for IpNetwork {
    fn from_sql(t: &Type, raw: &[u8]) -> Result<IpNetwork, Box<Error + Sync + Send>> {
        match raw[0] {
            IPV4_TYPE => Ok(IpNetwork::V4(Ipv4Network::from_sql(t, raw)?)),
            IPV6_TYPE => Ok(IpNetwork::V6(Ipv6Network::from_sql(t, raw)?)),
            _ => Err("CIDR is not IP version 4 or 6".into()),
        }
    }

    accepts!(CIDR);
}

impl ToSql for IpNetwork {
    fn to_sql(&self, t: &Type, w: &mut Vec<u8>) -> Result<IsNull, Box<Error + Sync + Send>> {
        match *self {
            IpNetwork::V4(ref network) => network.to_sql(t, w),
            IpNetwork::V6(ref network) => network.to_sql(t, w),
        }
    }

    accepts!(CIDR);
    to_sql_checked!();
}