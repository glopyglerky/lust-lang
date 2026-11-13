// expect: L0612
// rustc E0593: the predicate takes two elements.
fn main() { let v = vec![1, 1, 2]; let g: Vec<_> = v.chunk_by(|a| true).collect(); }
