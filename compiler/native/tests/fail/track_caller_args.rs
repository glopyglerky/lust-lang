// expect: L0522
// rustc: `#[track_caller]` takes no arguments (W-CHECK-G3).
#[track_caller(x)]
fn f() {}
fn main() { f(); }
