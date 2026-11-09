// expect: L0631
// rustc E0015: a const fn may only call const methods.
struct S;
impl S { fn m(&self) -> i32 { 3 } }
const fn h(s: S) -> i32 { s.m() }
fn main() { println!("{}", h(S)); }
