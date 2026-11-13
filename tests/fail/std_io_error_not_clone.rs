// expect: L0104
// io::Error is not Clone.
use std::io;
fn main() {
    let e = io::Error::other("x");
    let f = e.clone();
    println!("{}", f);
}
