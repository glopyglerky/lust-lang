// expect: L0107
// rustc E0107: `Vec::new` takes no generic arguments of its own.
type VS<T> = Vec<T>;
fn main() { let v: Vec<u8> = VS::new::<u8>(); println!("{:?}", v); }
