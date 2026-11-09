// expect: L0110
// rustc E0308: ilog's base has the receiver's type.
fn main() { let x: u64 = 5; let b: u32 = 2; println!("{}", x.ilog(b)); }
