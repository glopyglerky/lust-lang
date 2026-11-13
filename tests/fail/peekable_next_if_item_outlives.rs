// expect: L0306
// rustc E0597: the item borrows v, which drops at the block end.
fn main() { let x = { let v = vec![1]; let mut p = v.iter().peekable(); p.next_if(|_| true) }; println!("{:?}", x); }
