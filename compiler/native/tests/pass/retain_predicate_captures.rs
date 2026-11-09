// `retain`'s predicate runs during the call and takes `&T`: its captured
// borrows do not reach the Vec's elements (tsrust T10 gostd/context.rs
// `unregister_waker`).
use std::sync::{Mutex, MutexGuard};
struct St {
    wakers: Vec<(u64, Box<dyn FnOnce() + Send>)>,
}
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}
fn unregister(m: &Mutex<St>, id: u64) -> bool {
    let mut state = lock(m);
    let before = state.wakers.len();
    state.wakers.retain(|(w, _)| *w != id);
    state.wakers.len() != before
}
fn main() {
    let m = Mutex::new(St { wakers: vec![(1, Box::new(|| ())), (3, Box::new(|| ()))] });
    println!("{} {}", unregister(&m, 1), unregister(&m, 7));
    let mut v: Vec<&'static str> = vec!["a", "bb"];
    let n = 1;
    v.retain(|s| s.len() > n);
    println!("{:?}", v);
}
