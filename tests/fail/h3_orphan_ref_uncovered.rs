// expect: L0616
// rustc E0210: a fundamental wrapper does not cover T (RFC 2451), so T is
// uncovered before the first local type.
struct Local;
impl<T> std::ops::Add<Local> for &T { type Output = i32; fn add(self, _: Local) -> i32 { 7 } }
fn main() { println!("{}", &1u8 + Local); }
