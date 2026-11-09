// The definer checks a user iterator's items against `Item = &'static str`
// (through `ByNext` and `copied`), and the caller takes them as `'static`
// (ts-rust misc.rs `file_names`, go_frontend.rs `include_diagnostics_of`).
struct Map<K, V> { keys: Vec<K>, vals: Vec<V> }
struct Keys<'a, K, V> { m: &'a Map<K, V>, i: usize }
impl<K, V> Map<K, V> { fn keys(&self) -> Keys<'_, K, V> { Keys { m: self, i: 0 } } }
impl<'a, K, V> Iterator for Keys<'a, K, V> { type Item = &'a K; fn next(&mut self) -> Option<&'a K> { let r = self.m.keys.get(self.i); self.i += 1; r } }
struct D { fd: Map<&'static str, Vec<usize>> }
impl D {
    fn file_names(&self) -> impl Iterator<Item = &'static str> + '_ { self.fd.keys().copied() }
    fn take(&mut self, n: &'static str) -> usize { self.fd.keys.retain(|k| *k != n); n.len() }
}
fn main() {
    let mut d = D { fd: Map { keys: vec!["a", "bb"], vals: vec![vec![1], vec![2]] } };
    let names: Vec<&'static str> = d.file_names().collect();
    let t: usize = names.into_iter().map(|n| d.take(n)).sum();
    println!("{}", t);
}
