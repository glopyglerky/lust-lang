// expect: L0631
// rustc E0015: `for` is not allowed in a const fn (1.99).
const fn h() -> i32 { let mut s = 0; for i in 0..3 { s += i; } s }
fn main() { println!("{}", h()); }
