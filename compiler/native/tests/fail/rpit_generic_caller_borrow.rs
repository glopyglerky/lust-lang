// expect: L0303
// rustc E0502: the `impl Iterator` value borrows `m` immutably, so
// `m.e` cannot clear while the iterator is still used after it.
struct M<V> { e: Vec<V> }
impl<V> M<V> {
    fn iter(&self) -> impl Iterator<Item = &V> {
        self.e.iter()
    }
}
fn bad<V>(m: &mut M<V>) -> usize {
    let it = m.iter();
    m.e.clear();
    it.count()
}
fn main() {
    println!("{}", bad(&mut M { e: vec![1, 2] }));
}
