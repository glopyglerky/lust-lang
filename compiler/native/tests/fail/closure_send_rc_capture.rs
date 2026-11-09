// expect: L0612
// rustc E0277: a closure that moves an `Rc` is not `Send`.
use std::rc::Rc;
fn spawnish<R>(f: impl FnOnce() -> R + Send + 'static) -> R {
    f()
}
fn main() {
    let v = Rc::new(1);
    println!("{}", spawnish(move || Rc::strong_count(&v)));
}
