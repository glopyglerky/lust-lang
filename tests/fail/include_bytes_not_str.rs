// expect: L0110
// rustc E0308: include_bytes! gives bytes, not `&str` (W-CHECK-G3).
fn main() { let s: &str = include_bytes!("../pass/include_bytes_data.bin"); println!("{}", s); }
