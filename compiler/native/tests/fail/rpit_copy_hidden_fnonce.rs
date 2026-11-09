// expect: L0301
// rustc E0382: an `impl FnOnce` is consumed by its call, even when its
// hidden closure (it captures a reference) is `Copy`.
fn reader(n: &u8) -> impl FnOnce() -> u8 + '_ {
    move || *n
}
fn main() {
    let n = 3;
    let f = reader(&n);
    let a = f();
    let b = f();
    println!("{} {}", a, b);
}
