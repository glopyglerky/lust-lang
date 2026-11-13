// expect: L0001
fn main() { let _x = { #![allow(dead_code)] 1 }; println!("x"); }
