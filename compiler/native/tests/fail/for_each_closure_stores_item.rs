// expect: L0306
// rustc E0597: the closure stores items of a shorter-lived Vec.
fn main() { let mut keep: Vec<&u32> = vec![]; { let v = vec![1]; v.iter().for_each(|x| keep.push(x)); } println!("{}", keep.len()); }
