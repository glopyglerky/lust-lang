// expect: L0312
// Probe u_adt_two__mut_c_drop (probes3, w-opaque2): rustc and lust reject it.
#![allow(unused)]
struct C { v: Vec<u32>, n: u8 }
struct Loud(u32);
impl Drop for Loud { fn drop(&mut self) { println!("drop {}", self.0); } }
struct P<'x, 'y>(&'x u8, &'y u8);
fn mk<'a, 'b>(c: &'a C, p: P<'a, 'b>) -> impl Sized + use<'a> { p.0 }
fn go(c: &mut C) -> u8 {
    let mut b = 5u8;
    let r = &*c;
    let it = mk(r, P(&r.n, &b));
    c.n = 1;
    c.n + b
}
fn main() {
    let mut c = C { v: vec![1, 2, 3], n: 0 };
    println!("{}", go(&mut c));
}
