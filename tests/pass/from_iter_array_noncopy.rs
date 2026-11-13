// `Vec::from_iter` over an array of non-`Copy` elements by value
// (fx-array-iter), as `collect` over the same array does.
fn main() { let v: Vec<Vec<u32>> = Vec::from_iter([vec![1], vec![2]]); println!("{:?}", v); }
