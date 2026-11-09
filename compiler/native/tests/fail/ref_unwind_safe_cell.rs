// expect: L0612
// Cell<T> is not RefUnwindSafe.
fn needs_ref<T: std::panic::RefUnwindSafe>(_t: &T) {}
fn main() { let c = std::cell::Cell::new(1); needs_ref(&c); }
