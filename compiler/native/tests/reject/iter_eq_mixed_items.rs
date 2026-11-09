// expect: L0640
// Valid Rust: String: PartialEq<&str>; lust's Iterator::eq compares items of one type.
fn main() {
    let r = vec![String::from("a")].into_iter().eq(vec!["a"]);
    println!("{}", r);
}
