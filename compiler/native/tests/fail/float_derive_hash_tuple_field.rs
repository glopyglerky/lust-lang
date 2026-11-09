// expect: L0110
// rustc E0277: deriving `Hash` requires every field to implement `Hash`.
#[derive(Hash)]
struct S { t: (u8, f32) }
fn main() { let _ = S { t: (1u8, 1.0f32) }; }
