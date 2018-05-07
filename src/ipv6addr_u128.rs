use std::net::Ipv6Addr;
use extprim::u128::u128;

fn transform_u64_to_array_of_u16(x: u64) -> [u16; 4] {
    let b1: u16 = ((x >> 48) & 0xffff) as u16;
    let b2: u16 = ((x >> 32) & 0xffff) as u16;
    let b3: u16 = ((x >> 16) & 0xffff) as u16;
    let b4: u16 = (x & 0xffff) as u16;
    [b1, b2, b3, b4]
}

pub trait Ipv6AddrU128 {
    /// Create `Ipv6Addr` from `extprim` u128
    fn from_u128(input: u128) -> Ipv6Addr;
    /// Convert `Ipv6Addr` to `extprtim` u128
    fn to_u128(&self) -> u128;
}

impl Ipv6AddrU128 for Ipv6Addr {
    fn from_u128(input: u128) -> Self {
        let hi = transform_u64_to_array_of_u16(input.high64());
        let lo = transform_u64_to_array_of_u16(input.low64());
        Self::new(hi[0], hi[1], hi[2], hi[3], lo[0], lo[1], lo[2], lo[3])
    }

    fn to_u128(&self) -> u128 {
        let octets = self.octets();
        let hi: u64 = (octets[0] as u64) << 56 |
            (octets[1] as u64) << 48 |
            (octets[2] as u64) << 40 |
            (octets[3] as u64) << 32 |
            (octets[4] as u64) << 24 |
            (octets[5] as u64) << 16 |
            (octets[6] as u64) << 8 |
            octets[7] as u64;
        let lo: u64 = (octets[8] as u64) << 56 |
            (octets[9] as u64) << 48 |
            (octets[10] as u64) << 40 |
            (octets[11] as u64) << 32 |
            (octets[12] as u64) << 24 |
            (octets[13] as u64) << 16 |
            (octets[14] as u64) << 8 |
            octets[15] as u64;
        u128::from_parts(hi, lo)
    }
}

#[cfg(test)]
mod test {
    use std::net::Ipv6Addr;
    use Ipv6AddrU128;

    #[test]
    fn test_u128_ipv6_transform() {
        let ip = Ipv6Addr::new(1, 2, 3, 4, 5, 6, 7, 8);
        let ip_u128 = ip.to_u128();
        assert_eq!(Ipv6Addr::from_u128(ip_u128), ip);
    }
}