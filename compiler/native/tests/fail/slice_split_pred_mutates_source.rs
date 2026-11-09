// expect: L0303
// rustc E0502: the predicate cannot push to the slice it splits.
fn main() { let mut v = vec![1, 0, 2]; let n = v.split(|x| { v.push(1); *x == 0 }).count(); println!("{}", n); }
