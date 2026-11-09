// expect: L0001
struct S;
impl S { fn f() {} #![allow(dead_code)] fn g() {} }
fn main() { println!("x"); }
