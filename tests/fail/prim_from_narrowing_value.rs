// expect: L0612
// rustc E0277: `u32: From<u64>` is not implemented (narrowing).
fn main() { let v: Vec<u32> = [1u64, 2].iter().copied().map(u32::from).collect(); println!("{v:?}"); }
