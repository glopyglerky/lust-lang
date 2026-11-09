// expect: L0612
// rustc E0277: String does not implement Extend<u8>.
fn main() {
    let mut s = String::new();
    s.extend(vec![1u8, 2]);
    println!("{}", s);
}
