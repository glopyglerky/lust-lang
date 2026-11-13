// A closure that moves a Vec of integer literals is `Send` before the
// literals settle (`impl FnOnce() -> R + Send + 'static`).
fn spawnish<R>(f: impl FnOnce() -> R + Send + 'static) -> R {
    f()
}
fn main() {
    let v = vec![1, 2];
    println!("{}", spawnish(move || v.len()));
    let x = 2.5;
    println!("{}", spawnish(move || x * 2.0));
}
