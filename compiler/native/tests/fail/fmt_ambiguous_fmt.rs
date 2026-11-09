// expect: L0104
// rustc E0034: with Display and Debug both in scope, `fmt` on u32 is
// ambiguous.
use std::fmt;
use std::fmt::{Debug, Display};
struct T(u32);
impl Display for T {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
impl Debug for T {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("t")
    }
}
fn main() {
    println!("{} {:?}", T(1), T(2));
}
