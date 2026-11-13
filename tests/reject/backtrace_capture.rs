// expect: L0104
// Valid Rust: a captured backtrace prints rustc's frame symbols and capture()
// reads RUST_BACKTRACE at run time, so lust rejects both capture forms (tf-p6 R16).
fn main() { let b = std::backtrace::Backtrace::force_capture(); let c = std::backtrace::Backtrace::capture(); println!("{} {}", b.status() == c.status(), c); }
