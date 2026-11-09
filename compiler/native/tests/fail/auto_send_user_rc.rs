// expect: L0612
// rustc E0277: auto trait table (ARCH 2.7): Rc and env::Args are neither Send nor Sync, and containers follow their elements.
#![allow(unused)]
struct S { r: std::rc::Rc<u8> }
fn need<T: Send>(_t: &T) {}
fn main() { let v = S { r: std::rc::Rc::new(1u8) }; need(&v); println!("ok"); }
