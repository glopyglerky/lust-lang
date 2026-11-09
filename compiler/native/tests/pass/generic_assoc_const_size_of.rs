// An associated const of a generic impl that reads `size_of::<V>()` has one
// value per instance; the generic body is only checked, so an unused
// instance-dependent value is no error (ts-rust core.rs SparseLinkPage::FIRST).
use std::mem::size_of;
struct Page<V> {
    records: Vec<V>,
}
impl<V> Page<V> {
    const FIRST: usize = {
        let size = if size_of::<V>() == 0 { 1 } else { size_of::<V>() };
        if 64 / size > 4 { 64 / size } else { 4 }
    };
    const SHORT: usize = size_of::<V>() - 3;
    fn new() -> Self {
        Page { records: Vec::with_capacity(Self::FIRST) }
    }
}
fn main() {
    let a: Page<u8> = Page::new();
    let b: Page<[u64; 4]> = Page::new();
    let c: Page<()> = Page::new();
    println!("{} {} {}", Page::<u8>::FIRST, Page::<[u64; 4]>::FIRST, Page::<()>::FIRST);
    println!("{} {} {}", a.records.capacity() >= 4, b.records.capacity() >= 4, c.records.len());
    println!("{}", Page::<u64>::SHORT);
}
