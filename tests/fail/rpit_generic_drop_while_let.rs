// expect: L0312
// rustc E0506: the opaque iterator in a `let` drops on the return
// path, after the assignment, so its captured borrow is still live.
// The hidden type (`slice::Iter`) has no drop glue; rustc still
// assumes an opaque type may drop.
struct C<K> {
    v: Vec<K>,
    n: u8,
}
impl<K> C<K> {
    fn items(&self) -> impl Iterator<Item = &K> {
        self.v.iter()
    }
}
fn main() {
    let mut c = C { v: vec![1u32, 2], n: 0 };
    let mut it = c.items();
    while let Some(_x) = it.next() {
        c.n = 1;
        return;
    }
}
