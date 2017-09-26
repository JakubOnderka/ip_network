use std;
use std::net::Ipv6Addr;
use extprim;
use extprim::u128::u128;

pub fn bit_length(number: u32) -> u8 {
    32 - number.leading_zeros() as u8
}

pub fn get_bite_mask(mask: u8) -> u32 {
    !std::u32::MAX.checked_shr(mask as u32).unwrap_or(0)
}

pub fn get_bite_mask_u128(mask: u8) -> u128 {
    !extprim::u128::MAX.checked_shr(mask as u32).unwrap_or(extprim::u128::MIN)
}

pub fn split_ip_netmask(input: &str) -> Option<(&str, &str)> {
    let delimiter = match input.find('/') {
        Some(pos) => pos,
        None => return None,
    };
    let (ip, mask) = input.split_at(delimiter);
    let mask = &mask[1..];

    if ip.is_empty() || mask.is_empty() {
        None
    } else {
        Some((ip, mask))
    }
}

pub fn ipv6addr_to_u128(ip: Ipv6Addr) -> u128 {
    let octets = ip.octets();
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

fn transform_u64_to_array_of_u16(x: u64) -> [u16; 4] {
    let b1: u16 = ((x >> 48) & 0xffff) as u16;
    let b2: u16 = ((x >> 32) & 0xffff) as u16;
    let b3: u16 = ((x >> 16) & 0xffff) as u16;
    let b4: u16 = (x & 0xffff) as u16;
    [b1, b2, b3, b4]
}

pub fn u128_to_ipv6addr(input: u128) -> Ipv6Addr {
    let hi = transform_u64_to_array_of_u16(input.high64());
    let lo = transform_u64_to_array_of_u16(input.low64());
    Ipv6Addr::new(hi[0], hi[1], hi[2], hi[3], lo[0], lo[1], lo[2], lo[3])
}

#[cfg(test)]
mod tests {
    use std;
    use helpers::{get_bite_mask, ipv6addr_to_u128, u128_to_ipv6addr, split_ip_netmask};
    use std::net::Ipv6Addr;

    #[test]
    fn get_bite_mask_32() {
        assert_eq!(std::u32::MAX, get_bite_mask(32));
    }

    #[test]
    fn get_bite_mask_0() {
        assert_eq!(0, get_bite_mask(0));
    }

    #[test]
    fn test_u128_ipv6_transform() {
        let ip = Ipv6Addr::new(1, 2, 3, 4, 5, 6, 7, 8);
        let ip_u128 = ipv6addr_to_u128(ip);
        assert_eq!(u128_to_ipv6addr(ip_u128), ip);
    }

    #[test]
    fn test_split_ip_netmask() {
        let (ip, netmask) = split_ip_netmask("192.168.1.1/24").unwrap();
        assert_eq!("192.168.1.1", ip);
        assert_eq!("24", netmask);
    }

    #[test]
    fn test_split_ip_netmask_invalid_1() {
        let a = split_ip_netmask("ab");
        assert!(a.is_none());
    }

    #[test]
    fn test_split_ip_netmask_invalid_2() {
        let a = split_ip_netmask("/");
        assert!(a.is_none());
    }

    #[test]
    fn test_split_ip_netmask_invalid_3() {
        let a = split_ip_netmask("192.168.1.1/");
        assert!(a.is_none());
    }
}