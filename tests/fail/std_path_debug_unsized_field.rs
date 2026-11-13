// expect: L0112
// A &Path field argument of debug_struct must be a sized value behind &.
use std::fmt;
use std::path::Path;
struct S<'a> {
    p: &'a Path,
}
impl fmt::Debug for S<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("S").field("p", self.p).finish()
    }
}
fn main() {
    println!("{:?}", S { p: Path::new("x") });
}
