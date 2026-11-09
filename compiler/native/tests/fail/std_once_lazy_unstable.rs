// expect: L0104
// rustc E0658: `LazyLock::into_inner` is unstable (lazy_cell_into_inner).
use std::sync::LazyLock;
fn main() {
    let l = LazyLock::new(|| 5);
    println!("{:?}", LazyLock::into_inner(l));
}
