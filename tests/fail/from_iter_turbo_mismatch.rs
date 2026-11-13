// expect: L0110
// rustc E0308: the item turbofish names the `IntoIterator` type, and an array is not a `Vec`.
fn main() { let v = Vec::<u32>::from_iter::<Vec<u32>>([1, 2]); println!("{:?}", v); }
