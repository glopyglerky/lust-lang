// expect: L0001
// rustc: invalid start of unicode escape: `_`.
fn main() { let s = "\u{_61}"; println!("{}", s.len()); }
