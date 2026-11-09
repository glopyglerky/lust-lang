// expect: L0541
// rustc deny lint unconditional_panic: N is 0.
fn main() { let s: &[u8] = b"abc"; println!("{}", s.as_chunks::<0>().0.len()); }
