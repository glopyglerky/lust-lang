// expect: L0631
// rustc: function pointer calls are not allowed in a const fn.
fn g() -> i32 { 3 }
const fn h(f: fn() -> i32) -> i32 { f() }
fn main() { println!("{}", h(g)); }
