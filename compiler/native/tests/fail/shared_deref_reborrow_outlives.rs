// expect: L0311
// rustc E0505: a reborrow through `&v` still needs `v` alive, even though
// the loan of `(*r)[0]` itself is not tracked.
fn main() { let v = vec![1]; let r = &v; let x = &(*r)[0]; drop(v); println!("{x}"); }
