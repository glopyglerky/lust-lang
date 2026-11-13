// expect: L0631
// rustc E0015: a const fn may only call const fns.
fn g() -> i32 { 3 }
const fn h() -> i32 { g() }
fn main() { println!("{}", h()); }
