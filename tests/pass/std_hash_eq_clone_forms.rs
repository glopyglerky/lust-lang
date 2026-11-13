// Std's Hash, PartialEq, and Clone forms for std types (fx-splice-derive):
// slices hash their elements, IP addresses hash as one integer,
// `Ordering` as its `i8` discriminant, `FileType` by its type bits;
// a derived `Copy` on a generic std type does not copy a `String`.
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn h<T: Hash + ?Sized>(x: &T) -> u64 {
    let mut s = DefaultHasher::new();
    x.hash(&mut s);
    s.finish()
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct I(u8);
impl Iterator for I {
    type Item = u8;
    fn next(&mut self) -> Option<u8> {
        None
    }
}

fn main() {
    let a = [1i32, 2];
    let s = &a[..];
    let mut st = DefaultHasher::new();
    s.hash(&mut st);
    println!("{} {}", st.finish(), h(s));
    let b = vec![String::from("x")].into_boxed_slice();
    let mut st = DefaultHasher::new();
    b.hash(&mut st);
    println!("{}", st.finish());
    println!("{} {}", h(&std::net::Ipv4Addr::new(1, 2, 3, 4)), h(&std::net::Ipv6Addr::LOCALHOST));
    println!("{}", h(&std::net::SocketAddrV4::new(std::net::Ipv4Addr::new(10, 0, 0, 1), 80)));
    println!("{} {}", h(&std::cmp::Ordering::Less), h(&Some(std::cmp::Ordering::Greater)));
    let t1 = std::fs::metadata("/").unwrap().file_type();
    let t2 = std::fs::metadata("/tmp").unwrap().file_type();
    println!("{} {}", t1 == t2, h(&t1) == h(&t2));
    println!("{}", h(&t1));
    let x = std::ops::Bound::Included(String::from("hi"));
    let y = x.clone();
    println!("{:?} {:?}", x, y);
    let c = std::ops::ControlFlow::<String>::Break(String::from("b"));
    println!("{:?}", c.clone());
    // `&I` is no iterator: `eq` and `partial_cmp` are the comparison traits'.
    let i = I(1);
    let r = &i;
    println!("{} {:?}", r.eq(&r), r.partial_cmp(&&i));
}
