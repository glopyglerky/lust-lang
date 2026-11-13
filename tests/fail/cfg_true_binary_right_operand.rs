// expect: L0001
// rustc E0658: an attribute on the right operand of a binary operation is
// not permitted, even a cfg that is on (hunt8 12:53: lust ran `1 + 2`).
fn f() -> i32 { 1 + #[cfg(all())] 2 }
fn main() { println!("{}", f()); }
