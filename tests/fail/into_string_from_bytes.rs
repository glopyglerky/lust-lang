// expect: L0104
// rustc E0277: no From<Vec<u8>> for String (only String::from_utf8).
fn main() {
    let s: String = vec![97u8].into();
    println!("{s}");
}
