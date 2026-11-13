// expect: L0612
// rustc E0277: `TryFrom<Vec<T>> for [T; N]` needs the same element type.
fn main() {
    let v = vec![1u8, 2];
    let a: [u16; 2] = v.try_into().unwrap();
    println!("{:?}", a);
}
