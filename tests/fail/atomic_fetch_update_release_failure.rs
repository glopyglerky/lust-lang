// expect: L0542
// rustc invalid_atomic_ordering (deny): fetch_update's failure ordering may not be
// Release or AcqRel, since a failed fetch_update does not result in a write.
use std::sync::atomic::{AtomicU64, Ordering};
fn main() {
    let a = AtomicU64::new(7);
    let _ = a.fetch_update(Ordering::Relaxed, Ordering::AcqRel, |c| Some(c + 1));
}
