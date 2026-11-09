// expect: L0308
// rustc E0596: a call through an `FnMut` bound borrows the callee
// mutably, so the binding must be `mut`.
fn call<F: FnMut() -> u8>(f: F) -> u8 {
    f()
}

fn main() {
    println!("{}", call(|| 3));
}
