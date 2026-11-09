// expect: L0308
// rustc E0596: an `FnMut` callee behind `&` cannot be borrowed mutably.
fn call<F: FnMut() -> u8>(f: &F) -> u8 {
    f()
}

fn main() {
    let c = || 3;
    println!("{}", call(&c));
}
