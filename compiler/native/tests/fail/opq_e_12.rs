// expect: L0202
// Probe e_12 (probes, w-opaque2): rustc and lust reject it.
#![allow(unused)]
struct S { v: Vec<u8> }
fn f() -> impl Iterator<Item = &u8> { [1u8].iter() }
fn main() { let s = S { v: vec![1, 2] }; let a = [1u8, 2]; let b = [3u8]; println!("{}", f().count()); println!("done"); }
