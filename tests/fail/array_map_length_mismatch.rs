// expect: L0110
// rustc E0308: map keeps the length.
fn main() { let a = [1, 2]; let b: [u8; 3] = a.map(|x| x as u8); println!("{:?}", b); }
