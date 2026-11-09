// expect: L0112
// rustc E0277: derive(Debug) needs every field Debug; a Display impl
// is not enough.
use std::fmt;
struct Inner;
impl fmt::Display for Inner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("i")
    }
}
#[derive(Debug)]
struct Outer {
    inner: Inner,
}
fn main() {
    println!("{:?}", Outer { inner: Inner });
}
