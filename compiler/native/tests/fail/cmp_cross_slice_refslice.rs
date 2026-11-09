// expect: L0612
// rustc E0277: `[u8]: PartialEq<&[u8]>` has no impl.
fn main() {
    let v = vec![1u8];
    let s: &[u8] = &v;
    println!("{}", v[..].eq(&s));
}
