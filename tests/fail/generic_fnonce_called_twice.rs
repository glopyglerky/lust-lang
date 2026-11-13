// expect: L0301
// rustc E0382: a call through an `FnOnce` bound moves the callee.
fn twice<F: FnOnce() -> u8>(f: F) -> u8 {
    f() + f()
}

fn main() {
    println!("{}", twice(|| 3));
}
