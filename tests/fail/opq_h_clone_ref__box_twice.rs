// expect: L0301
// Probe h_clone_ref__box_twice (probes5, w-opaque2): rustc and lust reject it.
#![allow(unused)]
struct C { v: Vec<u32>, n: u8 }
struct Wrap<I> { it: I }
fn consume<T>(_t: T) {}
trait Take { fn take(self) -> u8; }
impl Take for &u8 { fn take(self) -> u8 { *self } }
fn mk(c: &C) -> impl Clone + '_ { &c.n }
fn go(c: &mut C) -> u8 {
    let r = &*c;
    let it = mk(r);
    let a = Box::new(it);
    let b = Box::new(it);
    c.n
}
fn main() {
    let mut c = C { v: vec![1, 2, 3], n: 0 };
    println!("{}", go(&mut c));
}
