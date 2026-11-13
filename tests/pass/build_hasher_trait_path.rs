// A trait-name path call through the expanded std trait path
// (`BuildHasher::build_hasher`), like the method form.
use std::hash::{BuildHasher, Hasher};
#[derive(Default)] struct F(u64);
impl Hasher for F { fn finish(&self) -> u64 { self.0 } fn write(&mut self, b: &[u8]) { for &x in b { self.0 += x as u64; } } }
struct FB;
impl BuildHasher for FB { type Hasher = F; fn build_hasher(&self) -> F { F(1) } }
fn main() { let b = FB; println!("{}", BuildHasher::build_hasher(&b).finish()); }
