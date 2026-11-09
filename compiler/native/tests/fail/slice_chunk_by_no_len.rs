// expect: L0104
// rustc E0599: ChunkBy is no ExactSizeIterator.
fn main() { let v = vec![1, 1, 2]; println!("{}", v.chunk_by(|a, b| a == b).len()); }
