// expect: L0112
// rustc E0277: core implements Debug for tuples of at most 12 elements.
fn main() {
    let t = (0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8);
    println!("{:?}", t);
}
