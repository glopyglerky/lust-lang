// Probe e_1 (probes, w-opaque2): rustc and lust accept it with the same output.
#![allow(unused)]
fn f(a: &[u8], b: &[u8]) -> impl Fn(&u8) -> u8 { let n = a.len() + b.len(); move |x: &u8| *x + n as u8 }
fn main() { let a = [1u8, 2]; let b = [3u8]; let g = f(&a, &b); println!("{}", g(&1)); println!("done"); }
