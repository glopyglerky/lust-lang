// expect: L0107
// rustc E0107: a plain alias takes no generic arguments.
type V4 = Vec<u8>;
fn main() { let v = V4::<u8>::new(); println!("{:?}", v); }
