// expect: L0612
// An integer is not a socket address.
use std::net::TcpListener;
fn main() {
    let _ = TcpListener::bind(3);
}
