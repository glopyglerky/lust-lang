// expect: L0001
// rustc parses cfg-disabled code before stripping it, so this syntax error counts.
#[cfg(any())] fn f<'fn>() {}
fn main() { println!("ok"); }
