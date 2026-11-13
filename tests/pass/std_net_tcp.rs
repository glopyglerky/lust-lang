// std::net TCP (w-cinterop): listener, stream, Read and Write,
// address forms, options, and the io::Error values of failures, all
// on loopback with kernel-assigned ports.
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};

fn txt(b: &[u8]) -> String {
    b.iter().map(|c| *c as char).collect()
}

fn main() {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut c = TcpStream::connect(l.local_addr().unwrap()).unwrap();
    let (mut s, peer) = l.accept().unwrap();
    println!("{} {}", c.peer_addr().unwrap() == l.local_addr().unwrap(), c.local_addr().unwrap() == peer);
    c.write_all(b"hello").unwrap();
    let mut buf = [0u8; 8];
    let n = s.read(&mut buf).unwrap();
    println!("read {n} {}", txt(&buf[..n]));
    let mut peeked = [0u8; 2];
    s.write_all(b"back").unwrap();
    println!("peek {}", c.peek(&mut peeked).unwrap());
    println!("nodelay {} ttl {}", c.nodelay().unwrap(), c.ttl().unwrap());
    c.set_nodelay(true).unwrap();
    c.set_ttl(32).unwrap();
    println!("opts {} {}", c.nodelay().unwrap(), c.ttl().unwrap());
    let c2 = c.try_clone().unwrap();
    println!("clone-peer {}", c2.peer_addr().unwrap() == c.peer_addr().unwrap());
    c.shutdown(Shutdown::Write).unwrap();
    let mut tail = [0u8; 8];
    let mut total = 0usize;
    loop {
        let n = s.read(&mut tail).unwrap();
        if n == 0 {
            break;
        }
        total += n;
    }
    println!("tail {total}");

    match TcpStream::connect("127.0.0.1:1") {
        Err(e) => println!("refused {:?} {:?} {e}", e.kind(), e.raw_os_error()),
        Ok(_) => println!("refused ok"),
    }
    match TcpListener::bind(l.local_addr().unwrap()) {
        Err(e) => println!("inuse {:?} {:?} {e}", e.kind(), e.raw_os_error()),
        Ok(_) => println!("inuse ok"),
    }
    match TcpStream::connect("no-such-host.invalid:80") {
        Err(e) => println!("badhost {:?} {e}", e.kind()),
        Ok(_) => println!("badhost ok"),
    }
    let l6 = TcpListener::bind("[::1]:0").unwrap();
    println!("v6 {}", TcpStream::connect(l6.local_addr().unwrap()).unwrap().peer_addr().unwrap().is_ipv6());
}
