// expect: L0112
// rustc E0277: Debug is not in the bounds (W-CHECK-G3).
#[derive(Debug)]
struct Sq(i32);
fn make() -> impl std::fmt::Display { 5 }
fn main() { println!("{:?}", make()); }
