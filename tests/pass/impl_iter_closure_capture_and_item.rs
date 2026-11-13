// A returned `impl Iterator` whose hidden `Map` holds a closure capture
// and an item of the same written lifetime (ts-rust effect/graph.rs
// `topo`, ast `nodes`), and a `'static` item read from a map's keys
// (ast misc.rs `file_names`).
use std::collections::BTreeMap;
struct G {
    v: Vec<u8>,
}
fn data(g: &G, i: usize) -> &u8 {
    &g.v[i]
}
fn topo<'a>(g: &'a G) -> impl Iterator<Item = &'a u8> + 'a {
    (0..2).map(move |n| data(g, n))
}
struct B {
    slots: Vec<u32>,
    entries: Vec<String>,
    names: BTreeMap<&'static str, u8>,
}
impl B {
    fn nodes(&self) -> impl Iterator<Item = &String> {
        self.slots.iter().map(move |&slot| &self.entries[slot as usize])
    }
    fn file_names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.names.keys().copied()
    }
}
fn main() {
    let g = G { v: vec![10, 20] };
    for x in topo(&g) {
        println!("{x}");
    }
    let mut names = BTreeMap::new();
    names.insert("f.ts", 1u8);
    let b = B { slots: vec![1, 0], entries: vec!["a".into(), "b".into()], names };
    for x in b.nodes() {
        println!("{x}");
    }
    let n: Vec<&str> = b.file_names().collect();
    println!("{:?}", n);
}
