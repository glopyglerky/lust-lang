// expect: L0110
// rustc E0277: deriving Ord needs Eq on the type.
#[derive(PartialEq, PartialOrd, Ord)]
struct S(u8);
fn main() {}
