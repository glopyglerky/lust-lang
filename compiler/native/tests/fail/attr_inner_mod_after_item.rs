// expect: L0001
mod m { pub fn f() {} #![allow(dead_code)] pub fn g() {} }
fn main() { println!("x"); }
