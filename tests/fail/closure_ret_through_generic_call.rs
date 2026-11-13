// expect: L0311
// rustc E0505: `call` returns the closure's output, which borrows `s`.

fn call<R, F: FnOnce() -> R>(f: F) -> R { f() }
fn main() {
    let s = String::from("s");
    let r = call(|| &s);
    drop(s);
    println!("{}", r);
}
