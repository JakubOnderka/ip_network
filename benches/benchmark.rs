#[macro_use]
extern crate criterion;

use std::net::{Ipv4Addr, Ipv6Addr};
use ip_network::{Ipv4Network, Ipv6Network};
use criterion::Criterion;

fn parse(c: &mut Criterion) {
    c.bench_function("parse ipv4", |b| {
        b.iter(|| "127.0.0.1/32".parse::<Ipv4Network>().unwrap())
    });
    c.bench_function("parse ipv6", |b| {
        b.iter(|| "::1/128".parse::<Ipv6Network>().unwrap())
    });
}

fn contains(c: &mut Criterion) {
    let ipv4_network = Ipv4Network::new(Ipv4Addr::new(127, 0, 0, 0), 8).unwrap();
    let ipv6_network = Ipv6Network::new(Ipv6Addr::new(127, 0, 0, 0, 0, 0, 0, 0), 16).unwrap();

    c.bench_function("contains ipv4", move |b| {
        b.iter(|| {
            ipv4_network.contains(Ipv4Addr::new(127, 0, 0, 1));
        })
    });
    c.bench_function("contains ipv6", move |b| {
        b.iter(|| {
            ipv6_network.contains(Ipv6Addr::new(127, 0, 0, 1, 0, 0, 0, 0));
        })
    });
}

criterion_group!(benches, parse, contains);
criterion_main!(benches);
