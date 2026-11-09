// A generic body's `i.next()` (and `size_hint`, `nth`, `len`) with I = &mut UserIter: core's
// `impl Iterator for &mut I` forwards them to the user impl. lust check accepted these and the
// instance failed to build.
struct Count(u32);
impl Iterator for Count {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        self.0 += 1;
        if self.0 < 5 { Some(self.0) } else { None }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let left = 4usize.saturating_sub(self.0 as usize);
        (left, Some(left))
    }
}
impl ExactSizeIterator for Count {}
fn first<I: Iterator>(mut i: I) -> Option<I::Item> {
    i.next()
}
fn drain_all<I: Iterator>(mut i: I) -> Vec<I::Item> {
    let mut v = Vec::new();
    while let Some(x) = i.next() {
        v.push(x);
    }
    v
}
fn left<I: ExactSizeIterator>(i: I) -> usize {
    i.len()
}
fn main() {
    let mut it = Count(0);
    println!("{:?} {}", first(&mut it), left(&mut it));
    println!("{:?}", first(it.by_ref()));
    println!("{:?} {:?}", drain_all(&mut &mut it), it.next());
}
