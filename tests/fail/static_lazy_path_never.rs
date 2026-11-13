// expect: L0110
// rustc E0308: a `fn() -> !` item does not coerce to the init fn
// pointer `fn() -> i32`.
use std::sync::LazyLock;
fn never() -> ! {
    panic!()
}
static X: LazyLock<i32> = LazyLock::new(never);
fn main() {
    println!("{}", *X);
}
