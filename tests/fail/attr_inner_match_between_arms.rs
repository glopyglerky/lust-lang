// expect: L0001
fn main() { match 1 { 1 => println!("x"), #![allow(dead_code)] _ => {} } }
