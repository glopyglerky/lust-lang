// expect: L0102
mod a { pub mod b { pub struct P; } }
struct S { x: crate::a::b::Gone, y: crate::a::b::P }
fn main() {}
