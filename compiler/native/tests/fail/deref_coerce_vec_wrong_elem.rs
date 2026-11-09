// expect: L0110
// rustc E0308: the deref chain ends at `Vec<u16>`, which does not give `&[u8]`.
use std::sync::LazyLock;
static T: LazyLock<Vec<u16>> = LazyLock::new(|| vec![1]);
fn f(x: &[u8]) -> usize {
    x.len()
}
fn main() {
    println!("{}", f(&T));
}
