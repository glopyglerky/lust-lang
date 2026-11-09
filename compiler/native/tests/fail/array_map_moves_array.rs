// expect: L0301
// rustc E0382: map moves a non-Copy array.
fn main() { let a = [String::new()]; let b = a.map(|s| s.len()); println!("{:?} {:?}", b, a); }
