// expect: L0640
// Valid Rust: slice clone_into clones element by element through
// each element's clone_from, which lust has no form for yet.
use std::borrow::ToOwned;

fn main() {
    let src = vec!["a".to_string()];
    let mut dst: Vec<String> = Vec::new();
    src[..].clone_into(&mut dst);
    println!("{:?}", dst);
}
