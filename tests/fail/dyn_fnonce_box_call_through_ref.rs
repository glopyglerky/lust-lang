// expect: L0110
// rustc E0507: calling a `Box<dyn FnOnce>` moves it out of the reference.
fn main() { let f: Box<dyn FnOnce()> = Box::new(|| ()); let r = &f; r(); }
