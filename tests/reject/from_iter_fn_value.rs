// expect: L0104
// Valid Rust; lust takes `from_iter` only as a call, not as a function value.
fn main() { let f = Vec::<u32>::from_iter::<Vec<u32>>; println!("{:?}", f(vec![1, 2])); }
