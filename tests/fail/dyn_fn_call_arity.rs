// expect: L0105
// rustc E0057: a `dyn Fn(i32)` call takes one argument.
fn call(f: &dyn Fn(i32) -> i32) -> i32 { f(3, 4) }
fn main() { println!("{}", call(&|n| n)); }
