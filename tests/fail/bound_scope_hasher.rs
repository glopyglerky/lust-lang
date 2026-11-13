// expect: L0104
// rustc E0599: `H: Hasher` gives `h.write_u8` for `h: &mut H`, not for a concrete `DefaultHasher`.
use std::collections::hash_map::DefaultHasher;
fn f<H: std::hash::Hasher>(h: &mut H, d: &mut DefaultHasher) {
    h.write_u8(1);
    d.write_u8(1);
}
fn main() {
    let mut d = DefaultHasher::new();
    let mut e = d.clone();
    f(&mut d, &mut e);
    println!("ok");
}
