// expect: L0612
// rustc E0593: the closure takes one element.
fn main() { let a = [1, 2]; let b = a.map(|x, y| x); println!("{:?}", b); }
