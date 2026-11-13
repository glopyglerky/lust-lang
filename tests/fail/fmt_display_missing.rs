// expect: L0112
// rustc E0277: a Debug impl does not give Display.
use std::fmt;
struct A;
impl fmt::Debug for A {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("A")
    }
}
fn main() {
    println!("{}", A);
}
