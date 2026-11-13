// Probe e_5 (probes, w-opaque2): rustc and lust accept it with the same output.
#![allow(unused)]
struct S { v: Vec<u8> }
fn f(a: &[u8], b: &[u8]) -> impl Iterator<Item = u8> { a.to_vec().into_iter().chain(b.to_vec()) }
fn main() { let s = S { v: vec![1, 2] }; let a = [1u8, 2]; let b = [3u8]; println!("{}", f(&a, &b).count()); println!("done"); }
