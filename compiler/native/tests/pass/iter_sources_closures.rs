// std::iter::once_with, from_fn and successors next to once: Debug text,
// size_hint, len, and the adapters over them (mono gives their closure-
// bound Iterator impls ByNext instances).
use std::iter;
use std::iter::{once, successors};
fn main() {
    println!("{:?}", iter::once(1));
    println!("{:#?}", std::iter::once("a"));
    let v: Vec<u8> = once(1).chain(vec![2, 3]).map(|x| x * 2).collect();
    println!("{:?} {} {:?}", v, once(5).len(), once(3).size_hint());
    let w: Vec<String> = vec![String::from("a")].into_iter().chain(once(String::from("b"))).rev().collect();
    println!("{:?}", w);
    let mut ow = iter::once_with(|| 7);
    println!("{:?} {:?}", ow, ow.size_hint());
    println!("{:?}", ow.next());
    println!("{:?} {:?}", ow, ow.len());
    let mut c = 0;
    let ff: Vec<i32> = iter::from_fn(|| { c += 1; if c < 4 { Some(c) } else { None } }).collect();
    println!("{:?} {:?}", ff, iter::from_fn(|| Some(1)));
    let p: Vec<u32> = successors(Some(1u32), |&x| x.checked_mul(10)).collect();
    println!("{:?} {:?}", p.len(), successors(Some(2), |&x: &i32| if x < 5 { Some(x + 1) } else { None }));
    let mut o = once(9);
    println!("{:?}", o.next_back());
    println!("{:?} {:?} {:?}", o.next(), o.len(), o);
    let cl = once(vec![1]).clone().next();
    println!("{:?}", cl);
}
