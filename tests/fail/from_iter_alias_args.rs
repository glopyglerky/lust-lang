// expect: L0107
// rustc E0107: a plain alias takes no generic arguments.
type V32 = Vec<u32>;
fn main() { let v = V32::<u8>::from_iter([1]); println!("{:?}", v); }
