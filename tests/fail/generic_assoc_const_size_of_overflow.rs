// expect: L0630
// rustc E0080: `Page::<u8>::SHORT` is `1 - 3`, which overflows at this use.
use std::mem::size_of;
struct Page<V>(Vec<V>);
impl<V> Page<V> {
    const SHORT: usize = size_of::<V>() - 3;
}
fn main() {
    println!("{}", Page::<u8>::SHORT);
}
