// expect: L0112
// rustc E0277: the tuple Debug limit is 12 elements, not 13.
fn main() {
    let t = (0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8);
    println!("{:?}", t);
}
