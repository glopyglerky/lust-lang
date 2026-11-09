// Probe h_debug_tuple__array_index_copy_ok (probes5, w-opaque2): rustc and lust accept it with the same output.
#![allow(unused)]
struct C { v: Vec<u32>, n: u8 }
struct Wrap<I> { it: I }
fn consume<T>(_t: T) {}
trait Take { fn take(self) -> u8; }
impl Take for &u8 { fn take(self) -> u8 { *self } }
fn mk(c: &C) -> impl std::fmt::Debug + '_ { (&c.n, 1u8) }
fn go(c: &mut C) -> u8 {
    let r = &*c;
    let a = [1u8, 2];
    let it = mk(r);
    let x = a[0];
    let y = a[0];
    c.n
}
fn main() {
    let mut c = C { v: vec![1, 2, 3], n: 0 };
    println!("{}", go(&mut c));
}
