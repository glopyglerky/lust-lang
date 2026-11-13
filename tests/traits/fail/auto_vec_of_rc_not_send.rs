// rustc-error: E0277
// lust-expect: L0612
// ARCH 2.7: Vec<T> is Send only when T is.

fn need<T: Send>(_t: &T) {}
fn main() { let v = vec![std::rc::Rc::new(1u8)]; need(&v); println!("ok"); }
