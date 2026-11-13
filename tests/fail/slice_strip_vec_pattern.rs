// expect: L0612
// rustc E0277: Vec<u8> is no SlicePattern; the generic pattern takes no deref coercion.
fn main() { let s: &[u8] = b"ab"; let v = vec![b'a']; println!("{:?}", s.strip_prefix(&v)); }
