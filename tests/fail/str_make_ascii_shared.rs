// expect: L0308
// rustc E0596: make_ascii_uppercase needs &mut str.
fn main() { let s = "ab"; s.make_ascii_uppercase(); println!("{s}"); }
