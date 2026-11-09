// expect: L0612
// rustc E0508: map takes the array by value; a non-Copy array cannot move out of a reference.
fn main() { let a = [String::new()]; let r = &a; let b = r.map(|s| s.len()); println!("{:?}", b); }
