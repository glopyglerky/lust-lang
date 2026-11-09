// expect: L0612
// rustc E0631: the closure's input type differs from the `dyn Fn` one.
fn call(f: &dyn Fn(i32) -> i32) -> i32 { f(3) }
fn main() { println!("{}", call(&|n: u8| n as i32)); }
