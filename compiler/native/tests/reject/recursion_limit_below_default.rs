// expect: L0522
// Valid Rust: rustc also applies a lowered limit to type, trait, and layout depth,
// which lust does not track, so lust rejects any limit below the default 128.
#![recursion_limit = "64"]
macro_rules! one { () => { 1 } }
fn main() { println!("{}", one!()); }
