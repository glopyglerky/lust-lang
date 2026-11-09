// std::net address types in source (W-STD-C phase 2): construction,
// text with padding, parsing (accepts and rejects), predicates,
// ordering, conversions.
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::str::FromStr;

fn v4s(s: &str) {
    match Ipv4Addr::from_str(s) {
        Ok(a) => println!("v4 {s:?} {a} {:?} bits={}", a.octets(), a.to_bits()),
        Err(e) => println!("v4 {s:?} ERR {e:?} {e}"),
    }
}

fn v6s(s: &str) {
    match Ipv6Addr::from_str(s) {
        Ok(a) => println!("v6 {s:?} {a} {:?} to4={:?} mapped={:?}", a.segments(), a.to_ipv4(), a.to_ipv4_mapped()),
        Err(e) => println!("v6 {s:?} ERR {e:?} {e}"),
    }
}

fn main() {
    for s in ["127.0.0.1", "0.0.0.0", "255.255.255.255", "01.2.3.4", "1.2.3", "1.2.3.256", "1..2.3", "1.2.3.4x", ""] {
        v4s(s);
    }
    for s in ["::", "::1", "1::", "1:0:0:2::3", "::ffff:1.2.3.4", "::1.2.3.4", "1:2:3:4:5:6:1.2.3.4", "1::2::3", "12345::", "fe80::1%eth0", "1:2:3:4:5:6::8", "1:2:3:4:5:6:7::8"] {
        v6s(s);
    }
    let a = Ipv4Addr::new(192, 168, 0, 1);
    println!(
        "{} priv={} ll={} mc={} doc={} loop={} uns={} bc={} lo={} un={} br={}",
        a, a.is_private(), a.is_link_local(), a.is_multicast(), a.is_documentation(), a.is_loopback(),
        a.is_unspecified(), a.is_broadcast(), Ipv4Addr::LOCALHOST, Ipv4Addr::UNSPECIFIED, Ipv4Addr::BROADCAST
    );
    println!("compat={} mapped={}", a.to_ipv6_compatible(), a.to_ipv6_mapped());
    let b = Ipv6Addr::new(0x2001, 0xdb8, 0x85a3, 0, 0, 0x8a2e, 0x370, 0x7334);
    println!("{} dbg={b:?} mc={} bits={:x}", b, b.is_multicast(), b.to_bits());
    println!("[{:>18}] [{:<18}] [{:^18}]", a, b, Ipv6Addr::LOCALHOST);
    println!("ord {} {} {}", a < Ipv4Addr::new(192, 168, 0, 2), IpAddr::V4(a) < IpAddr::V6(b), Ipv6Addr::from_bits(2) > Ipv6Addr::from_bits(1));
    let sa4 = SocketAddrV4::from_str("1.2.3.4:80").unwrap();
    let mut sa6 = SocketAddrV6::from_str("[fe80::1%25]:443").unwrap();
    sa6.set_port(8443);
    sa6.set_flowinfo(7);
    println!("{} {} {} {:?}", sa4, sa6, sa6.flowinfo(), sa4.ip());
    let mut s = SocketAddr::from((a, 22));
    s.set_ip(IpAddr::V6(b));
    println!("{} v4={} v6={} port={} ip={}", s, s.is_ipv4(), s.is_ipv6(), s.port(), s.ip());
    for t in ["1.2.3.4:80", "[::1]:80", "x", "1.2.3.4:99999", "[::1]"] {
        match SocketAddr::from_str(t) {
            Ok(x) => println!("sa {t:?} {x}"),
            Err(e) => println!("sa {t:?} ERR {e:?} {e}"),
        }
    }
    println!("{:?} {:?}", IpAddr::from([9u16, 0, 0, 0, 0, 0, 0, 1]), Ipv4Addr::from(0x7f000001u32));
}
