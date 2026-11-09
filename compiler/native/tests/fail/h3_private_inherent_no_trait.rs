// expect: L0510
// rustc E0624: the inherent method is private and no visible trait method has the name.
mod m { pub struct S; impl S { fn h(&self) -> i32 { 1 } } }
fn main() { println!("{}", m::S.h()); }
