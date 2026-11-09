// feature: dyn.fn
// form: &dyn Fn param called in the body
fn call(f: &dyn Fn(i32) -> i32) -> i32 { f(3) }
fn main() { println!("{}", call(&|n| n * 2)); }
