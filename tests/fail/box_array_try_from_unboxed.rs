// expect: L0612
// rustc E0277: a `Box<[T]>` converts to `Box<[T; N]>`, not to a plain array.
fn main() {
    let b: Box<[u8]> = vec![1u8].into();
    let a: [u8; 1] = b.try_into().unwrap();
    println!("{:?}", a);
}
