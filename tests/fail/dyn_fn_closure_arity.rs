// expect: L0612
// rustc E0593: the closure takes two inputs, the `dyn Fn` one.
fn call(f: &dyn Fn(i32) -> i32) -> i32 { f(3) }
fn main() { println!("{}", call(&|a, b| a + b)); }
