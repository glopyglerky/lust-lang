// expect: L0110
// rustc E0308: u32 items compared with a &u64.
fn main() { let mut p = vec![1u32].into_iter().peekable(); println!("{:?}", p.next_if_eq(&1u64)); }
