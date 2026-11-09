// expect: L0541
// rustc deny lint unconditional_panic: N is { 2 - 2 }.
fn main() { let s: &[u8] = b"abc"; println!("{}", s.as_chunks::<{ 2 - 2 }>().0.len()); }
