// expect: L0312
// rustc E0506: a generic `impl Trait` value may have drop glue that
// uses the borrow it captures, so the borrow is live until the loop's
// iterator drops and the assignment inside the loop is rejected (the
// revealed hidden type must be zonked before lowering marks the
// iterator local drop-live, `Body::opaque_locals`).
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
    fn m(&self) -> &Map<String, u32> {
        &self.m
    }
    fn init(&mut self) {
        let m = self.m();
        for k in m.keys() {
            if k.starts_with(".") {
                self.kind = 1;
                return;
            }
        }
        self.kind = 2;
    }
}
fn main() {
    let mut s = S { m: Map { entries: vec![(".a".to_string(), 1)] }, kind: 0 };
    s.init();
    println!("{}", s.kind);
}
