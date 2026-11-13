// expect: L0001
// rustc parses cfg-disabled code before stripping it, so this syntax error counts.
#[cfg(any())] mod m { pub fn f() { 1 + } }
fn main() { println!("ok"); }
