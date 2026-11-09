// expect: L0301
// rustc E0382: calling an `impl FnOnce` moves it. The hidden closure
// moves a `T` out (FnOnce kind), and `T = i32` makes it `Copy` here.
fn wrap<T>(t: T) -> impl FnOnce() -> T {
    move || t
}
fn main() {
    let h = wrap(5);
    let a = h();
    let g = h;
    println!("{}", a);
}
