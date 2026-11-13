// expect: L0612
// rustc E0277: core implements Default for tuples of at most 12 elements.
fn main() {
    let t: (u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8, u8) = Default::default();
    println!("{}", t.0);
}
