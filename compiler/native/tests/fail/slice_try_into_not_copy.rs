// expect: L0612
// rustc E0277: `TryFrom<&[T]> for [T; N]` needs `T: Copy`.
fn main() {
    let v = vec!["a".to_string()];
    let a: [String; 1] = v.as_slice().try_into().unwrap();
    println!("{:?}", a);
}
