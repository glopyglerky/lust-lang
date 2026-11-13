// expect: L0001
// rustc: incorrect visibility restriction
pub(a) fn f() {}
fn main() { f(); }
