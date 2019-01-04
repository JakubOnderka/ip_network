ip_network
========

IPv4 and IPv6 network structs.

[![Build Status](https://travis-ci.org/JakubOnderka/ip_network.svg?branch=master)](https://travis-ci.org/JakubOnderka/ip_network)
[![Coverage Status](https://coveralls.io/repos/github/JakubOnderka/ip_network/badge.svg?branch=master)](https://coveralls.io/github/JakubOnderka/ip_network?branch=master)
[![Crates.io](https://img.shields.io/crates/v/ip_network.svg)](https://crates.io/crates/ip_network)

- [Documentation](https://docs.rs/ip_network)

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
ip_network = "0.3"
```

this to your crate root (necessary just when your project is Rust 2015 edition):

```rust
extern crate ip_network;
```

and then you can use it like this:

```rust
use std::net::Ipv4Addr;
use ip_network::Ipv4Network;

let ip_network = Ipv4Network::new(Ipv4Addr::new(192, 168, 0, 0), 16).unwrap();
assert_eq!("192.168.0.0/16", ip_network.to_string());
```

Minimal required version of Rust compiler is:
- 1.31 for version 0.3 and newer (because of 2018 edition),
- 1.26 for version 0.2 (because of support `u128` data type),
- for older compiler you can use 0.1 version.   

## Serde support

To enable serialization, just add `serde` feature to package in `Cargo.toml`:

```toml
[dependencies]
ip_network = { version = "0.3", features = ["serde"] }
``` 

## Postgres support

To enable support for [postgres](https://github.com/sfackler/rust-postgres) crate CIDR type, just add `postgres` feature to package in `Cargo.toml`:

```toml
[dependencies]
ip_network = { version = "0.3", features = ["postgres"] }
``` 

## Diesel support

To enable support for [diesel](https://diesel.rs) CIDR type for PostgreSQL, just add `diesel` feature to package in `Cargo.toml`:

```toml
[dependencies]
ip_network = { version = "0.3", features = ["diesel"] }
``` 

You can then use `ip_network::diesel_support::PqCidrExtensionMethods` trait for CIDR operators support.