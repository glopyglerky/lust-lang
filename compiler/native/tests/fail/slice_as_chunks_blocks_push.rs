// expect: L0303
// rustc E0502: the chunks borrow v; push needs it mutably.
fn main() { let mut v = vec![1u8, 2, 3, 4]; let (a, _) = v.as_chunks::<2>(); v.push(1); println!("{:?}", a); }
