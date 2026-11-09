// Probe e_m2 (probes, w-opaque2): rustc and lust accept it with the same output.
#![allow(unused)]
struct S { v: Vec<u8> }
impl S { fn f(self: &Self, b: &[u8]) -> impl Iterator<Item = &u8> + '_ { self.v.iter() } }
fn main() { let s = S { v: vec![1, 2] }; println!("{}", s.f(&[1]).count()); }
