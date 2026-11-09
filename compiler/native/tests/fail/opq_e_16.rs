// expect: L0202
// Probe e_16 (probes, w-opaque2): rustc and lust reject it.
#![allow(unused)]
fn f(a: &[u8], b: &[u8]) -> impl Iterator<Item = (u8, &u8)> { a.iter().map(|x| (*x, x)) }
fn main() { let a = [1u8, 2]; let b = [3u8]; println!("{}", f(&a, &b).count()); println!("done"); }
