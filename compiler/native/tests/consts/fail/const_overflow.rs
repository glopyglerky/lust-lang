// rustc-error: E0080
// ARCH 2.13: overflow in a const expression is a const error,
// not a wrap.
const X: u8 = 255 + 1;
fn main() {
    println!("{X}");
}
