// expect: L0202
// Probe e_m3 (probes, w-opaque2): rustc and lust reject it.
#![allow(unused)]
struct S { v: Vec<u8> }
impl S { fn f(a: &[u8], b: &[u8]) -> impl Iterator<Item = &u8> { a.iter() } }
fn main() { println!("{}", S::f(&[1], &[2]).count()); }
