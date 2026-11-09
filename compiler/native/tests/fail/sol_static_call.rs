// expect: L0306
// Sol review probe `static_call`; rustc: E0597 `x` does not live long enough
fn take(_x: &'static i32) {}
fn main() { let x = 1; take(&x); }
