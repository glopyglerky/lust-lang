// expect: L0202
// Probe e_10 (probes, w-opaque2): rustc and lust reject it.
#![allow(unused)]
struct S { v: Vec<u8> }
fn f(a: &[u8], b: &[u8]) -> Option<impl Iterator<Item = &u8>> { Some(a.iter()) }
fn main() { let s = S { v: vec![1, 2] }; let a = [1u8, 2]; let b = [3u8]; println!("{}", f(&a, &b).unwrap().count()); println!("done"); }
