// expect: L0630
// rustc E0599: `S::C` names no trait, and the trait is not in scope here.
mod m { pub trait Tr { const C: i32; } impl Tr for super::S { const C: i32 = 7; } }
struct S;
const K: i32 = S::C;
fn main() { println!("{}", K); }
