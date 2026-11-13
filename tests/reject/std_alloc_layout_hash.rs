// expect: L0612
// Valid Rust: Layout is Hash in std; lust leaves Hash out (std hashes the align through a private enum).
use std::alloc::Layout;
use std::collections::HashSet;
fn main() { let mut s = HashSet::new(); s.insert(Layout::new::<u8>()); println!("{}", s.len()); }
