// expect: L0104
// TcpListener has try_clone, not Clone.
use std::net::TcpListener;
fn main() {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let m = l.clone();
    println!("{}", m.local_addr().is_ok());
}
