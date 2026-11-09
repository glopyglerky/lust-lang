// expect: L0107
// rustc E0107: `Vec::new` takes no generic arguments of its own.
fn main() { let v: Vec<u8> = Vec::new::<u8>(); println!("{:?}", v); }
