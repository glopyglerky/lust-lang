// expect: L0733
// rustc: lifetime may not live long enough. `Self::Item` (from the Iterator impl) is `&'a u8`;
// next_back returns a borrow of the `'b` field.
struct S<'a, 'b> { x: &'a [u8], y: &'b [u8] }
impl<'a, 'b> Iterator for S<'a, 'b> {
    type Item = &'a u8;
    fn next(&mut self) -> Option<Self::Item> {
        let (f, r) = self.x.split_first()?;
        self.x = r;
        Some(f)
    }
}
impl DoubleEndedIterator for S<'_, '_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let (l, r) = self.y.split_last()?;
        self.y = r;
        Some(l)
    }
}
fn main() {
    let va = vec![1u8, 2, 3];
    let vb = vec![4u8, 5, 6];
    let mut s = S { x: &va, y: &vb };
    let r = s.next_back().unwrap();
    println!("{}", *r);
}
