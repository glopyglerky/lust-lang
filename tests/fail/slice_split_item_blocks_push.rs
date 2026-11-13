// expect: L0303
// rustc E0502: a split item borrows the Vec; push needs it mutably.
fn main() { let mut v = vec![1, 0, 2]; let first = v.split(|x| *x == 0).next().unwrap(); v.push(3); println!("{:?}", first); }
