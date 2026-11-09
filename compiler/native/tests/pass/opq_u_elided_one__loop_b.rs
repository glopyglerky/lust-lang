// Probe u_elided_one__loop_b (probes3, w-opaque2): rustc and lust accept it with the same output.
#![allow(unused)]
struct C { v: Vec<u32>, n: u8 }
struct Loud(u32);
impl Drop for Loud { fn drop(&mut self) { println!("drop {}", self.0); } }
fn mk(c: &C) -> impl Sized + use<'_> { &c.n }
fn go(c: &mut C) -> u8 {
    let mut b = 5u8;
    let r = &*c;
    for _ in 0..2 { let it = { let _ = &b; mk(r) }; b += 1; }
    c.n + b
}
fn main() {
    let mut c = C { v: vec![1, 2, 3], n: 0 };
    println!("{}", go(&mut c));
}
