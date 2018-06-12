use postgres_common;

use std::error::Error;
use std::io::prelude::*;
use {IpNetwork, Ipv4Network, Ipv6Network};

use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::Cidr;

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
        out.write_all(&data).map(|_| IsNull::No).map_err(Into::into)
    }
}

impl ToSql<Cidr, Pg> for Ipv6Network {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        let data = postgres_common::to_sql_ipv6_network(self);
        out.write_all(&data).map(|_| IsNull::No).map_err(Into::into)
    }
}

impl ToSql<Cidr, Pg> for IpNetwork {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        match *self {
            IpNetwork::V4(ref network) => {
                let data = postgres_common::to_sql_ipv4_network(network);
                out.write_all(&data).map(|_| IsNull::No).map_err(Into::into)
            },
            IpNetwork::V6(ref network) => {
                let data = postgres_common::to_sql_ipv6_network(network);
                out.write_all(&data).map(|_| IsNull::No).map_err(Into::into)
            }
        }
    }
}

#[allow(dead_code)]
mod foreign_derives {
    use super::*;

    #[derive(FromSqlRow, AsExpression)]
    #[diesel(foreign_derive)]
    #[sql_type = "Cidr"]
    struct IpNetworkProxy(IpNetwork);

    #[derive(FromSqlRow, AsExpression)]
    #[diesel(foreign_derive)]
    #[sql_type = "Cidr"]
    struct Ipv4NetworkProxy(Ipv4Network);

    #[derive(FromSqlRow, AsExpression)]
    #[diesel(foreign_derive)]
    #[sql_type = "Cidr"]
    struct Ipv6NetworkProxy(Ipv6Network);
}

#[cfg(test)]
mod tests {
    use super::{IpNetwork, Ipv4Network, Ipv6Network};

    table! {
        test {
            id -> Integer,
            ip_network -> Cidr,
            ipv4_network -> Cidr,
            ipv6_network -> Cidr,
        }
    }

    #[derive(Insertable)]
    #[table_name="test"]
    pub struct NewPost{
        pub id: i32,
        pub ip_network: IpNetwork,
        pub ipv4_network: Ipv4Network,
        pub ipv6_network: Ipv6Network,
    }
}
