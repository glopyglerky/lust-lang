// `use std::str;` names the module for `str::from_utf8`, and `str` in a
// type position stays the primitive.
use std::str;
fn len_of(s: &str) -> usize {
    s.len()
}
fn main() {
    let b = [104u8, 105];
    if let Ok(s) = str::from_utf8(&b) {
        let t: &str = "abc";
        println!("{} {} {}", len_of(s), t, s);
    }
}
