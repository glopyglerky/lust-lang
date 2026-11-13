// compare_exchange_weak with a release failure order panics in core's
// atomic.rs (no track_caller), on its own line.
// panic: there is no such thing as a release failure ordering
use std::sync::atomic::{AtomicUsize, Ordering};
fn main() {
    let a = AtomicUsize::new(1);
    let f = if a.load(Ordering::SeqCst) == 1 { Ordering::Release } else { Ordering::SeqCst };
    println!("{:?}", a.compare_exchange_weak(1, 2, Ordering::SeqCst, Ordering::Relaxed).is_ok());
    println!("{:?}", a.compare_exchange_weak(2, 3, Ordering::SeqCst, f));
}
