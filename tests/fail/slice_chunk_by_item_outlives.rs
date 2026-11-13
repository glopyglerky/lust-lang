// expect: L0306
// rustc E0597: the run borrows v, which drops at the block end.
fn main() { let g = { let v = vec![1, 1, 2]; v.chunk_by(|a, b| a == b).next() }; println!("{:?}", g); }
