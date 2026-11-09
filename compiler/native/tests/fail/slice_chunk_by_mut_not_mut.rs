// expect: L0308
// rustc E0596: chunk_by_mut needs a mutable Vec.
fn main() { let v = vec![1, 1, 2]; for g in v.chunk_by_mut(|a, b| a == b) { g[0] = 0; } }
