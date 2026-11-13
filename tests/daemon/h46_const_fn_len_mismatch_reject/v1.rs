const fn n() -> usize { 2 }
const N: usize = n();
fn f() -> usize { let a: [u8; 2] = [0; N]; a.len() }
fn main() { println!("{}", f()); }
