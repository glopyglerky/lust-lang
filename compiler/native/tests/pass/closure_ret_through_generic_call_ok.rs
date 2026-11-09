// `call` returns the closure's output, a borrow of `s` that `s` outlives.
fn call<R, F: FnOnce() -> R>(f: F) -> R { f() }
fn main() {
    let s = String::from("s");
    let r = call(|| &s);
    println!("{}", r);
}
