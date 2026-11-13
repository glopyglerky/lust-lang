// expect: L0112
// rustc E0277: a pair that holds a 13-tuple is not Debug either.
fn main() {
    let t = ((0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8), 1u8);
    println!("{:?}", t);
}
