// expect: L0110
// rustc: generic parameters may not be used in const operations.
struct S<const K: usize>;
fn f<const N: usize>() { let _: S<(N)> = S; }
fn main() { f::<3>(); println!("ok"); }
