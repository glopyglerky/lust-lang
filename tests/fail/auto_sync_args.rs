// expect: L0612
// rustc E0277: auto trait table (ARCH 2.7): Rc and env::Args are neither Send nor Sync, and containers follow their elements.
#![allow(unused)]

fn need<T: Sync>(_t: &T) {}
fn main() { let v = std::env::args(); need(&v); println!("ok"); }
