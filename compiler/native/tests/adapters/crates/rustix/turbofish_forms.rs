// Upstream's generic parameter orders: `accessat<P, Fd>`, `read<Fd, Buf>`.
use rustix::fs::{Access, AtFlags, CWD, accessat};

fn main() {
    let r = accessat::<&str, _>(CWD, "/", Access::EXISTS, AtFlags::empty());
    println!("{}", r.is_ok());
    let (rd, wr) = rustix::pipe::pipe().unwrap();
    rustix::io::write(&wr, b"xy").unwrap();
    let mut b = [0u8; 2];
    let n = rustix::io::read::<_, &mut [u8; 2]>(&rd, &mut b).unwrap();
    println!("{n} {:?}", b);
}
