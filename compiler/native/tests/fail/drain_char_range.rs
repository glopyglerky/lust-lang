// expect: L0110
// String::drain indexes bytes, not chars.
fn main() {
    let mut s = String::from("abc");
    let _ = s.drain('a'..'c');
}
