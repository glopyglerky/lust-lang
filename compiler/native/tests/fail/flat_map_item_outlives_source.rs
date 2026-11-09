// expect: L0306
// rustc E0597: flat_map items still borrow the source.
fn main() { let r: Vec<&u32>; { let v = vec![1]; r = v.iter().flat_map(|x| vec![x]).collect(); } println!("{}", r.len()); }
