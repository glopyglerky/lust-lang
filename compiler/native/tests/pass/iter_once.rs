// std::iter::once by path, imported, and from core: core 1.99's Once<T>
// (Debug text with its option::IntoIter nesting, len, size_hint,
// next_back, clone) through the adapters.
use std::iter;
use std::iter::once;
fn main() {
    println!("{:?}", iter::once(1));
    println!("{:#?}", std::iter::once("a"));
    let v: Vec<u8> = once(1).chain(vec![2, 3]).map(|x| x * 2).collect();
    println!("{:?} {} {:?}", v, once(5).len(), once(3).size_hint());
    let w: Vec<String> = vec![String::from("a")].into_iter().chain(once(String::from("b"))).rev().collect();
    println!("{:?}", w);
    let mut o = once(9);
    println!("{:?}", o.next_back());
    println!("{:?} {:?} {:?}", o.next(), o.len(), o);
    let cl = once(vec![1]).clone().next();
    println!("{:?}", cl);
    let s = String::from("x");
    let r: Vec<&String> = once(&s).chain(once(&s)).collect();
    println!("{:?} {}", r, core::iter::once(2u8).sum::<u8>());
    for x in once(4).chain(once(5)).enumerate() { println!("{:?}", x); }
    let f: Vec<(u8, char)> = once(1).zip("ab".chars()).collect();
    println!("{:?} {:?}", f, once(7).filter(|x| *x > 3).last());
}
