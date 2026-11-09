// expect: L0001
// rustc: bare CR not allowed in raw string.
fn main() { let s = r"ab"; println!("{}", s.len()); }
