use std::error::Error;
use std::io::prelude::*;
use diesel::deserialize::{self, FromSql};
use diesel::expression::{AsExpression, Expression};
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::Cidr;
use crate::{IpNetwork, Ipv4Network, Ipv6Network};
use crate::postgres_common;

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
            }
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

diesel_infix_operator!(IsContainedBy, " << ", backend: Pg);
diesel_infix_operator!(IsContainedByOrEquals, " <<= ", backend: Pg);
diesel_infix_operator!(Contains, " >> ", backend: Pg);
diesel_infix_operator!(ContainsOrEquals, " >>= ", backend: Pg);
diesel_infix_operator!(ContainsOrIsContainedBy, " && ", backend: Pg);

/// Support for PostgreSQL Network Address Operators for Diesel
///
/// See [PostgreSQL documentation for details](https://www.postgresql.org/docs/current/static/functions-net.html).
pub trait PqCidrExtensionMethods: Expression<SqlType = Cidr> + Sized {
    /// Creates a SQL `<<` expression.
    fn is_contained_by<T>(self, other: T) -> IsContainedBy<Self, T::Expression>
    where
        T: AsExpression<Self::SqlType>,
    {
        IsContainedBy::new(self, other.as_expression())
    }

    /// Creates a SQL `<<=` expression.
    fn is_contained_by_or_equals<T>(self, other: T) -> IsContainedByOrEquals<Self, T::Expression>
    where
        T: AsExpression<Self::SqlType>,
    {
        IsContainedByOrEquals::new(self, other.as_expression())
    }

    /// Creates a SQL `>>` expression.
    fn contains<T>(self, other: T) -> Contains<Self, T::Expression>
    where
        T: AsExpression<Self::SqlType>,
    {
        Contains::new(self, other.as_expression())
    }

    /// Creates a SQL `>>=` expression.
    fn contains_or_equals<T>(self, other: T) -> ContainsOrEquals<Self, T::Expression>
    where
        T: AsExpression<Self::SqlType>,
    {
        ContainsOrEquals::new(self, other.as_expression())
    }

    /// Creates a SQL `&&` expression.
    fn contains_or_is_contained_by<T>(
        self,
        other: T,
    ) -> ContainsOrIsContainedBy<Self, T::Expression>
    where
        T: AsExpression<Self::SqlType>,
    {
        ContainsOrIsContainedBy::new(self, other.as_expression())
    }
}

impl<T> PqCidrExtensionMethods for T
where
    T: Expression<SqlType = Cidr>,
{
}

#[cfg(test)]
mod tests {
    use super::PqCidrExtensionMethods;
    use super::{IpNetwork, Ipv4Network, Ipv6Network};
    use std::net::Ipv4Addr;

    table! {
        test {
            id -> Integer,
            ip_network -> Cidr,
            ipv4_network -> Cidr,
            ipv6_network -> Cidr,
        }
    }

    #[derive(Insertable)]
    #[table_name = "test"]
    pub struct NewPost {
        pub id: i32,
        pub ip_network: IpNetwork,
        pub ipv4_network: Ipv4Network,
        pub ipv6_network: Ipv6Network,
    }

    #[test]
    fn operators() {
        let ip = IpNetwork::new(Ipv4Addr::new(127, 0, 0, 1), 32).unwrap();
        test::ip_network.is_contained_by(&ip);
        test::ip_network.is_contained_by_or_equals(&ip);
        test::ip_network.contains(&ip);
        test::ip_network.contains_or_equals(&ip);
        test::ip_network.contains_or_is_contained_by(&ip);
    }
}
