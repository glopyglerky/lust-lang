// expect: L0001
// rustc: invalid character in numeric character escape: `+`.
fn main() { let s = "\x+1"; println!("{}", s.len()); }
