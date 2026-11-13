// std::net UDP (w-cinterop): bind forms, send and receive, peek,
// connect, options, and address resolution errors.
use std::net::{ToSocketAddrs, UdpSocket};

fn txt(b: &[u8]) -> String {
    b.iter().map(|c| *c as char).collect()
}

fn show(s: &str) {
    match s.to_socket_addrs() {
        Ok(mut it) => println!("{s:?} -> {:?}", it.next()),
        Err(e) => println!("{s:?} -> err {:?} {e}", e.kind()),
    }
}

fn main() {
    let a = UdpSocket::bind("127.0.0.1:0").unwrap();
    let b = UdpSocket::bind(("127.0.0.1", 0)).unwrap();
    b.send_to(b"xy", a.local_addr().unwrap()).unwrap();
    let mut buf = [0u8; 8];
    let (n, from) = a.recv_from(&mut buf).unwrap();
    println!("{n} {} {}", txt(&buf[..n]), from == b.local_addr().unwrap());
    b.connect(a.local_addr().unwrap()).unwrap();
    b.send(b"z").unwrap();
    println!("conn {} {}", a.recv(&mut buf).unwrap(), b.peer_addr().unwrap() == a.local_addr().unwrap());
    b.send(b"peek").unwrap();
    let (n, _) = a.peek_from(&mut buf).unwrap();
    println!("peek {n} again {}", a.recv(&mut buf).unwrap());
    a.set_broadcast(true).unwrap();
    println!("broadcast {} ttl {}", a.broadcast().unwrap(), a.ttl().unwrap());
    match a.peer_addr() {
        Err(e) => println!("nopeer {:?} {:?}", e.kind(), e.raw_os_error()),
        Ok(_) => println!("nopeer ok"),
    }
    for s in ["127.0.0.1:80", "127.0.0.1", "[::1]:80", "host:notaport", "", "127.0.0.1:+80", "127.0.0.1:65536"] {
        show(s);
    }
}
