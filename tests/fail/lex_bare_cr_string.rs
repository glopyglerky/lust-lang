// expect: L0001
// rustc: bare CR not allowed in string (a CR with no LF after it).
fn main() { let s = "ab"; println!("{}", s.len()); }
