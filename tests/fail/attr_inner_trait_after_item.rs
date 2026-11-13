// expect: L0001
trait T { fn f(); #![allow(dead_code)] fn g(); }
fn main() { println!("x"); }
