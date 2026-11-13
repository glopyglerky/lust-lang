// expect: L0104
// rustc E0599: MapWhile is no ExactSizeIterator.
fn main() { let v = vec![1u32]; println!("{}", v.iter().map_while(|x| Some(*x)).len()); }
