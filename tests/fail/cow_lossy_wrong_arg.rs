// expect: L0110
// rustc E0308: from_utf8_lossy takes &[u8], not a String.
fn main() {
    let s = String::from("x");
    println!("{}", String::from_utf8_lossy(s));
}
