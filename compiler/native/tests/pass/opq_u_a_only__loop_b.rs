// Probe u_a_only__loop_b (probes3, w-opaque2): rustc and lust accept it with the same output.
#![allow(unused)]
struct C { v: Vec<u32>, n: u8 }
struct Loud(u32);
impl Drop for Loud { fn drop(&mut self) { println!("drop {}", self.0); } }
fn mk<'a>(c: &'a C, x: &u8) -> impl Iterator<Item = &'a u32> + use<'a> { c.v.iter() }
fn go(c: &mut C) -> u8 {
    let mut b = 5u8;
    let r = &*c;
    for _ in 0..2 { let it = mk(r, &b); b += 1; }
    c.n + b
}
fn main() {
    let mut c = C { v: vec![1, 2, 3], n: 0 };
    println!("{}", go(&mut c));
}
