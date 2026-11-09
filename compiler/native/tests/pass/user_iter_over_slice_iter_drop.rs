// A user iterator holding a `slice::Iter<'a, ..>` whose items need drop:
// dropping it reads no borrow (slice::Iter has no destructor), so the
// container is free once the iterator is last used.
struct M<K, V> { e: Vec<(K, V)> }
struct It<'a, K, V> { i: std::slice::Iter<'a, (K, V)> }
impl<'a, K, V> Iterator for It<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<(&'a K, &'a V)> { self.i.next().map(|(k, v)| (k, v)) }
}
fn run(mut m: M<u32, Vec<&'static str>>) -> usize {
    let mut n = 0;
    let mut it = It { i: m.e.iter() };
    for (_k, v) in &mut it { n += v.len(); }
    m.e.clear();
    n + m.e.len()
}
fn main() { println!("{}", run(M { e: vec![(1, vec!["a", "b"])] })); }
