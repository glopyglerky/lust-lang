// expect: L0306
// rustc E0597: the item borrows v, which drops at the block end.
fn main() { let r = { let v = vec![1u32]; v.iter().map_while(|x| Some(x)).next() }; println!("{:?}", r); }
