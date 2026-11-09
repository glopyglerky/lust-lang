// expect: L0104
// rustc E0599: next_if is a Peekable method.
fn main() { let v = vec![1u32]; let mut p = v.iter(); println!("{:?}", p.next_if(|x| **x == 1)); }
