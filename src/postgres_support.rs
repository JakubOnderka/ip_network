use std::error::Error;
use postgres::types::{FromSql, IsNull, ToSql, Type, CIDR};
use postgres::{accepts, to_sql_checked};
use crate::{IpNetwork, Ipv4Network, Ipv6Network};
use crate::postgres_common;

type PostgresResult<T> = Result<T, Box<Error + Sync + Send>>;

impl FromSql for Ipv4Network {
    fn from_sql(_: &Type, raw: &[u8]) -> PostgresResult<Ipv4Network> {
        postgres_common::from_sql_ipv4_network(raw)
    }

    accepts!(CIDR);
}

impl FromSql for Ipv6Network {
    fn from_sql(_: &Type, raw: &[u8]) -> PostgresResult<Ipv6Network> {
        postgres_common::from_sql_ipv6_network(raw)
    }

    accepts!(CIDR);
}

impl FromSql for IpNetwork {
    fn from_sql(t: &Type, raw: &[u8]) -> PostgresResult<IpNetwork> {
        match raw[0] {
            postgres_common::IPV4_TYPE => Ok(IpNetwork::V4(Ipv4Network::from_sql(t, raw)?)),
            postgres_common::IPV6_TYPE => Ok(IpNetwork::V6(Ipv6Network::from_sql(t, raw)?)),
            _ => Err("CIDR is not IP version 4 or 6".into()),
        }
    }

    accepts!(CIDR);
}

impl ToSql for Ipv4Network {
    fn to_sql(&self, _: &Type, w: &mut Vec<u8>) -> PostgresResult<IsNull> {
        let bytes = postgres_common::to_sql_ipv4_network(*self);
        w.extend_from_slice(&bytes);

        Ok(IsNull::No)
    }

    accepts!(CIDR);
    to_sql_checked!();
}

impl ToSql for Ipv6Network {
    fn to_sql(&self, _: &Type, w: &mut Vec<u8>) -> PostgresResult<IsNull> {
        let bytes = postgres_common::to_sql_ipv6_network(*self);
        w.extend_from_slice(&bytes);

        Ok(IsNull::No)
    }

    accepts!(CIDR);
    to_sql_checked!();
}

impl ToSql for IpNetwork {
    fn to_sql(&self, t: &Type, w: &mut Vec<u8>) -> PostgresResult<IsNull> {
        match *self {
            IpNetwork::V4(ref network) => network.to_sql(t, w),
            IpNetwork::V6(ref network) => network.to_sql(t, w),
        }
    }

    accepts!(CIDR);
    to_sql_checked!();
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr};
    use postgres::types::{FromSql, ToSql, CIDR};
    use crate::{IpNetwork, Ipv4Network, Ipv6Network};

    fn return_test_ipv4_network() -> Ipv4Network {
        Ipv4Network::new(Ipv4Addr::new(192, 168, 0, 0), 16).unwrap()
    }

    fn return_test_ipv6_network() -> Ipv6Network {
        Ipv6Network::new(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0), 32).unwrap()
    }

    #[test]
    fn ivp4_to_sql() {
        let ip_network = return_test_ipv4_network();
        let mut output = vec![];
        assert!(ip_network.to_sql(&CIDR, &mut output).is_ok());
        assert_eq!(2, output[0]);
        assert_eq!(16, output[1]);
        assert_eq!(1, output[2]);
        assert_eq!(4, output[3]);
        assert_eq!(192, output[4]);
        assert_eq!(168, output[5]);
        assert_eq!(0, output[6]);
        assert_eq!(0, output[7]);
    }

    #[test]
    fn ivp4_both_direction() {
        let ip_network = return_test_ipv4_network();
        let mut output = vec![];

        assert!(ip_network.to_sql(&CIDR, &mut output).is_ok());

        let result = Ipv4Network::from_sql(&CIDR, &output);
        assert!(result.is_ok());

        let ip_network_converted = result.unwrap();
        assert_eq!(ip_network, ip_network_converted);
    }

    #[test]
    fn ivp6_to_sql() {
        let ip_network = return_test_ipv6_network();
        let mut output = vec![];
        assert!(ip_network.to_sql(&CIDR, &mut output).is_ok());
        assert_eq!(3, output[0]);
        assert_eq!(32, output[1]);
        assert_eq!(1, output[2]);
        assert_eq!(16, output[3]);
        assert_eq!(0x20, output[4]);
        assert_eq!(0x01, output[5]);
        assert_eq!(0x0d, output[6]);
        assert_eq!(0xb8, output[7]);
        for i in 8..20 {
            assert_eq!(0, output[i]);
        }
    }

    #[test]
    fn ivp6_both_direction() {
        let ip_network = return_test_ipv6_network();
        let mut output = vec![];

        assert!(ip_network.to_sql(&CIDR, &mut output).is_ok());

        let result = Ipv6Network::from_sql(&CIDR, &output);
        assert!(result.is_ok());

        let ip_network_converted = result.unwrap();
        assert_eq!(ip_network, ip_network_converted);
    }

    #[test]
    fn ipnetwork_to_sql_v4() {
        let ip_network = IpNetwork::V4(return_test_ipv4_network());
        let mut output = vec![];
        assert!(ip_network.to_sql(&CIDR, &mut output).is_ok());
    }

    #[test]
    fn ipnetwork_to_sql_v6() {
        let ip_network = IpNetwork::V6(return_test_ipv6_network());
        let mut output = vec![];
        assert!(ip_network.to_sql(&CIDR, &mut output).is_ok());
    }
}
