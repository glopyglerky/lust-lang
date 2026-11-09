// expect: L0001
enum E { #![allow(dead_code)] A }
fn main() { let _e = E::A; println!("x"); }
