// expect: L0721
// Valid Rust: N from a free const item by path; lust takes a literal, a block, or a const parameter (R13).
const K: usize = 2; fn main() { let s: &[u8] = b"abcde"; let (a, b) = s.as_chunks::<K>(); println!("{:?} {:?}", a, b); }
