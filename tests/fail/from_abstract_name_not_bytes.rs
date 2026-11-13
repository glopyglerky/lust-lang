// expect: L0612
// rustc E0277: from_abstract_name takes `N: AsRef<[u8]>`.
use std::os::linux::net::SocketAddrExt;
fn main() { let _ = std::os::unix::net::SocketAddr::from_abstract_name(5u8); }
