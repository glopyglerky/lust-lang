// A returned impl Iterator<Item = &'static str> over a user iterator of
// `&'static str` keys (ts-rust ast/misc.rs `file_names` over IndexMap).
pub struct Keys<'a, K, V> { inner: std::slice::Iter<'a, (K, V)> }
impl<'a, K, V> Iterator for Keys<'a, K, V> { type Item = &'a K; fn next(&mut self) -> Option<&'a K> { self.inner.next().map(|p| &p.0) } }
pub struct M<K, V> { e: Vec<(K, V)> }
impl<K, V> M<K, V> { pub fn keys(&self) -> Keys<'_, K, V> { Keys { inner: self.e.iter() } } }
struct B { names: M<&'static str, Vec<usize>> }
impl B {
    fn file_names(&self) -> impl Iterator<Item = &'static str> + '_ { self.names.keys().copied() }
}
fn main() { let b = B { names: M { e: vec![("f.ts", vec![1])] } }; for n in b.file_names() { println!("{n}"); } }
