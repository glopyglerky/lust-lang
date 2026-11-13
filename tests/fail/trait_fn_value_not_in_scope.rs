// expect: L0104
// rustc E0599: the trait is not in scope, so S::mk names nothing.
struct S;
mod m { pub trait Tr { fn mk() -> u8; } impl Tr for super::S { fn mk() -> u8 { 7 } } }
fn main() { let f = S::mk; println!("{}", f()); }
