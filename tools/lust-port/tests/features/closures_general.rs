// feature: closures.general
// form: closure passed to a user fn, move capture, and a plain fn pointer value
fn apply<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 { f(x) }
fn inc(n: i32) -> i32 { n + 1 }
fn main() {
    let base = 10;
    let add = move |x: i32| x + base;
    let ptr: fn(i32) -> i32 = inc;
    println!("{} {}", apply(add, 1), ptr(2));
}
