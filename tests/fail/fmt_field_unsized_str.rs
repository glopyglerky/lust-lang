// expect: L0112
// rustc E0277: `&str` cannot coerce to `&dyn Debug` (str is unsized).
use std::fmt;
struct N(&'static str);
impl fmt::Debug for N {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("N").field(self.0).finish()
    }
}
fn main() {
    println!("{:?}", N("x"));
}
