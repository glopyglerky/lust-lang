// expect: L0641
// Valid Rust: Debug of std::env::Args prints Args { inner: [..] }; lust has no rendering for it yet.
fn main() { println!("{:?}", std::env::args()); }
