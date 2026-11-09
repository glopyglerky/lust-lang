// expect: L0631
// rustc E0015 even when the const fn is never called.
fn g() -> i32 { 3 }
const fn h() -> i32 { g() }
fn main() { println!("ok"); }
