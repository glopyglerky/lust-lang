// expect: L0110
// rustc: generic parameters may not be used in const operations.
fn g<const K: usize>() -> usize { K }
fn f<const N: usize>() -> usize { g::<(N)>() }
fn main() { println!("{}", f::<3>()); }
