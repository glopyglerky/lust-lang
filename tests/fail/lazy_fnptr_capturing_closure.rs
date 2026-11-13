// expect: L0110
// rustc E0308: a capturing closure does not coerce to `fn() -> T`,
// the default `F` of `LazyCell<T>`.
use std::cell::LazyCell;

fn main() {
    let k = 3u32;
    let a: LazyCell<u32> = LazyCell::new(|| k);
    println!("{}", *a);
}
