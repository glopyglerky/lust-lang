// expect: L0615
// rustc E0119: core has `impl<T: Hash> Hash for &T`, and Local: Hash.
#[derive(Hash)]
struct Local;
impl std::hash::Hash for &Local { fn hash<H: std::hash::Hasher>(&self, h: &mut H) {} }
fn main() { println!("ok"); }
