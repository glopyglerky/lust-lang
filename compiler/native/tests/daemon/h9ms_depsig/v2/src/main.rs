use std::fmt::Write as _;
fn show(v: i64) -> String { let mut s = String::new(); write!(s, "{}", v).unwrap(); s }
fn main() { let n = util::helper(3); println!("{} {}", matches!(n, 5), show(n)); }
