// expect: L0612
// rustc E0277: auto trait table (ARCH 2.7): Rc and env::Args are neither Send nor Sync, and containers follow their elements.
#![allow(unused)]
struct W<T> { t: T }
fn need<T: Send>(_t: &T) {}
fn main() { let v = W { t: std::rc::Rc::new(1u8) }; need(&v); println!("ok"); }
