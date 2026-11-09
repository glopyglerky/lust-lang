// Socket timeouts: set/get round trips, the zero-duration
// errors, a read timeout expiry, and connect_timeout.
use std::io::Read;
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::time::Duration;

fn main() {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = l.local_addr().unwrap();
    let mut s = TcpStream::connect_timeout(&addr, Duration::from_secs(5)).unwrap();
    let (peer, _) = l.accept().unwrap();
    println!("{:?} {:?}", s.read_timeout().unwrap(), s.write_timeout().unwrap());
    println!("{:?}", s.set_read_timeout(Some(Duration::ZERO)).unwrap_err().kind());
    println!("{}", s.set_write_timeout(Some(Duration::ZERO)).unwrap_err());
    s.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
    s.set_write_timeout(Some(Duration::from_millis(1500))).unwrap();
    println!("{:?} {:?}", s.read_timeout().unwrap(), s.write_timeout().unwrap());
    s.set_read_timeout(Some(Duration::from_millis(100))).unwrap();
    println!("{}", s.read_timeout().unwrap().is_some());
    let mut buf = [0u8; 4];
    println!("{:?}", s.read(&mut buf).unwrap_err().kind());
    s.set_read_timeout(None).unwrap();
    println!("{:?}", s.read_timeout().unwrap());
    drop(peer);
    println!("{:?}", TcpStream::connect_timeout(&addr, Duration::ZERO).map_err(|e| e.kind()));
    let refused = {
        let l2 = TcpListener::bind("127.0.0.1:0").unwrap();
        let a = l2.local_addr().unwrap();
        drop(l2);
        TcpStream::connect_timeout(&a, Duration::from_secs(5)).map_err(|e| e.kind())
    };
    println!("{:?}", refused);
    let u = UdpSocket::bind("127.0.0.1:0").unwrap();
    u.set_read_timeout(Some(Duration::from_millis(100))).unwrap();
    let mut ub = [0u8; 4];
    println!("{:?} {}", u.recv(&mut ub).unwrap_err().kind(), u.write_timeout().unwrap().is_none());
}
