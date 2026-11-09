use std::fmt::Write as _;
fn show(v: i64) -> String { let mut s = String::new(); write!(s, "{}", v).unwrap(); s }
fn main() { println!("unrelated"); }
