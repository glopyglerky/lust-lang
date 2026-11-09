// expect: L0110
// rustc E0277: the field type has no Default.
struct N;
#[derive(Default)]
struct S { n: N }
fn main() {}
