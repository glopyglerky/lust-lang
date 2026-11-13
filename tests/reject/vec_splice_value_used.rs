// expect: L0641
// Valid Rust: the Splice value is used (here dropped by `let _`); lust takes splice as an expression statement only.
fn main() { let mut v = vec![1, 2]; let _ = v.splice(..1, [5]); println!("{v:?}"); }
