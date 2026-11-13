// expect: L0630
// rustc E0308: a fn that takes `&'static u8` is not a `fn(&u8)`.
fn f(x: &'static u8) -> u8 {
    *x
}
const F: fn(&u8) -> u8 = f;
fn main() {}
