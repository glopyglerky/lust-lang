// expect: L0303
// rustc E0502: the item borrows v; push needs it mutably.
fn main() { let mut v = vec![1u32]; let mut p = v.iter().peekable(); let a = p.next_if(|_| true); v.push(2); println!("{:?}", a); }
