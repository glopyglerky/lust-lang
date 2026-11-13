// expect: L0104
// rustc E0599: read_at needs `use std::os::unix::fs::FileExt`.
fn main() { let f = std::fs::File::open("/dev/null").unwrap(); let mut b = [0u8; 2]; let _ = f.read_at(&mut b, 0); }
