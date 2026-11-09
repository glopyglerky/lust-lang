// expect: L0612
// rustc E0277: &[u8] is no SlicePattern (the pattern is behind two references).
fn main() { let s: &[u8] = b"ab"; let p: &[u8] = b"a"; println!("{:?}", s.strip_prefix(&p)); }
