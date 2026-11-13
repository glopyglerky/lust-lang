// expect: L0110
// rustc E0277: `Vec<u32>` is not `FromIterator<u16>`.
fn main() { let v: Vec<u32> = <Vec<u32> as FromIterator<u16>>::from_iter([1u32]); println!("{:?}", v); }
