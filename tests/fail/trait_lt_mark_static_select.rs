// expect: L0733
// rustc E0521: only `impl Mark<'static>` exists, so `'a` must be `'static`.
trait Mark<'a> {}
struct S;
impl Mark<'static> for S {}
fn need<'a, T: Mark<'a>>(_x: &'a i32, _t: T) {}
fn bad<'a>(x: &'a i32) { need(x, S); }
fn main() { let x = 1; bad(&x); }
