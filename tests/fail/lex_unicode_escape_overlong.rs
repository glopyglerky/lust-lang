// expect: L0001
// rustc: overlong unicode escape (more than six hex digits).
fn main() { let s = "\u{0000061}"; println!("{}", s.len()); }
