// expect: L0107
// rustc E0107: `Vec::new` takes no generic arguments of its own.
type V4 = Vec<u8>;
fn main() { let v = <V4>::new::<u8>(); println!("{:?}", v); }
