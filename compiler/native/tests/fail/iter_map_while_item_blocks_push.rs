// expect: L0303
// rustc E0502: the item borrows v; push needs it mutably.
fn main() { let mut v = vec![1u32]; let mut it = v.iter().map_while(|x| Some(x)); let a = it.next(); v.push(2); println!("{:?}", a); }
