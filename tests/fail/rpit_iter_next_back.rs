// expect: L0612
// rustc E0599: `next_back` needs DoubleEndedIterator, which an `impl
// Iterator` value does not have.
fn evens() -> impl Iterator<Item = u32> {
    vec![2u32, 4].into_iter()
}
fn main() {
    let mut it = evens();
    println!("{:?}", it.next_back());
}
