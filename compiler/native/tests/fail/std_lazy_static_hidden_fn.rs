// expect: L0101
// rustc E0425: the hidden init fn of a lazy static has no name user code
// can write.
use std::sync::LazyLock;
static X: LazyLock<u8> = LazyLock::new(|| 1);
fn main() {
    let f = __lust_lazy_X;
    println!("{}", *X);
}
