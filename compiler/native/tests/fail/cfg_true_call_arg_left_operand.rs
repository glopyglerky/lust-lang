// expect: L0001
// rustc E0658: a call argument takes outer attributes, but on `a + b` the
// attribute goes to the left operand, where it is not permitted.
fn g(x: i32) { println!("{x}"); }
fn main() { g(#[cfg(all())] 1 + 2); }
