// expect: L0111
// rustc E0284: nothing fixes N.
fn main() { let s: &[u8] = b"abc"; let (a, b) = s.as_chunks(); println!("{} {}", a.len(), b.len()); }
