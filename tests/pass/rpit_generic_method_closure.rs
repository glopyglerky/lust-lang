// A generic method whose hidden type holds a closure, called once from
// a method body (the accepted twin of the W-BENCH2 opaque_drop2 repro).
// A method probe that lost must not count as a second call.
struct Map<K, V> {
    entries: Vec<(K, V)>,
}
impl<K, V> Map<K, V> {
    fn keys(&self) -> impl Iterator<Item = &K> {
        self.entries.iter().map(|(k, _)| k)
    }
}
struct S {
    m: Map<String, u32>,
    kind: u8,
}
impl S {
    fn init(&mut self) {
        let found = self.m.keys().any(|k| k.starts_with("."));
        self.kind = if found { 1 } else { 2 };
    }
}
fn main() {
    let mut s = S { m: Map { entries: vec![(".a".to_string(), 1)] }, kind: 0 };
    s.init();
    println!("{} {}", s.kind, s.m.keys().count());
}
