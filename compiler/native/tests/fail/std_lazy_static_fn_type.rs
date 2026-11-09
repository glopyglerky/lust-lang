// expect: L0110
// rustc E0308: the closure is `fn() -> u8`, the static's `F` says `u16`.
use std::sync::LazyLock;
static Y: LazyLock<u8, fn() -> u16> = LazyLock::new(|| 1);
fn main() {
    println!("{}", *Y);
}
