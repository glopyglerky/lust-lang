// expect: L0311
// rustc E0505: the item of next_back borrows `va` (`Self::Item` is `&'a u8`), so `va` cannot move.
struct S<'a, 'b> { x: &'a [u8], y: &'b [u8] }
impl<'a, 'b> Iterator for S<'a, 'b> {
    type Item = &'a u8;
    fn next(&mut self) -> Option<Self::Item> {
        let (f, r) = self.x.split_first()?;
        self.x = r;
        Some(f)
    }
}
impl<'q, 'p> DoubleEndedIterator for S<'p, 'q> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let (l, r) = self.x.split_last()?;
        self.x = r;
        Some(l)
    }
}
fn main() {
    let va = vec![1u8, 2, 3];
    let vb = vec![4u8, 5, 6];
    let mut s = S { x: &va, y: &vb };
    let r = s.next_back().unwrap();
    drop(va);
    println!("{}", *r);
}
