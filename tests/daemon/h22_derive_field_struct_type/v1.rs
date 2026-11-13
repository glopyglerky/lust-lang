#[derive(Debug, Clone, PartialEq)]
struct T { v: u8 }
#[derive(Debug, Clone, PartialEq)]
struct S { b: u8 }
fn main() { let s = S { b: 3 }; let c = s.clone(); println!("{:?} {}", s, s == c); }
