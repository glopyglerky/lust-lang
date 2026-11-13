// expect: L0612
// rustc E0310: `X` may not live long enough; `need` requires `X: 'static`.
fn need<X: 'static>(_x: X) {}
fn g<X>(x: X) { need(x) }
fn main() {}
