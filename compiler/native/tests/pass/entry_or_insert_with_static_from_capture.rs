// ts-rust misc.rs note_file_names: `or_insert_with(|| name_of(file))` stores
// what the closure returns (`'static`), not its borrow of `file`.
use std::collections::HashMap;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Node(u32);
fn name_of(file: Node) -> &'static str { if file.0 == 0 { "a" } else { "b" } }
struct D { file: Node, rel: Vec<D> }
impl D { fn file(&self) -> Node { self.file } fn related(&self) -> &[D] { &self.rel } }
fn note(stored: &mut HashMap<Node, &'static str>, d: &D) {
    let file = d.file();
    if file.0 < 10 {
        stored.entry(file).or_insert_with(|| name_of(file));
    }
    for r in d.related() { note(stored, r); }
}
fn main() { let mut m = HashMap::new(); note(&mut m, &D { file: Node(0), rel: vec![D { file: Node(1), rel: vec![] }] }); let mut v: Vec<_> = m.into_iter().collect(); v.sort_by_key(|p| p.0.0); println!("{:?}", v); }
