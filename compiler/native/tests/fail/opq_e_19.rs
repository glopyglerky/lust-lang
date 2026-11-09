// expect: L0110
// Probe e_19 (probes, w-opaque2): rustc and lust reject it.
#![allow(unused)]
trait Tr<'a> { fn get(&self) -> u8; } impl<'a> Tr<'a> for u8 { fn get(&self) -> u8 { *self } } fn f(a: &[u8], b: &[u8]) -> impl Tr<'_> { 1u8 }
fn main() { let a = [1u8, 2]; let b = [3u8]; println!("{}", f(&a, &b).get()); println!("done"); }
