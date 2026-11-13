// expect: L0301
// rustc E0382: the first call consumed the box.
fn main() { let f: Box<dyn FnOnce()> = Box::new(|| ()); f(); f(); }
