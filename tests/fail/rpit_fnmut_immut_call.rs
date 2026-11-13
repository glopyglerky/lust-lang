// expect: L0308
// rustc E0596: calling an `impl FnMut` borrows it mutably, even when its
// hidden closure is an `Fn` closure.
fn wrap<T: Copy>(t: T) -> impl FnMut() -> T {
    move || t
}
fn main() {
    let h = wrap(5);
    let g = h;
    println!("{}", g());
}
