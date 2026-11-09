// expect: L0612
// rustc E0277: `copy_within` needs `T: Copy`; String is not.
fn main() {
    let mut v = vec!["a".to_string(), "b".to_string()];
    v.copy_within(0..1, 1);
    println!("{:?}", v);
}
