// expect: L0102
// An associated type of a supertrait (`Iterator::Item` through a
// DoubleEndedIterator bound) is not in scope by the bare path yet.
fn xb<I: DoubleEndedIterator>(i: &mut I) -> Option<I::Item> {
    i.next_back()
}
fn main() {
    let mut v = vec![1, 2, 3];
    println!("{:?}", xb(&mut v.drain(1..)));
}
