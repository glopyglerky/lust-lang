// expect: L0110
// rustc E0277: f64 is not Hash.
#[derive(Hash)]
struct S(f64);
fn main() {}
