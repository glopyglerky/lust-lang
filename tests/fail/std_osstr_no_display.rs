// expect: L0112
// OsStr has no Display impl.
use std::ffi::OsStr;
fn main() {
    println!("{}", OsStr::new("a"));
}
