// A `&F` of a `Fn` closure as a thread function: Builder::spawn_scoped and
// Scope::spawn (ts-rust emit_files.rs `.spawn_scoped(scope, &work)`, tf-p6 R23).
use std::sync::atomic::{AtomicU32, Ordering};
fn main() {
    let n = AtomicU32::new(0);
    let work = || {
        n.fetch_add(1, Ordering::Relaxed);
        n.load(Ordering::Relaxed) * 10
    };
    let r = std::thread::scope(|scope| {
        let h = std::thread::Builder::new().spawn_scoped(scope, &work).unwrap();
        let h2 = scope.spawn(&work);
        work();
        h.join().unwrap() + h2.join().unwrap()
    });
    println!("{} {}", n.into_inner(), r >= 10);
}
