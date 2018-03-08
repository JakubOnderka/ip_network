use postgres_common;

use {Ipv4Network, Ipv6Network, IpNetwork};
use std::io::prelude::*;
use std::error::Error;

use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::Cidr;
use diesel::pg::Pg;

type BoxedError = Box<Error + Sync + Send>;

impl FromSql<Cidr, Pg> for Ipv4Network {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        let raw = bytes.ok_or::<BoxedError>("Input for Ipv4Network::from_sql is empty".into())?;
        postgres_common::from_sql_ipv4_network(raw)
    }
}

impl FromSql<Cidr, Pg> for Ipv6Network {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        let raw = bytes.ok_or::<BoxedError>("Input for Ipv6Network::from_sql is empty".into())?;
        postgres_common::from_sql_ipv6_network(raw)
    }
}

impl FromSql<Cidr, Pg> for IpNetwork {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        let raw = bytes.ok_or::<BoxedError>("Input for IpNetwork::from_sql is empty".into())?;
        match raw[0] {
            postgres_common::IPV4_TYPE => Ok(IpNetwork::V4(Ipv4Network::from_sql(bytes)?)),
            postgres_common::IPV6_TYPE => Ok(IpNetwork::V6(Ipv6Network::from_sql(bytes)?)),
            _ => Err("CIDR is not IP version 4 or 6".into()),
        }
    }
}

impl ToSql<Cidr, Pg> for Ipv4Network {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        let data = postgres_common::to_sql_ipv4_network(self);
        out.write_all(&data)
            .map(|_| IsNull::No)
            .map_err(Into::into)
    }
}

impl ToSql<Cidr, Pg> for Ipv6Network {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        let data = postgres_common::to_sql_ipv6_network(self);
        out.write_all(&data)
            .map(|_| IsNull::No)
            .map_err(Into::into)
    }
}

impl ToSql<Cidr, Pg> for IpNetwork {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match *self {
            IpNetwork::V4(ref network) => network.to_sql(out),
            IpNetwork::V6(ref network) => network.to_sql(out),
        }
    }
}