// expect: L0306
// rustc E0597: extended items borrow the shorter-lived Vec.
fn main() { let mut e: Vec<&str> = vec![]; { let a = vec![String::new()]; e.extend(a.iter().map(|k| k.as_str())); } println!("{}", e.len()); }
