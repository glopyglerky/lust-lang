// expect: L0110
// write_fmt takes fmt::Arguments, not a String.
use std::io::Write;
fn main() {
    let mut v: Vec<u8> = Vec::new();
    let _ = v.write_fmt(String::from("x"));
}
