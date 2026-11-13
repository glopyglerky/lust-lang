// expect: L0306
// rustc E0597: the item borrows v, which drops at the block end.
fn main() { let r = { let v = vec![1]; v.iter().zip(0..1).next_back() }; println!("{:?}", r); }
