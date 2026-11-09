// expect: L0110
// rustc E0308: [[u8; 2]] is not [[u8; 3]].
fn main() { let s: &[u8] = b"abcd"; let (a, _) = s.as_chunks::<2>(); let x: &[[u8; 3]] = a; }
