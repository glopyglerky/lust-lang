// expect: L0110
// rustc E0308: include_bytes! is `&[u8; N]` with the file's exact length (W-CHECK-G3).
fn main() { let a: &[u8; 3] = include_bytes!("../pass/include_bytes_data.bin"); println!("{}", a.len()); }
