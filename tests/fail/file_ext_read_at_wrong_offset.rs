// expect: L0110
// rustc E0308: FileExt::read_at takes a u64 offset.
use std::os::unix::fs::FileExt;
fn main() { let f = std::fs::File::open("/dev/null").unwrap(); let mut b = [0u8; 2]; let _ = f.read_at(&mut b, -1i64); }
