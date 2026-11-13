// expect: L0105
// rustc E0057: `from_iter` takes one argument.
fn main() { let v: Vec<u32> = Vec::from_iter([1], [2]); println!("{:?}", v); }
