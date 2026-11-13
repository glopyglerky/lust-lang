// feature: std.net
// form: a loopback TcpListener with one connect and a Shutdown
use std::net::{Shutdown, TcpListener, TcpStream};
fn main() {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let c = TcpStream::connect(l.local_addr().unwrap()).unwrap();
    let ok = c.shutdown(Shutdown::Both).is_ok();
    println!("{}", ok);
}
