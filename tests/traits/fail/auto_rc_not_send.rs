// rustc-error: E0277
// lust-expect: L0612
// ARCH 2.7 table: Rc<T> is never Send.

fn need<T: Send>(_t: &T) {}
fn main() { let v = std::rc::Rc::new(1u8); need(&v); println!("ok"); }
