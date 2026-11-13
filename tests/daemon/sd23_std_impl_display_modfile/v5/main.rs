// sd23: a written Display impl in a module file, removed and added again (hunt11 21:30).
mod m;
struct S { a: u32 }
fn other() -> u32 { 3 }
fn main() { let s = S { a: 1 }; println!("{} {}", s, s.to_string()); println!("{}", other()); }
