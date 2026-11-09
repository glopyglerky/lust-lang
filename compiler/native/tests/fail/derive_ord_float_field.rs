// expect: L0110
// rustc E0277: f64 is not Ord (or Eq).
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct S(f64);
fn main() {}
