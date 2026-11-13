// expect: L0110
// rustc: generic parameters may not be used in const operations.
fn f<const N: usize>() -> usize { let a = [0u8; (N)]; a.len() }
fn main() { println!("{}", f::<3>()); }
