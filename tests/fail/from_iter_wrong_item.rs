// expect: L0110
// rustc E0308: `Vec<u32>` collects `u32` items, not `&str`.
fn main() { let v: Vec<u32> = Vec::from_iter(["a"]); println!("{:?}", v); }
