// expect: L0303
// rustc E0502: the stripped slice borrows the Vec; push needs it mutably.
fn main() { let mut v = vec![1u8, 2]; let r = v.strip_prefix(&[1]).unwrap(); v.push(3); println!("{:?}", r); }
