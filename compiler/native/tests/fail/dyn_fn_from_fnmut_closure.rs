// expect: L0612
// rustc E0594: a closure that mutates a capture is not `dyn Fn`.
fn call(f: &dyn Fn(i32) -> i32) -> i32 { f(3) }
fn main() { let mut c = 0; println!("{}", call(&|n| { c += 1; n })); }
