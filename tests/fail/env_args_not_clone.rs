// expect: L0104
// rustc E0599: std::env::Args is not Clone.
fn main() { let a = std::env::args(); let b = a.clone(); println!("{}", b.len()); }
