// expect: L0104
// rustc E0599: `BuildHasher` is not in scope (no `use`), so its method does not apply.
#[derive(Default)] struct F(u64);
impl std::hash::Hasher for F { fn finish(&self) -> u64 { self.0 } fn write(&mut self, b: &[u8]) { for &x in b { self.0 += x as u64; } } }
struct FB;
impl std::hash::BuildHasher for FB { type Hasher = F; fn build_hasher(&self) -> F { F(1) } }
fn main() { let b = FB; println!("{}", b.build_hasher().finish()); }
