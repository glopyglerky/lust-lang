const fn n() -> usize { 2 }
const N: usize = n();
fn f() -> usize { let a = [0u8; N]; a.len() }
fn main() { println!("{}", f()); }
