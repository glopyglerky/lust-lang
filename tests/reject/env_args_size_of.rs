// expect: L0652
// Valid Rust (32 on x86_64); lust's Args state has another layout, so a layout query rejects.
fn main() { println!("{}", std::mem::size_of::<std::env::Args>()); }
