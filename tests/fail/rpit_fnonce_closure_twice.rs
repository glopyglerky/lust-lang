// expect: L0301
// rustc E0382: a closure that calls a captured `impl FnOnce` is `FnOnce`
// itself, though the opaque's hidden closure is `Copy` here.
fn wrap<T>(t: T) -> impl FnOnce() -> T {
    move || t
}
fn main() {
    let h = wrap(5);
    let c = move || {
        let _ = h();
    };
    c();
    c();
}
