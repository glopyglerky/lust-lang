// expect: L0110
// rustc E0308: the init closure returns `&str`, the static holds `u8`.
use std::sync::LazyLock;
static X: LazyLock<u8> = LazyLock::new(|| "s");
fn main() {
    println!("{}", *X);
}
