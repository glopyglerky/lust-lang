// feature: std.net_addr
// form: IP and socket address values, text, parsing, predicates
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::str::FromStr;
fn main() {
    let a = Ipv4Addr::new(127, 0, 0, 1);
    let b = Ipv6Addr::from_str("2001:db8::2:1").unwrap();
    let sa = SocketAddr::from_str("[::1]:8080").unwrap();
    let bad = Ipv4Addr::from_str("01.2.3.4").unwrap_err();
    println!("{} {} {} {}", a.is_loopback(), b, sa, bad);
    println!("{:?} {}", IpAddr::from([1u8, 2, 3, 4]), a.octets()[0]);
}
