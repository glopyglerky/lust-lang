// feature: std.unix_net
// form: a UnixStream pair with one write and read, as compile.rs makes them
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
fn main() {
    let (mut a, mut b) = UnixStream::pair().unwrap();
    a.write_all(b"hi").unwrap();
    let mut buf = [0u8; 4];
    let n = b.read(&mut buf).unwrap();
    println!("{} {}", n, String::from_utf8_lossy(&buf[..n]));
}
