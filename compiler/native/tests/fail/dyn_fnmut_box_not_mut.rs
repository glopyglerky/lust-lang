// expect: L0308
// rustc E0596: calling a `Box<dyn FnMut>` needs a mutable binding.
fn main() { let f: Box<dyn FnMut()> = Box::new(|| ()); f(); }
