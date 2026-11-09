// expect: L0612
// rustc E0277: the type has no Hash impl.
fn h<T: std::hash::Hash>(_t: &T) {}
#[derive(PartialEq)]
struct S(u8);
fn main() { h(&S(1)); }
