// expect: L0612
// rustc E0277: `Vec<u8>` is not Display.
use std::fmt;
struct W(Vec<u8>);
impl fmt::Display for W {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
fn main() {
    println!("{}", W(vec![1]));
}
