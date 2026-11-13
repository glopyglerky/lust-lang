// expect: L0303
// rustc E0502: the run borrows v; push needs it mutably.
fn main() { let mut v = vec![1, 1, 2]; let g = v.chunk_by(|a, b| a == b).next().unwrap(); v.push(3); println!("{:?}", g); }
