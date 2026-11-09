// expect: L0610
// rustc E0284: nothing fixes N.
struct S<const N: usize>;
fn main() { S; println!("x"); }
