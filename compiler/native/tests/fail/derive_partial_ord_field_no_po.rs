// expect: L0110
// rustc E0277: a field type has no PartialOrd.
#[derive(PartialEq)]
struct F(u8);
#[derive(PartialEq, PartialOrd)]
struct S(F);
fn main() {}
