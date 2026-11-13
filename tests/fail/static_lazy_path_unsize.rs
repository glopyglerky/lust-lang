// expect: L0110
// rustc E0308: `fn() -> &'static [u8; 2]` is not `fn() -> &'static [u8]`.
use std::sync::LazyLock;
fn two() -> &'static [u8; 2] {
    &[1, 2]
}
static X: LazyLock<&[u8]> = LazyLock::new(two);
fn main() {
    println!("{:?}", *X);
}
