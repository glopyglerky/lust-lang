// expect: L0631
// rustc E0015: closures cannot be called in a const fn.
const fn h() -> i32 { let f = || 3; f() }
fn main() { println!("{}", h()); }
