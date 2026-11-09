// expect: L0104
// rustc E0277: `u8: From<u32>` is not one of core's lossless impls.
fn main() {
    let a: u8 = 5u32.into();
    println!("{}", a);
}
