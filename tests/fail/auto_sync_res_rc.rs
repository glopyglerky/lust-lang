// expect: L0612
// rustc E0277: auto trait table (ARCH 2.7): Rc and env::Args are neither Send nor Sync, and containers follow their elements.
#![allow(unused)]

fn need<T: Sync>(_t: &T) {}
fn main() { let v = { let r: Result<u8, std::rc::Rc<u8>> = Ok(1); r }; need(&v); println!("ok"); }
