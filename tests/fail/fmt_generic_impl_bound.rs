// expect: L0612
// rustc E0277: `W<T>: Display` needs `T: Display`.
use std::fmt;
struct W<T>(T);
struct NoShow;
impl<T: fmt::Display> fmt::Display for W<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
fn main() {
    println!("{}", W(NoShow));
}
