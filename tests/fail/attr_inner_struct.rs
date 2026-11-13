// expect: L0001
struct S { #![allow(dead_code)] x: i64 }
fn main() { let _s = S { x: 1 }; println!("x"); }
