// expect: L0612
// Sol review probe `generic_static_forward`; rustc: E0310 the parameter type `T` may not live long enough
fn need<T: 'static>(_x: T) {}
fn bad<T>(x: T) { need(x); }
fn main() { bad(1); }
