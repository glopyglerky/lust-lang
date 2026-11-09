// expect: L0110
// rustc E0277: deriving `Eq` requires every field to implement `Eq`.
#[derive(PartialEq, Eq)]
struct S { x: f32 }
fn main() { let _ = S { x: 1.0 }; }
