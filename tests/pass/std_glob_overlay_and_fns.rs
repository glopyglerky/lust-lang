// A std glob brings the items of lust's std source overlay modules and
// the std fns and types lust knows by path, at module and block level.
use std::cell::*;
use std::collections::*;
use std::mem::*;
use std::panic::*;

fn block_globs() {
    use std::cmp::*;
    use std::net::*;
    println!("{} {}", max(3, 9), Ipv4Addr::new(10, 0, 0, 1));
    let c = OnceCell::new();
    c.set(5).unwrap();
    println!("{:?}", c.get());
}

fn main() {
    let x = 4;
    let r = catch_unwind(AssertUnwindSafe(|| x + 1));
    println!("{:?}", r.ok());
    let cell: OnceCell<u8> = OnceCell::new();
    println!("{:?}", cell.get());
    let mut a = vec![1, 2];
    let mut b = vec![3];
    swap(&mut a, &mut b);
    let t = take(&mut a);
    println!("{:?} {:?} {:?} {}", a, b, t, size_of::<u64>());
    let mut s = HashSet::new();
    s.insert(7);
    println!("{}", s.len());
    block_globs();
}
