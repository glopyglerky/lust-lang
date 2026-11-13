// expect: L0306
// rustc E0597: the outputs borrow v, which drops at the block end.
fn main() { let r = { let v = vec![1, 2]; [0usize, 1].map(|i| &v[i]) }; println!("{:?}", r); }
