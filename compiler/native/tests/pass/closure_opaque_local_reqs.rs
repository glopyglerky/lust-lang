// A closure passed to a fn whose body holds an `impl Trait` value (a call of
// `take_once`, then the box made from it): its requirements are computed with the
// opaque value tracked, as a fn body's are (ts-rust gopanic.rs GoThread::start).
use std::sync::{Arc, Mutex};
type Main<'a, T> = Box<dyn FnOnce() -> T + Send + 'a>;
fn take_once<F: FnOnce() -> T, T>(slot: &Arc<Mutex<Option<F>>>) -> impl FnOnce() -> T + use<F, T> {
    let slot = Arc::clone(slot);
    move || {
        let f = slot.lock().unwrap().take();
        f.expect("once")()
    }
}
fn retry<H>(mut f: impl FnMut() -> H) -> H { f() }
fn start<'a, F, T, H>(f: F, mut clone: impl FnMut(Main<'a, T>) -> H) -> H
where
    F: FnOnce() -> T + Send + 'a,
    T: 'a,
{
    let slot = Arc::new(Mutex::new(Some(f)));
    retry(|| clone(Box::new(take_once(&slot))))
}
fn main() {
    println!("{}", start(|| 5, |m| m() + 1));
}
