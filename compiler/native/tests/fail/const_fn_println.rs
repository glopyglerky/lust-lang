// expect: L0631
// rustc E0015: println calls non-const std::io code.
const fn h() { println!("hi"); }
fn main() { h(); }
