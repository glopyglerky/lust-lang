// Sol review probe `positive_generic_bound`: rustc accepts.
fn need<T: 'static>(_x: T) {}
fn good<T: 'static>(x: T) { need(x); }
fn main() { good(1); }
