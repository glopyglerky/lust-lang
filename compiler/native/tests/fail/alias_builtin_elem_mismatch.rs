// expect: L0110
// rustc E0277: the alias fixes `Vec<u8>`, which has no `From<[u16; 1]>`.
type V4 = Vec<u8>;
fn main() { let v = V4::from([300u16]); println!("{:?}", v); }
