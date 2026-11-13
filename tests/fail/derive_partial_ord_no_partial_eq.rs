// expect: L0110
// rustc E0277: deriving PartialOrd needs PartialEq on the type.
#[derive(PartialOrd)]
struct S(u8);
fn main() {}
