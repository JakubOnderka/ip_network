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
ip_network = "0.1"
```

this to your crate root:

```rust
extern crate ip_network;
```

and then you can use it like this:

```rust
use std::net::Ipv4Addr;
use ip_network::Ipv4Network;

let ip_network = Ipv4Network::from(Ipv4Addr::new(192, 168, 0, 0), 16).unwrap();
assert_eq!(format!("{}", ip_network), "192.168.0.0/16");
```

## Serde support

To enable serialization, just add `serde` feature to package in `Cargo.toml`:

```toml
[dependencies]
ip_network = { version = "0.1", features = ["serde"] }
``` 

## Postgres support

To enable support for [postgres](https://github.com/sfackler/rust-postgres) crate CIDR type, just add `postgres` feature to package in `Cargo.toml`:

```toml
[dependencies]
ip_network = { version = "0.1", features = ["postgres"] }
``` 

## Diesel support

To enable support for [diesel](https://diesel.rs) crate CIDR type, just add `diesel` feature to package in `Cargo.toml`:

```toml
[dependencies]
ip_network = { version = "0.1", features = ["diesel"] }
``` 