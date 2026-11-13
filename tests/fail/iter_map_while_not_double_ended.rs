// expect: L0612
// rustc E0277: MapWhile is no DoubleEndedIterator.
fn main() { let v = vec![1u32]; let c: Vec<u32> = v.iter().map_while(|x| Some(*x)).rev().collect(); println!("{:?}", c); }
