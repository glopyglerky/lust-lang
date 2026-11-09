// A generic fn uses an `impl Iterator` method of `M<V>` where `V`
// is the caller's own parameter: the template checks in the opaque
// view and the borrow checker tracks the opaque value through its
// capture var (W-BENCH4 rank 8).
struct M<V> { e: Vec<V> }
impl<V> M<V> {
    fn iter(&self) -> impl Iterator<Item = &V> {
        self.e.iter()
    }
}
fn count<V>(m: &M<V>) -> usize {
    m.iter().enumerate().count()
}
fn cloned<V: Clone>(m: &M<V>) -> Vec<V> {
    m.iter().cloned().collect()
}
fn main() {
    println!("{}", count(&M { e: vec![1, 2] }));
    println!("{:?}", cloned(&M { e: vec![3, 4] }));
}
