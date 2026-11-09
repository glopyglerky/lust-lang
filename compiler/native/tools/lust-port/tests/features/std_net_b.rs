// feature: std.net
// form: socket timeouts (set/get round trip) and connect_timeout
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
fn main() {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let s = TcpStream::connect_timeout(&l.local_addr().unwrap(), Duration::from_secs(5)).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    println!("{:?}", s.read_timeout().unwrap());
}
