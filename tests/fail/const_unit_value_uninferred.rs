// expect: L0111
// rustc E0284: nothing fixes N.
struct S<const N: usize>;
fn main() { let _s = S; println!("x"); }
