// expect: L0110
// rustc E0277/E0308: the later use makes the target `Vec<u8>`, and the items are `usize`.
fn f(v: &[String]) -> Vec<u8> {
    let r = v.iter().map(|s| s.len()).collect();
    r
}
fn main() {
    println!("{:?}", f(&[]));
}
