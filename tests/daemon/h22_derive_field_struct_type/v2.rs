#[derive(Debug, Clone, PartialEq)]
struct T { v: u8 }
#[derive(Debug, Clone, PartialEq)]
struct S { b: T }
fn main() { let s = S { b: T { v: 3 } }; let c = s.clone(); println!("{:?} {}", s, s == c); }
