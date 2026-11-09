// expect: L0305
// rustc E0507: calling an `impl FnOnce` through a reference moves out
// of the reference, though the hidden closure is `Copy` here.
fn wrap<T>(t: T) -> impl FnOnce() -> T {
    move || t
}
fn main() {
    let h = wrap(5);
    let r = &h;
    println!("{}", r());
}
