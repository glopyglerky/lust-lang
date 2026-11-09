// expect: L0110
// rustc E0277: Vec<u8> does not implement Extend<char>.
fn main() {
    let mut v: Vec<u8> = Vec::new();
    v.extend("ab".chars());
    println!("{:?}", v);
}
