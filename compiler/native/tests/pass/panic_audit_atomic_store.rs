// A store with an acquire order panics in core's atomic.rs.
// panic: there is no such thing as an acquire store
use std::sync::atomic::{AtomicBool, Ordering};
fn main() {
    let b = AtomicBool::new(false);
    let o = if b.load(Ordering::SeqCst) { Ordering::SeqCst } else { Ordering::Acquire };
    println!("before");
    b.store(true, o);
    println!("after");
}
