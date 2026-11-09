// expect: L0524
// rustc patterns_in_fns_without_body (deny): `mut x` in a trait signature.
trait T { fn f(mut x: u8); }
fn main() { println!("ok"); }
