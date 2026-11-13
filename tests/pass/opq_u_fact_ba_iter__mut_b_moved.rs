// Probe u_fact_ba_iter__mut_b_moved (probes3, w-opaque2): rustc and lust accept it with the same output.
#![allow(unused)]
struct C { v: Vec<u32>, n: u8 }
struct Loud(u32);
impl Drop for Loud { fn drop(&mut self) { println!("drop {}", self.0); } }
fn mk<'a, 'b: 'a>(c: &'a C, x: &'b [u8]) -> impl Iterator<Item = &'a u8> + use<'a> { x.iter() }
fn go(c: &mut C) -> u8 {
    let mut b = 5u8;
    let r = &*c;
    let it = mk(r, std::slice::from_ref(&b));
    drop(it);
    b = 7;
    c.n + b
}
fn main() {
    let mut c = C { v: vec![1, 2, 3], n: 0 };
    println!("{}", go(&mut c));
}
