// expect: L0110
// rustc E0308: a `dyn Fn(i32)` call takes an `i32`.
fn call(f: &dyn Fn(i32) -> i32) -> i32 { f("x") }
fn main() { println!("{}", call(&|n| n)); }
