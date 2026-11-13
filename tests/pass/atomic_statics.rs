use std::sync::atomic::{AtomicUsize, AtomicBool, AtomicI32, Ordering};
static A: AtomicUsize = AtomicUsize::new(3);
static B: AtomicBool = AtomicBool::new(true);
static C: [AtomicI32; 3] = [AtomicI32::new(1), AtomicI32::new(2), AtomicI32::new(-3)];
const Z: AtomicUsize = AtomicUsize::new(9);
fn main() {
    A.fetch_add(Z.load(Ordering::Relaxed), Ordering::SeqCst);
    Z.store(100, Ordering::SeqCst);
    C[1].fetch_add(40, Ordering::SeqCst);
    println!("{} {} {:?} {}", A.load(Ordering::SeqCst), B.fetch_and(false, Ordering::SeqCst), C, Z.load(Ordering::SeqCst));
}
